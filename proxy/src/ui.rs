use crate::latency_injector::{CreateLatencyRule, LatencyInjector, UpdateLatencyRule};
use crate::mock::MockManager;
use crate::modifier::{CreateModifierRule, ResponseModifier, UpdateModifierRule};
use crate::rate_limiter::{CreateRateLimitRule, RateLimiter, UpdateRateLimitRule};
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

pub async fn start_ui_server(
    port: u16,
    storage: Storage,
    mock_manager: MockManager,
    response_modifier: ResponseModifier,
    rate_limiter: RateLimiter,
    latency_injector: LatencyInjector,
) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    println!("UI server listening on {}", addr);

    let storage = Arc::new(storage);
    let mock_manager = Arc::new(mock_manager);
    let response_modifier = Arc::new(response_modifier);
    let rate_limiter = Arc::new(rate_limiter);
    let latency_injector = Arc::new(latency_injector);

    loop {
        let (stream, _) = listener.accept().await?;
        let storage = storage.clone();
        let mock_manager = mock_manager.clone();
        let response_modifier = response_modifier.clone();
        let rate_limiter = rate_limiter.clone();
        let latency_injector = latency_injector.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(
                stream,
                storage,
                mock_manager,
                response_modifier,
                rate_limiter,
                latency_injector,
            )
            .await
            {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    storage: Arc<Storage>,
    mock_manager: Arc<MockManager>,
    response_modifier: Arc<ResponseModifier>,
    rate_limiter: Arc<RateLimiter>,
    latency_injector: Arc<LatencyInjector>,
) -> Result<()> {
    let io = hyper_util::rt::TokioIo::new(stream);

    let service = hyper::service::service_fn(move |req| {
        let storage = storage.clone();
        let mock_manager = mock_manager.clone();
        let response_modifier = response_modifier.clone();
        let rate_limiter = rate_limiter.clone();
        let latency_injector = latency_injector.clone();
        async move {
            handle_request(
                req,
                storage,
                mock_manager,
                response_modifier,
                rate_limiter,
                latency_injector,
            )
            .await
        }
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
    response_modifier: Arc<ResponseModifier>,
    rate_limiter: Arc<RateLimiter>,
    latency_injector: Arc<LatencyInjector>,
) -> Result<hyper::Response<http_body_util::Full<Bytes>>, Infallible> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let query = req.uri().query().map(|q| q.to_string());

    // API routes
    if path.starts_with("/api/") {
        return handle_api_request(
            method,
            path,
            query,
            req,
            storage,
            mock_manager,
            response_modifier,
            rate_limiter,
            latency_injector,
        )
        .await;
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
    response_modifier: Arc<ResponseModifier>,
    rate_limiter: Arc<RateLimiter>,
    latency_injector: Arc<LatencyInjector>,
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
        ("POST", p) if p.starts_with("/api/recordings/") && p.ends_with("/replay") => {
            let id = p
                .trim_start_matches("/api/recordings/")
                .trim_end_matches("/replay");

            if let Some(replay_req) = storage.get_for_replay(id) {
                // Get the upstream URL from query params or use default
                let upstream_url = query
                    .as_deref()
                    .and_then(|q| {
                        q.split('&')
                            .find(|param| param.starts_with("upstream="))
                            .map(|param| {
                                urlencoding::decode(&param[9..])
                                    .unwrap_or_default()
                                    .to_string()
                            })
                    })
                    .unwrap_or_else(|| "http://localhost:8000".to_string());

                match replay_request(&replay_req, &upstream_url).await {
                    Ok(response) => {
                        let json = serde_json::to_string(&response).unwrap();
                        Ok(json_response(json))
                    }
                    Err(e) => Ok(error_response(&format!("Replay failed: {}", e))),
                }
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
        ("POST", p) if p.starts_with("/api/mocks/") && p.ends_with("/toggle") => {
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
        // Modifier endpoints - ADD THESE
        ("GET", "/api/modifiers") => {
            let rules = response_modifier.get_all_rules();
            let json = serde_json::to_string(&rules).unwrap();
            Ok(json_response(json))
        }
        ("POST", "/api/modifiers") => match read_body_json::<CreateModifierRule>(req).await {
            Ok(rule) => {
                let id = response_modifier.add_rule(rule);
                Ok(json_response(json!({" id": id}).to_string()))
            }
            Err(e) => Ok(error_response(&format!("Invalid request: {}", e))),
        },
        ("GET", p) if p.starts_with("/api/modifiers/") && !p.ends_with("/toggle") => {
            let id = p.trim_start_matches("/api/modifiers/");
            if let Some(rule) = response_modifier.get_rule(id) {
                let json = serde_json::to_string(&rule).unwrap();
                Ok(json_response(json))
            } else {
                Ok(not_found_response())
            }
        }
        ("PUT", p) if p.starts_with("/api/modifiers/") => {
            match read_body_json::<UpdateModifierRule>(req).await {
                Ok(rule) => {
                    if response_modifier.update_rule(rule) {
                        Ok(json_response(json!({"success": true}).to_string()))
                    } else {
                        Ok(not_found_response())
                    }
                }
                Err(e) => Ok(error_response(&format!("Invalid request: {}", e))),
            }
        }
        ("POST", p) if p.starts_with("/api/modifiers/") && p.ends_with("/toggle") => {
            let parts: Vec<&str> = p.split('/').collect();
            if parts.len() >= 4 {
                let id = parts[3]; // Gets the ID part
                if response_modifier.toggle_rule(id) {
                    Ok(json_response(json!({"success": true}).to_string()))
                } else {
                    Ok(not_found_response())
                }
            } else {
                Ok(not_found_response())
            }
        }
        ("DELETE", p) if p.starts_with("/api/modifiers/") => {
            let id = p.trim_start_matches("/api/modifiers/");
            if response_modifier.delete_rule(id) {
                Ok(json_response(json!({"success": true}).to_string()))
            } else {
                Ok(not_found_response())
            }
        }
        ("DELETE", "/api/modifiers") => {
            response_modifier.clear_all();
            Ok(json_response(json!({"success": true}).to_string()))
        }

        ("GET", "/api/rate-limits") => {
            let rules = rate_limiter.get_all_rules();
            let json = serde_json::to_string(&rules).unwrap();
            Ok(json_response(json))
        }
        ("POST", "/api/rate-limits") => match read_body_json::<CreateRateLimitRule>(req).await {
            Ok(rule) => {
                let id = rate_limiter.add_rule(rule);
                Ok(json_response(json!({"id": id}).to_string()))
            }
            Err(e) => Ok(error_response(&format!("Invalid request: {}", e))),
        },
        ("GET", p)
            if p.starts_with("/api/rate-limits/")
                && !p.ends_with("/toggle")
                && !p.ends_with("/reset") =>
        {
            let id = p.trim_start_matches("/api/rate-limits/");
            if let Some(rule) = rate_limiter.get_rule(id) {
                let json = serde_json::to_string(&rule).unwrap();
                Ok(json_response(json))
            } else {
                Ok(not_found_response())
            }
        }
        ("PUT", p) if p.starts_with("/api/rate-limits/") => {
            match read_body_json::<UpdateRateLimitRule>(req).await {
                Ok(rule) => {
                    if rate_limiter.update_rule(rule) {
                        Ok(json_response(json!({"success": true}).to_string()))
                    } else {
                        Ok(not_found_response())
                    }
                }
                Err(e) => Ok(error_response(&format!("Invalid request: {}", e))),
            }
        }
        ("POST", p) if p.starts_with("/api/rate-limits/") && p.ends_with("/toggle") => {
            let parts: Vec<&str> = p.split('/').collect();
            if parts.len() >= 4 {
                let id = parts[3];
                if rate_limiter.toggle_rule(id) {
                    Ok(json_response(json!({"success": true}).to_string()))
                } else {
                    Ok(not_found_response())
                }
            } else {
                Ok(not_found_response())
            }
        }
        ("POST", p) if p.starts_with("/api/rate-limits/") && p.ends_with("/reset") => {
            let parts: Vec<&str> = p.split('/').collect();
            if parts.len() >= 4 {
                let id = parts[3];
                rate_limiter.reset_bucket(id);
                Ok(json_response(json!({"success": true}).to_string()))
            } else {
                Ok(not_found_response())
            }
        }
        ("DELETE", p) if p.starts_with("/api/rate-limits/") => {
            let id = p.trim_start_matches("/api/rate-limits/");
            if rate_limiter.delete_rule(id) {
                Ok(json_response(json!({"success": true}).to_string()))
            } else {
                Ok(not_found_response())
            }
        }
        ("DELETE", "/api/rate-limits") => {
            rate_limiter.clear_all();
            Ok(json_response(json!({"success": true}).to_string()))
        }
        ("GET", "/api/rate-limits/stats") => {
            let stats = rate_limiter.get_bucket_stats();
            let json = serde_json::to_string(&stats).unwrap();
            Ok(json_response(json))
        }

        // Latency injection endpoints
        ("GET", "/api/latency-rules") => {
            let rules = latency_injector.get_all_rules();
            let json = serde_json::to_string(&rules).unwrap();
            Ok(json_response(json))
        }
        ("POST", "/api/latency-rules") => match read_body_json::<CreateLatencyRule>(req).await {
            Ok(rule) => {
                let id = latency_injector.add_rule(rule);
                Ok(json_response(json!({"id": id}).to_string()))
            }
            Err(e) => Ok(error_response(&format!("Invalid request: {}", e))),
        },
        ("GET", p)
            if p.starts_with("/api/latency-rules/")
                && !p.ends_with("/toggle")
                && !p.ends_with("/stats") =>
        {
            let id = p.trim_start_matches("/api/latency-rules/");
            if let Some(rule) = latency_injector.get_rule(id) {
                let json = serde_json::to_string(&rule).unwrap();
                Ok(json_response(json))
            } else {
                Ok(not_found_response())
            }
        }
        ("PUT", p) if p.starts_with("/api/latency-rules/") => {
            match read_body_json::<UpdateLatencyRule>(req).await {
                Ok(rule) => {
                    if latency_injector.update_rule(rule) {
                        Ok(json_response(json!({"success": true}).to_string()))
                    } else {
                        Ok(not_found_response())
                    }
                }
                Err(e) => Ok(error_response(&format!("Invalid request: {}", e))),
            }
        }
        ("POST", p) if p.starts_with("/api/latency-rules/") && p.ends_with("/toggle") => {
            let parts: Vec<&str> = p.split('/').collect();
            if parts.len() >= 4 {
                let id = parts[3];
                if latency_injector.toggle_rule(id) {
                    Ok(json_response(json!({"success": true}).to_string()))
                } else {
                    Ok(not_found_response())
                }
            } else {
                Ok(not_found_response())
            }
        }
        ("DELETE", p) if p.starts_with("/api/latency-rules/") => {
            let id = p.trim_start_matches("/api/latency-rules/");
            if latency_injector.delete_rule(id) {
                Ok(json_response(json!({"success": true}).to_string()))
            } else {
                Ok(not_found_response())
            }
        }
        ("DELETE", "/api/latency-rules") => {
            latency_injector.clear_all();
            Ok(json_response(json!({"success": true}).to_string()))
        }
        ("GET", "/api/latency-stats") => {
            let stats = latency_injector.get_stats();
            let json = serde_json::to_string(&stats).unwrap();
            Ok(json_response(json))
        }
        ("POST", "/api/latency-stats/reset") => {
            latency_injector.reset_stats();
            Ok(json_response(json!({"success": true}).to_string()))
        }

        _ => Ok(not_found_response()),
    }
}

async fn replay_request(
    replay_req: &crate::storage::ReplayRequest,
    upstream_url: &str,
) -> Result<crate::storage::RecordedRequest, String> {
    let start = std::time::Instant::now();
    let client = reqwest::Client::new();

    // Parse method
    let method = reqwest::Method::from_bytes(replay_req.method.as_bytes())
        .map_err(|e| format!("Invalid method: {}", e))?;

    // Build full URL
    let full_url = if replay_req.url.starts_with("http") {
        replay_req.url.clone()
    } else {
        format!("{}{}", upstream_url, replay_req.url)
    };

    // Build request
    let mut request = client.request(method, &full_url);

    // Add headers (skip host header)
    for (key, value) in &replay_req.headers {
        if key.to_lowercase() != "host" {
            request = request.header(key, value);
        }
    }

    // Add body if present
    if let Some(ref body) = replay_req.body {
        request = request.body(body.clone());
    }

    // Send request
    let response = request
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status().as_u16();
    let mut response_headers = std::collections::HashMap::new();

    for (name, value) in response.headers().iter() {
        if let Ok(value_str) = value.to_str() {
            response_headers.insert(name.to_string(), value_str.to_string());
        }
    }

    let response_body = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?
        .to_vec();

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(crate::storage::RecordedRequest {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        method: replay_req.method.clone(),
        url: replay_req.url.clone(),
        headers: replay_req.headers.clone(),
        body: replay_req.body.clone(),
        response: Some(crate::storage::RecordedResponse {
            status,
            headers: response_headers,
            body: Some(response_body),
        }),
        duration_ms: Some(duration_ms),
    })
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
