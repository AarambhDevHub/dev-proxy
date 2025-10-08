use crate::config::ProxyConfig;
use crate::mock::MockManager;
use crate::recorder::Recorder;
use crate::storage::Storage;
use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use pingora::prelude::*;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_proxy::{ProxyHttp, Session, http_proxy_service};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct DevProxy {
    upstream_url: String,
    recorder: Arc<Recorder>,
    mock_manager: Arc<MockManager>,
}

pub struct ProxyCtx {
    request_id: Option<(String, Instant)>,
    request_body: Vec<u8>,
    response_body: Vec<u8>,
}

#[async_trait]
impl ProxyHttp for DevProxy {
    type CTX = ProxyCtx;

    fn new_ctx(&self) -> Self::CTX {
        ProxyCtx {
            request_id: None,
            request_body: Vec::new(),
            response_body: Vec::new(),
        }
    }

    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<bool, Box<pingora::Error>> {
        // Extract method and URI first (before any mutable operations)
        let method = session.req_header().method.as_str().to_string();
        let uri = session.req_header().uri.to_string();

        // Record request
        ctx.request_id = self
            .recorder
            .record_request(&method, &uri, session.req_header(), None);

        // Check for mock rule
        if let Some(mock_rule) = self.mock_manager.find_matching_rule(&method, &uri) {
            // Clone everything we need from mock_rule
            let status = mock_rule.response.status;
            let headers = mock_rule.response.headers.clone();
            let body = mock_rule.response.body.clone();
            let delay = mock_rule.delay_ms;

            // Add delay if specified
            if let Some(delay_ms) = delay {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }

            // Build response
            let status_code = http::StatusCode::from_u16(status).unwrap_or(http::StatusCode::OK);

            let mut header =
                pingora_http::ResponseHeader::build(status_code, None).map_err(|e| {
                    pingora::Error::because(pingora::ErrorType::HTTPStatus(500), "build header", e)
                })?;

            // Add headers
            for (k, v) in &headers {
                if let (Ok(name), Ok(value)) = (
                    http::header::HeaderName::try_from(k.as_str()),
                    http::HeaderValue::try_from(v.as_str()),
                ) {
                    let _ = header.insert_header(name, value);
                }
            }

            if !headers.contains_key("content-type") {
                let _ = header.insert_header(http::header::CONTENT_TYPE, "application/json");
            }

            // Now we can mutate session
            let _ = session.write_response_header(Box::new(header), false).await;

            if !body.is_empty() {
                let _ = session
                    .write_response_body(Some(Bytes::from(body.clone())), true)
                    .await;
            } else {
                let _ = session.write_response_body(None, true).await;
            }

            // Record it
            if let Some((ref id, start)) = ctx.request_id {
                let dur = start.elapsed().as_millis() as u64;
                self.recorder.storage.update_response(
                    id,
                    crate::storage::RecordedResponse {
                        status,
                        headers,
                        body: Some(body.into_bytes()),
                    },
                    dur,
                );
            }

            println!("{} {} - {} [MOCKED]", method, uri, status);

            // Return true to skip upstream
            return Ok(true);
        }

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>, Box<pingora::Error>> {
        let url = url::Url::parse(&self.upstream_url).map_err(|e| {
            pingora::Error::explain(
                pingora::ErrorType::ConnectError,
                format!("Invalid URL: {}", e),
            )
        })?;

        let host = url
            .host_str()
            .ok_or_else(|| pingora::Error::explain(pingora::ErrorType::ConnectError, "No host"))?;

        let port = url
            .port()
            .unwrap_or(if url.scheme() == "https" { 443 } else { 80 });
        let use_tls = url.scheme() == "https";

        Ok(Box::new(HttpPeer::new(
            (host, port),
            use_tls,
            host.to_string(),
        )))
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut pingora_http::ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> Result<(), Box<pingora::Error>> {
        let status = upstream_response.status.as_u16();

        if let Some((ref id, start_time)) = ctx.request_id {
            let duration_ms = start_time.elapsed().as_millis() as u64;

            let mut header_map = std::collections::HashMap::new();
            for (name, value) in upstream_response.headers.iter() {
                if let Ok(value_str) = value.to_str() {
                    header_map.insert(name.to_string(), value_str.to_string());
                }
            }

            self.recorder.storage.update_response(
                id,
                crate::storage::RecordedResponse {
                    status,
                    headers: header_map,
                    body: None,
                },
                duration_ms,
            );
        }

        Ok(())
    }

    fn response_body_filter(
        &self,
        _session: &mut Session,
        body: &mut Option<Bytes>,
        _end_of_stream: bool,
        ctx: &mut Self::CTX,
    ) -> Result<Option<Duration>, Box<pingora::Error>> {
        if let Some(data) = body {
            ctx.response_body.extend_from_slice(data);
        }
        Ok(None)
    }

    async fn request_body_filter(
        &self,
        _session: &mut Session,
        body: &mut Option<Bytes>,
        _end_of_stream: bool,
        ctx: &mut Self::CTX,
    ) -> Result<(), Box<pingora::Error>> {
        if let Some(data) = body {
            ctx.request_body.extend_from_slice(data);
        }
        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        let method = session.req_header().method.as_str();
        let uri = session.req_header().uri.to_string();
        let status = session
            .response_written()
            .map(|h| h.status.as_u16())
            .unwrap_or(0);

        if let Some((ref id, _)) = ctx.request_id {
            if let Some(recording) = self.recorder.storage.get_by_id(id) {
                let mut updated = recording.clone();

                if !ctx.request_body.is_empty() {
                    updated.body = Some(ctx.request_body.clone());
                }

                if let Some(ref mut response) = updated.response {
                    if !ctx.response_body.is_empty() {
                        response.body = Some(ctx.response_body.clone());
                    }
                }

                self.recorder
                    .storage
                    .recordings
                    .write()
                    .insert(id.clone(), updated);
            }
        }

        if status > 0 {
            println!("{} {} - {}", method, uri, status);
        }
    }
}

pub fn start_proxy_server(
    config: ProxyConfig,
    storage: Storage,
    mock_manager: MockManager,
) -> Result<()> {
    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    let recorder = Arc::new(Recorder::new(storage, config.recording_enabled));

    let proxy_service = DevProxy {
        upstream_url: config.upstream_url.clone(),
        recorder,
        mock_manager: Arc::new(mock_manager),
    };

    let mut proxy_service_http = http_proxy_service(&server.configuration, proxy_service);
    proxy_service_http.add_tcp(&format!("0.0.0.0:{}", config.proxy_port));

    server.add_service(proxy_service_http);
    server.run_forever();
}
