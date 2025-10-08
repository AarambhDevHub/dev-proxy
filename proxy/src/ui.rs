use crate::mock::MockManager;
use crate::storage::{FilterOptions, Storage};
use anyhow::Result;
use bytes::Bytes;
use http::{Method, StatusCode, header};
use mime_guess::from_path;
use rust_embed::RustEmbed;
use serde_json::json;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(RustEmbed)]
#[folder = "../ui/build"]
struct Assets;

pub async fn start_ui_server(port: u16, storage: Storage, mock_manager: MockManager) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    println!("UI server listening on {}", addr);

    let storage = Arc::new(storage);
    let mock_manager = Arc::new(mock_manager);

    loop {
        let (stream, _) = listener.accept().await?;
        let storage = storage.clone();
        let mock_manager = mock_manager.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, storage, mock_manager).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    storage: Arc<Storage>,
    mock_manager: Arc<MockManager>,
) -> Result<()> {
    let io = hyper_util::rt::TokioIo::new(stream);

    let service = hyper::service::service_fn(move |req| {
        let storage = storage.clone();
        let mock_manager = mock_manager.clone();
        async move { handle_request(req, storage, mock_manager).await }
    });

    hyper::server::conn::http1::Builder::new()
        .serve_connection(io, service)
        .await?;

    Ok(())
}

async fn handle_request(
    req: hyper::Request<hyper::body::Incoming>,
    storage: Arc<Storage>,
    mock_manager: Arc<MockManager>,
) -> Result<hyper::Response<http_body_util::Full<Bytes>>, Infallible> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let query = req.uri().query().map(|q| q.to_string());

    // API routes
    if path.starts_with("/api/") {
        return handle_api_request(method, path, query, req, storage, mock_manager).await;
    }

    // Serve static files
    serve_static_file(&path).await
}

async fn handle_api_request(
    method: Method,
    path: String,
    query: Option<String>,
    req: hyper::Request<hyper::body::Incoming>,
    storage: Arc<Storage>,
    mock_manager: Arc<MockManager>,
) -> Result<hyper::Response<http_body_util::Full<Bytes>>, Infallible> {
    match (method.as_str(), path.as_str()) {
        // Existing endpoints
        ("GET", "/api/recordings") => {
            let filters = parse_filter_options(query.as_deref());
            let recordings = if has_filters(&filters) {
                storage.get_filtered(&filters)
            } else {
                storage.get_all()
            };
            let json = serde_json::to_string(&recordings).unwrap();
            Ok(json_response(json))
        }
        ("GET", "/api/stats") => {
            let stats = storage.get_stats();
            let json = serde_json::to_string(&stats).unwrap();
            Ok(json_response(json))
        }
        ("GET", "/api/analytics") => {
            let analytics = storage.get_analytics();
            let json = serde_json::to_string(&analytics).unwrap();
            Ok(json_response(json))
        }
        ("GET", p) if p.starts_with("/api/recordings/") => {
            let id = p.trim_start_matches("/api/recordings/");
            if let Some(recording) = storage.get_by_id(id) {
                let json = serde_json::to_string(&recording).unwrap();
                Ok(json_response(json))
            } else {
                Ok(not_found_response())
            }
        }
        ("DELETE", "/api/recordings") => {
            storage.clear();
            Ok(json_response(json!({"success": true}).to_string()))
        }

        // Mock endpoints
        ("GET", "/api/mocks") => {
            let rules = mock_manager.get_all_rules();
            let json = serde_json::to_string(&rules).unwrap();
            Ok(json_response(json))
        }
        ("POST", "/api/mocks") => match read_body_json::<crate::mock::CreateMockRule>(req).await {
            Ok(rule) => {
                let id = mock_manager.add_rule(rule);
                Ok(json_response(json!({"id": id}).to_string()))
            }
            Err(e) => Ok(error_response(&format!("Invalid request: {}", e))),
        },
        ("GET", p) if p.starts_with("/api/mocks/") && !p.ends_with("/toggle") => {
            let id = p.trim_start_matches("/api/mocks/");
            if let Some(rule) = mock_manager.get_rule(id) {
                let json = serde_json::to_string(&rule).unwrap();
                Ok(json_response(json))
            } else {
                Ok(not_found_response())
            }
        }
        ("PUT", p) if p.starts_with("/api/mocks/") => {
            match read_body_json::<crate::mock::UpdateMockRule>(req).await {
                Ok(rule) => {
                    if mock_manager.update_rule(rule) {
                        Ok(json_response(json!({"success": true}).to_string()))
                    } else {
                        Ok(not_found_response())
                    }
                }
                Err(e) => Ok(error_response(&format!("Invalid request: {}", e))),
            }
        }
        ("POST", p) if p.ends_with("/toggle") => {
            let id = p
                .trim_start_matches("/api/mocks/")
                .trim_end_matches("/toggle");
            if mock_manager.toggle_rule(id) {
                Ok(json_response(json!({"success": true}).to_string()))
            } else {
                Ok(not_found_response())
            }
        }
        ("DELETE", p) if p.starts_with("/api/mocks/") => {
            let id = p.trim_start_matches("/api/mocks/");
            if mock_manager.delete_rule(id) {
                Ok(json_response(json!({"success": true}).to_string()))
            } else {
                Ok(not_found_response())
            }
        }
        ("DELETE", "/api/mocks") => {
            mock_manager.clear_all();
            Ok(json_response(json!({"success": true}).to_string()))
        }

        _ => Ok(not_found_response()),
    }
}

