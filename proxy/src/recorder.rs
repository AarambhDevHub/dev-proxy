use crate::storage::{RecordedRequest, Storage};
use bytes::Bytes;
use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;

pub struct Recorder {
    pub storage: Storage,
    enabled: bool,
}

impl Recorder {
    pub fn new(storage: Storage, enabled: bool) -> Self {
        Self { storage, enabled }
    }

    pub fn record_request(
        &self,
        method: &str,
        url: &str,
        headers: &pingora_http::RequestHeader,
        body: Option<&Bytes>,
    ) -> Option<(String, Instant)> {
        if !self.enabled {
            return None;
        }

        let mut header_map = HashMap::new();
        for (name, value) in headers.headers.iter() {
            if let Ok(value_str) = value.to_str() {
                header_map.insert(name.to_string(), value_str.to_string());
            }
        }

        let recorded_request = RecordedRequest {
            id: String::new(),
            timestamp: Utc::now(),
            method: method.to_string(),
            url: url.to_string(),
            headers: header_map,
            body: body.map(|b| b.to_vec()),
            response: None,
            duration_ms: None,
        };

        let id = self.storage.store_request(recorded_request);
        Some((id, Instant::now()))
    }
}
