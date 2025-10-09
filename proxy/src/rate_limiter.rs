use parking_lot::RwLock;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub match_request: RateLimitMatch,
    pub limit: RateLimit,
    pub response: RateLimitResponse,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRateLimitRule {
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub match_request: RateLimitMatch,
    pub limit: RateLimit,
    pub response: RateLimitResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRateLimitRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub match_request: RateLimitMatch,
    pub limit: RateLimit,
    pub response: RateLimitResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitMatch {
    pub method: Option<String>,
    pub url_pattern: String,
    pub url_match_type: MatchType,
    pub key_type: KeyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchType {
    Exact,
    Contains,
    Regex,
    StartsWith,
    EndsWith,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyType {
    Global,
    #[serde(rename = "ipaddress")]
    IpAddress,
    Header {
        name: String,
    },
    Custom {
        pattern: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub max_requests: u32,
    pub window_seconds: u32,
    pub burst_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub delay_ms: Option<u64>,
}

#[derive(Debug, Clone)]
struct RateLimitBucket {
    requests: Vec<Instant>,
    burst_used: u32,
}

impl RateLimitBucket {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            burst_used: 0,
        }
    }

    fn is_allowed(&mut self, limit: &RateLimit, now: Instant) -> (bool, RateLimitInfo) {
        let window = Duration::from_secs(limit.window_seconds as u64);

        // Clean old requests outside the window
        self.requests
            .retain(|&req_time| now.duration_since(req_time) < window);

        let current_count = self.requests.len() as u32;
        let remaining = limit.max_requests.saturating_sub(current_count);

        // Check if burst is available
        let burst_size = limit.burst_size.unwrap_or(0);
        let can_use_burst = self.burst_used < burst_size && current_count >= limit.max_requests;

        let allowed = current_count < limit.max_requests || can_use_burst;

        if allowed {
            self.requests.push(now);
            if can_use_burst {
                self.burst_used += 1;
            }

            // Reset burst counter if we're back under the limit
            if current_count < limit.max_requests {
                self.burst_used = 0;
            }
        }

        // Calculate reset time
        let oldest = self.requests.first().copied();
        let reset_in_seconds = oldest
            .map(|t| {
                window
                    .as_secs()
                    .saturating_sub(now.duration_since(t).as_secs())
            })
            .unwrap_or(limit.window_seconds as u64);

        (
            allowed,
            RateLimitInfo {
                limit: limit.max_requests,
                remaining,
                reset_in_seconds,
                retry_after: if !allowed {
                    Some(reset_in_seconds)
                } else {
                    None
                },
            },
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_in_seconds: u64,
    pub retry_after: Option<u64>,
}

#[derive(Clone)]
pub struct RateLimiter {
    rules: Arc<RwLock<HashMap<String, RateLimitRule>>>,
    buckets: Arc<RwLock<HashMap<String, RateLimitBucket>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_rule(&self, create_rule: CreateRateLimitRule) -> String {
        let id = Uuid::new_v4().to_string();

        let rule = RateLimitRule {
            id: id.clone(),
            name: create_rule.name,
            enabled: create_rule.enabled,
            priority: create_rule.priority,
            match_request: create_rule.match_request,
            limit: create_rule.limit,
            response: create_rule.response,
            created_at: chrono::Utc::now(),
        };

        let mut rules = self.rules.write();
        rules.insert(id.clone(), rule);

        id
    }

    pub fn update_rule(&self, update_rule: UpdateRateLimitRule) -> bool {
        let mut rules = self.rules.write();
        if let Some(existing) = rules.get(&update_rule.id) {
            let rule = RateLimitRule {
                id: update_rule.id.clone(),
                name: update_rule.name,
                enabled: update_rule.enabled,
                priority: update_rule.priority,
                match_request: update_rule.match_request,
                limit: update_rule.limit,
                response: update_rule.response,
                created_at: existing.created_at,
            };
            rules.insert(update_rule.id, rule);
            true
        } else {
            false
        }
    }

    pub fn delete_rule(&self, id: &str) -> bool {
        let mut rules = self.rules.write();
        rules.remove(id).is_some()
    }

    pub fn get_rule(&self, id: &str) -> Option<RateLimitRule> {
        let rules = self.rules.read();
        rules.get(id).cloned()
    }

    pub fn get_all_rules(&self) -> Vec<RateLimitRule> {
        let rules = self.rules.read();
        let mut all_rules: Vec<_> = rules.values().cloned().collect();
        all_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        all_rules
    }

    pub fn toggle_rule(&self, id: &str) -> bool {
        let mut rules = self.rules.write();
        if let Some(rule) = rules.get_mut(id) {
            rule.enabled = !rule.enabled;
            true
        } else {
            false
        }
    }

    pub fn clear_all(&self) {
        let mut rules = self.rules.write();
        let mut buckets = self.buckets.write();
        rules.clear();
        buckets.clear();
    }

    pub fn check_rate_limit(
        &self,
        method: &str,
        url: &str,
        client_key: &str,
        headers: &HashMap<String, String>,
    ) -> Option<(RateLimitRule, RateLimitInfo)> {
        let rules = self.rules.read();
        let matching_rule = rules
            .values()
            .filter(|rule| rule.enabled && self.matches(rule, method, url))
            .max_by_key(|rule| rule.priority)?;

        let bucket_key = self.generate_bucket_key(matching_rule, client_key, headers);

        let mut buckets = self.buckets.write();
        let bucket = buckets
            .entry(bucket_key)
            .or_insert_with(RateLimitBucket::new);

        let now = Instant::now();
        let (allowed, info) = bucket.is_allowed(&matching_rule.limit, now);

        if !allowed {
            Some((matching_rule.clone(), info))
        } else {
            None
        }
    }

    fn matches(&self, rule: &RateLimitRule, method: &str, url: &str) -> bool {
        // Check method
        if let Some(ref rule_method) = rule.match_request.method {
            if rule_method != method {
                return false;
            }
        }

        // Check URL pattern
        match rule.match_request.url_match_type {
            MatchType::Exact => url == rule.match_request.url_pattern,
            MatchType::Contains => url.contains(&rule.match_request.url_pattern),
            MatchType::StartsWith => url.starts_with(&rule.match_request.url_pattern),
            MatchType::EndsWith => url.ends_with(&rule.match_request.url_pattern),
            MatchType::Regex => {
                if let Ok(re) = Regex::new(&rule.match_request.url_pattern) {
                    re.is_match(url)
                } else {
                    false
                }
            }
        }
    }

    fn generate_bucket_key(
        &self,
        rule: &RateLimitRule,
        client_key: &str,
        headers: &HashMap<String, String>,
    ) -> String {
        match &rule.match_request.key_type {
            KeyType::Global => format!("{}:global", rule.id),
            KeyType::IpAddress => format!("{}:ip:{}", rule.id, client_key),
            KeyType::Header { name } => {
                let header_value = headers.get(name).map(|v| v.as_str()).unwrap_or("unknown");
                format!("{}:header:{}:{}", rule.id, name, header_value)
            }
            KeyType::Custom { pattern } => format!("{}:custom:{}", rule.id, pattern),
        }
    }

    pub fn reset_bucket(&self, rule_id: &str) {
        let mut buckets = self.buckets.write();
        buckets.retain(|key, _| !key.starts_with(&format!("{}:", rule_id)));
    }

    pub fn get_bucket_stats(&self) -> BucketStats {
        let buckets = self.buckets.read();
        BucketStats {
            total_buckets: buckets.len(),
            active_limits: buckets.values().filter(|b| !b.requests.is_empty()).count(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BucketStats {
    pub total_buckets: usize,
    pub active_limits: usize,
}
