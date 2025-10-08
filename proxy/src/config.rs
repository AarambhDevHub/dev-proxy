use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub proxy_port: u16,
    pub ui_port: u16,
    pub upstream_url: String,
    pub recording_enabled: bool,
}
