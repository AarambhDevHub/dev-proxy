use crate::mock::MockManager;
use crate::modifier::ResponseModifier;
use crate::rate_limiter::RateLimiter;

use crate::latency_injector::{ApplyTo, LatencyInjector};

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
    response_modifier: ResponseModifier,
    rate_limiter: RateLimiter,
    latency_injector: LatencyInjector,
) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    println!("HTTP layer listening on {}", addr);

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
            let io = hyper_util::rt::TokioIo::new(stream);

            let service = hyper::service::service_fn(move |req| {
                let storage = storage.clone();
                let mock_manager = mock_manager.clone();
                let response_modifier = response_modifier.clone();
                let rate_limiter = rate_limiter.clone();
                let latency_injector = latency_injector.clone();
                handle_request(
                    req,
                    pingora_port,
                    storage,
                    mock_manager,
                    response_modifier,
                    rate_limiter,
                    latency_injector,
                )
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
    response_modifier: Arc<ResponseModifier>,
    rate_limiter: Arc<RateLimiter>,
    latency_injector: Arc<LatencyInjector>,
) -> Result<Response<http_body_util::Full<Bytes>>, Infallible> {
    let method = req.method().as_str().to_string();
    let uri = req.uri().to_string();

    let client_key = extract_client_ip(&req);

    // Extract headers
    let mut headers_map = std::collections::HashMap::new();
    for (name, value) in req.headers().iter() {
        if let Ok(value_str) = value.to_str() {
            headers_map.insert(name.to_string(), value_str.to_string());
        }
    }

    if let Some(delay_ms) = latency_injector
        .apply_delay(&method, &uri, ApplyTo::Request)
        .await
    {
        println!("{} {} [REQUEST LATENCY: {}ms]", method, uri, delay_ms);
    }

    // Check rate limit FIRST
    if let Some((rule, info)) =
        rate_limiter.check_rate_limit(&method, &uri, &client_key, &headers_map)
    {
        // Add delay if specified
        if let Some(delay_ms) = rule.response.delay_ms {
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        }

        let status =
            StatusCode::from_u16(rule.response.status).unwrap_or(StatusCode::TOO_MANY_REQUESTS);
        let mut response = Response::builder().status(status);

        // Add rate limit headers
        response = response
            .header("X-RateLimit-Limit", info.limit.to_string())
            .header("X-RateLimit-Remaining", info.remaining.to_string())
            .header("X-RateLimit-Reset", info.reset_in_seconds.to_string());

        if let Some(retry_after) = info.retry_after {
            response = response.header("Retry-After", retry_after.to_string());
        }

        // Add custom headers
        for (key, value) in &rule.response.headers {
            response = response.header(key.as_str(), value.as_str());
        }

        // Add default content-type if not present
        if !rule.response.headers.contains_key("content-type") {
            response = response.header("content-type", "application/json");
        }

        let body = Bytes::from(rule.response.body.clone());

        println!(
            "{} {} - {} [RATE LIMITED]",
            method, uri, rule.response.status
        );

        return Ok(response.body(http_body_util::Full::new(body)).unwrap());
    }

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

        if let Some(delay_ms) = latency_injector
            .apply_delay(&method, &uri, ApplyTo::Response)
            .await
        {
            println!("{} {} [RESPONSE LATENCY: {}ms]", method, uri, delay_ms);
        }

        return Ok(response.body(http_body_util::Full::new(body)).unwrap());
    }

    // No mock - proxy to Pingora
    match proxy_to_pingora(req, pingora_port, &method, &uri, response_modifier).await {
        Ok(resp) => {
            // Apply response latency after proxying
            if let Some(delay_ms) = latency_injector
                .apply_delay(&method, &uri, ApplyTo::Response)
                .await
            {
                println!("{} {} [RESPONSE LATENCY: {}ms]", method, uri, delay_ms);
            }
            Ok(resp)
        }
        Err(e) => {
            eprintln!("Proxy error: {}", e);
            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(http_body_util::Full::new(Bytes::from("Bad Gateway")))
                .unwrap())
        }
    }
}

/// Extract client IP address from request headers or connection
/// Follows the X-Forwarded-For chain for proxy environments
fn extract_client_ip(req: &Request<hyper::body::Incoming>) -> String {
    // Try X-Forwarded-For header first (standard for proxies)
    if let Some(xff) = req.headers().get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            // X-Forwarded-For format: "client, proxy1, proxy2"
            // Take the leftmost (original client) IP
            if let Some(client_ip) = xff_str.split(',').next() {
                let trimmed = client_ip.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }

    // Try X-Real-IP header (nginx standard)
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            let trimmed = ip_str.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    // Try CF-Connecting-IP (Cloudflare)
    if let Some(cf_ip) = req.headers().get("cf-connecting-ip") {
        if let Ok(ip_str) = cf_ip.to_str() {
            let trimmed = ip_str.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    // Try True-Client-IP (Akamai, Cloudflare)
    if let Some(true_ip) = req.headers().get("true-client-ip") {
        if let Ok(ip_str) = true_ip.to_str() {
            let trimmed = ip_str.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    // Fallback to connection remote address (not available in Hyper without additional setup)
    // For development/testing, return localhost
    "127.0.0.1".to_string()
}

async fn proxy_to_pingora(
    req: Request<hyper::body::Incoming>,
    pingora_port: u16,
    method: &str,
    url: &str,
    response_modifier: Arc<ResponseModifier>,
) -> Result<Response<http_body_util::Full<Bytes>>> {
    use http_body_util::BodyExt;

    let (parts, body) = req.into_parts();
    let body_bytes = body.collect().await?.to_bytes();

    let uri = format!("http://127.0.0.1:{}{}", pingora_port, parts.uri);

    let client = reqwest::Client::new();
    let mut request = client.request(parts.method.clone(), &uri);

    for (name, value) in parts.headers.iter() {
        if name.as_str().to_lowercase() != "host" {
            if let Ok(value_str) = value.to_str() {
                request = request.header(name.as_str(), value_str);
            }
        }
    }

    if !body_bytes.is_empty() {
        request = request.body(body_bytes.to_vec());
    }

    let response = request.send().await?;

    let status = response.status();
    let status_u16 = status.as_u16();

    let mut header_map = std::collections::HashMap::new();
    for (name, value) in response.headers().iter() {
        if let Ok(value_str) = value.to_str() {
            header_map.insert(name.to_string(), value_str.to_string());
        }
    }

    let response_bytes = response.bytes().await?;
    let mut response_vec = response_bytes.to_vec();

    // Apply response modifications BEFORE building response
    let modified_status = response_modifier
        .apply_modifications(method, url, status_u16, &mut header_map, &mut response_vec)
        .await;

    // **FIX: Update Content-Length after modification**
    header_map.insert("content-length".to_string(), response_vec.len().to_string());

    // Remove transfer-encoding if present (conflicts with content-length)
    header_map.remove("transfer-encoding");

    // Build response with modified values
    let mut builder = Response::builder().status(modified_status);

    for (name, value) in header_map.iter() {
        if let (Ok(header_name), Ok(header_value)) = (
            http::header::HeaderName::from_bytes(name.as_bytes()),
            http::HeaderValue::from_str(value),
        ) {
            builder = builder.header(header_name, header_value);
        }
    }

    Ok(builder.body(http_body_util::Full::new(Bytes::from(response_vec)))?)
}
