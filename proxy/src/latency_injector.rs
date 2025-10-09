use parking_lot::RwLock;
use rand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub match_request: LatencyMatch,
    pub delay: DelayConfig,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLatencyRule {
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub match_request: LatencyMatch,
    pub delay: DelayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLatencyRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub match_request: LatencyMatch,
    pub delay: DelayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMatch {
    pub method: Option<String>,
    pub url_pattern: String,
    pub url_match_type: MatchType,
    pub apply_to: ApplyTo,
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
pub enum ApplyTo {
    Request,  // Delay before proxying to upstream
    Response, // Delay before sending response to client
    Both,     // Delay both request and response
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DelayConfig {
    Fixed {
        delay_ms: u64,
    },
    Random {
        min_ms: u64,
        max_ms: u64,
    },
    Normal {
        mean_ms: u64,
        std_dev_ms: u64,
    },
    Spike {
        base_delay_ms: u64,
        spike_delay_ms: u64,
        spike_probability: f64, // 0.0 to 1.0
    },
}

impl DelayConfig {
    pub fn calculate_delay(&self) -> u64 {
        match self {
            DelayConfig::Fixed { delay_ms } => *delay_ms,

            DelayConfig::Random { min_ms, max_ms } => {
                let mut rng = rand::thread_rng();
                rng.gen_range(*min_ms..=*max_ms)
            }

            DelayConfig::Normal {
                mean_ms,
                std_dev_ms,
            } => {
                use rand_distr::{Distribution, Normal};
                let mut rng = rand::thread_rng();

                if let Ok(normal) = Normal::new(*mean_ms as f64, *std_dev_ms as f64) {
                    let value = normal.sample(&mut rng);
                    value.max(0.0) as u64
                } else {
                    *mean_ms
                }
            }

            DelayConfig::Spike {
                base_delay_ms,
                spike_delay_ms,
                spike_probability,
            } => {
                let mut rng = rand::thread_rng();
                let random: f64 = rng.gen_range(0.0..1.0);

                if random < *spike_probability {
                    *spike_delay_ms
                } else {
                    *base_delay_ms
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct LatencyInjector {
    rules: Arc<RwLock<HashMap<String, LatencyRule>>>,
    stats: Arc<RwLock<LatencyStats>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub total_injections: u64,
    pub total_delay_ms: u64,
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub avg_delay_ms: u64,
    pub by_rule: HashMap<String, RuleStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleStats {
    pub rule_id: String,
    pub rule_name: String,
    pub hits: u64,
    pub total_delay_ms: u64,
    pub avg_delay_ms: u64,
}

impl Default for LatencyStats {
    fn default() -> Self {
        Self {
            total_injections: 0,
            total_delay_ms: 0,
            min_delay_ms: u64::MAX,
            max_delay_ms: 0,
            avg_delay_ms: 0,
            by_rule: HashMap::new(),
        }
    }
}

impl LatencyInjector {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(LatencyStats::default())),
        }
    }

    pub fn add_rule(&self, create_rule: CreateLatencyRule) -> String {
        let id = Uuid::new_v4().to_string();

        let rule = LatencyRule {
            id: id.clone(),
            name: create_rule.name,
            enabled: create_rule.enabled,
            priority: create_rule.priority,
            match_request: create_rule.match_request,
            delay: create_rule.delay,
            created_at: chrono::Utc::now(),
        };

        let mut rules = self.rules.write();
        rules.insert(id.clone(), rule);

        id
    }

    pub fn update_rule(&self, update_rule: UpdateLatencyRule) -> bool {
        let mut rules = self.rules.write();
        if let Some(existing) = rules.get(&update_rule.id) {
            let rule = LatencyRule {
                id: update_rule.id.clone(),
                name: update_rule.name,
                enabled: update_rule.enabled,
                priority: update_rule.priority,
                match_request: update_rule.match_request,
                delay: update_rule.delay,
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

    pub fn get_rule(&self, id: &str) -> Option<LatencyRule> {
        let rules = self.rules.read();
        rules.get(id).cloned()
    }

    pub fn get_all_rules(&self) -> Vec<LatencyRule> {
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
        let mut stats = self.stats.write();
        rules.clear();
        *stats = LatencyStats::default();
    }

    pub fn find_matching_rule(
        &self,
        method: &str,
        url: &str,
        apply_to: ApplyTo,
    ) -> Option<LatencyRule> {
        let rules = self.rules.read();

        rules
            .values()
            .filter(|rule| {
                rule.enabled
                    && self.matches(rule, method, url)
                    && self.applies_to(&rule.match_request.apply_to, &apply_to)
            })
            .max_by_key(|rule| rule.priority)
            .cloned()
    }

    fn matches(&self, rule: &LatencyRule, method: &str, url: &str) -> bool {
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

    fn applies_to(&self, rule_apply: &ApplyTo, current_apply: &ApplyTo) -> bool {
        match rule_apply {
            ApplyTo::Both => true,
            ApplyTo::Request => matches!(current_apply, ApplyTo::Request),
            ApplyTo::Response => matches!(current_apply, ApplyTo::Response),
        }
    }

    pub async fn apply_delay(&self, method: &str, url: &str, apply_to: ApplyTo) -> Option<u64> {
        if let Some(rule) = self.find_matching_rule(method, url, apply_to) {
            let delay_ms = rule.delay.calculate_delay();

            if delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                self.record_delay(&rule.id, &rule.name, delay_ms);
                return Some(delay_ms);
            }
        }
        None
    }

    fn record_delay(&self, rule_id: &str, rule_name: &str, delay_ms: u64) {
        let mut stats = self.stats.write();

        // Update global stats
        stats.total_injections += 1;
        stats.total_delay_ms += delay_ms;
        stats.min_delay_ms = stats.min_delay_ms.min(delay_ms);
        stats.max_delay_ms = stats.max_delay_ms.max(delay_ms);
        stats.avg_delay_ms = stats.total_delay_ms / stats.total_injections;

        // Update per-rule stats
        let rule_stats = stats
            .by_rule
            .entry(rule_id.to_string())
            .or_insert(RuleStats {
                rule_id: rule_id.to_string(),
                rule_name: rule_name.to_string(),
                hits: 0,
                total_delay_ms: 0,
                avg_delay_ms: 0,
            });

        rule_stats.hits += 1;
        rule_stats.total_delay_ms += delay_ms;
        rule_stats.avg_delay_ms = rule_stats.total_delay_ms / rule_stats.hits;
    }

    pub fn get_stats(&self) -> LatencyStats {
        let stats = self.stats.read();
        stats.clone()
    }

    pub fn reset_stats(&self) {
        let mut stats = self.stats.write();
        *stats = LatencyStats::default();
    }
}
