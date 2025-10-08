use crate::mock::MockManager;
use crate::storage::Storage;
use anyhow::Result;
use bytes::Bytes;
use hyper::{Request, Response, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn start_http_layer(
    port: u16,
    pingora_port: u16,
    storage: Storage,
    mock_manager: MockManager,
) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    println!("HTTP layer listening on {}", addr);

    let storage = Arc::new(storage);
    let mock_manager = Arc::new(mock_manager);

    loop {
        let (stream, _) = listener.accept().await?;
        let storage = storage.clone();
        let mock_manager = mock_manager.clone();

        tokio::spawn(async move {
            let io = hyper_util::rt::TokioIo::new(stream);

            let service = hyper::service::service_fn(move |req| {
                let storage = storage.clone();
                let mock_manager = mock_manager.clone();
                handle_request(req, pingora_port, storage, mock_manager)
            });

            if let Err(e) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await
            {
                eprintln!("HTTP layer connection error: {}", e);
            }
        });
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    pingora_port: u16,
    storage: Arc<Storage>,
    mock_manager: Arc<MockManager>,
) -> Result<Response<http_body_util::Full<Bytes>>, Infallible> {
    let method = req.method().as_str().to_string();
    let uri = req.uri().to_string();

    // Check for mock rule
    if let Some(mock_rule) = mock_manager.find_matching_rule(&method, &uri) {
        // Add delay if specified
        if let Some(delay_ms) = mock_rule.delay_ms {
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        }

        // Build mock response
        let status = StatusCode::from_u16(mock_rule.response.status).unwrap_or(StatusCode::OK);

        let mut response = Response::builder().status(status);

        // Add headers
        for (key, value) in &mock_rule.response.headers {
            response = response.header(key.as_str(), value.as_str());
        }

        // Add default content-type if not present
        if !mock_rule.response.headers.contains_key("content-type") {
            response = response.header("content-type", "application/json");
        }

        let body = Bytes::from(mock_rule.response.body.clone());

        println!(
            "{} {} - {} [MOCKED]",
            method, uri, mock_rule.response.status
        );

        // Record the mock
        let start = std::time::Instant::now();
        let id = uuid::Uuid::new_v4().to_string();

        storage.recordings.write().insert(
            id.clone(),
            crate::storage::RecordedRequest {
                id: id.clone(),
                timestamp: chrono::Utc::now(),
                method: method.clone(),
                url: uri.clone(),
                headers: std::collections::HashMap::new(),
                body: None,
                response: Some(crate::storage::RecordedResponse {
                    status: mock_rule.response.status,
                    headers: mock_rule.response.headers.clone(),
                    body: Some(body.to_vec()),
                }),
                duration_ms: Some(start.elapsed().as_millis() as u64),
            },
        );

        return Ok(response.body(http_body_util::Full::new(body)).unwrap());
    }

    // No mock - proxy to Pingora
    match proxy_to_pingora(req, pingora_port).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            eprintln!("Proxy error: {}", e);
            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(http_body_util::Full::new(Bytes::from("Bad Gateway")))
                .unwrap())
        }
    }
}

async fn proxy_to_pingora(
    req: Request<hyper::body::Incoming>,
    pingora_port: u16,
) -> Result<Response<http_body_util::Full<Bytes>>> {
    use http_body_util::BodyExt;

    let (parts, body) = req.into_parts();

    // Read request body
    let body_bytes = body.collect().await?.to_bytes();

    // Build new request to Pingora
    let uri = format!("http://127.0.0.1:{}{}", pingora_port, parts.uri);

    let client = reqwest::Client::new();
    let mut request = client.request(parts.method.clone(), &uri);

    // Copy headers (skip host header, let reqwest set it)
    for (name, value) in parts.headers.iter() {
        if name.as_str().to_lowercase() != "host" {
            if let Ok(value_str) = value.to_str() {
                request = request.header(name.as_str(), value_str);
            }
        }
    }

    // Add body if present
    if !body_bytes.is_empty() {
        request = request.body(body_bytes.to_vec());
    }

    // Send request
    let response = request.send().await?;

    // Build response
    let status = response.status();
    let mut builder = Response::builder().status(status);

    // Copy response headers
    for (name, value) in response.headers().iter() {
        builder = builder.header(name, value);
    }

    // Get response body
    let response_bytes = response.bytes().await?;

    Ok(builder.body(http_body_util::Full::new(response_bytes))?)
}