async fn read_body_json<T: serde::de::DeserializeOwned>(
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<T, String> {
    use http_body_util::BodyExt;

    let body = req.into_body();
    let bytes = body
        .collect()
        .await
        .map_err(|e| format!("Failed to read body: {}", e))?
        .to_bytes();

    serde_json::from_slice(&bytes).map_err(|e| format!("Failed to parse JSON: {}", e))
}

fn parse_filter_options(query: Option<&str>) -> FilterOptions {
    let mut filters = FilterOptions {
        search: None,
        method: None,
        status: None,
        min_duration: None,
        max_duration: None,
        from_time: None,
        to_time: None,
    };

    if let Some(query_str) = query {
        for param in query_str.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                let decoded_value = urlencoding::decode(value).unwrap_or_default();
                match key {
                    "search" if !decoded_value.is_empty() => {
                        filters.search = Some(decoded_value.to_string());
                    }
                    "method" if !decoded_value.is_empty() => {
                        filters.method = Some(decoded_value.to_string());
                    }
                    "status" => {
                        if let Ok(status) = decoded_value.parse::<u16>() {
                            filters.status = Some(status);
                        }
                    }
                    "minDuration" => {
                        if let Ok(duration) = decoded_value.parse::<u64>() {
                            filters.min_duration = Some(duration);
                        }
                    }
                    "maxDuration" => {
                        if let Ok(duration) = decoded_value.parse::<u64>() {
                            filters.max_duration = Some(duration);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    filters
}

fn has_filters(filters: &FilterOptions) -> bool {
    filters.search.is_some()
        || filters.method.is_some()
        || filters.status.is_some()
        || filters.min_duration.is_some()
        || filters.max_duration.is_some()
        || filters.from_time.is_some()
        || filters.to_time.is_some()
}

async fn serve_static_file(
    path: &str,
) -> Result<hyper::Response<http_body_util::Full<Bytes>>, Infallible> {
    let path = path.trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => {
            let mime = from_path(path).first_or_octet_stream();
            let bytes = Bytes::from(content.data.into_owned());
            Ok(hyper::Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(http_body_util::Full::new(bytes))
                .unwrap())
        }
        None => {
            if let Some(index) = Assets::get("index.html") {
                let bytes = Bytes::from(index.data.into_owned());
                Ok(hyper::Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(http_body_util::Full::new(bytes))
                    .unwrap())
            } else {
                Ok(not_found_response())
            }
        }
    }
}

fn json_response(json: String) -> hyper::Response<http_body_util::Full<Bytes>> {
    hyper::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(http_body_util::Full::new(Bytes::from(json)))
        .unwrap()
}

fn error_response(message: &str) -> hyper::Response<http_body_util::Full<Bytes>> {
    let json = json!({"error": message}).to_string();
    hyper::Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(http_body_util::Full::new(Bytes::from(json)))
        .unwrap()
}

fn not_found_response() -> hyper::Response<http_body_util::Full<Bytes>> {
    hyper::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(http_body_util::Full::new(Bytes::from("Not Found")))
        .unwrap()
}
