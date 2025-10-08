use serde::{Deserialize, Serialize};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub method: Option<String>,
    pub url_pattern: String,
    pub url_match_type: MatchType,
    pub response: MockResponse,
    pub delay_ms: Option<u64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// For creating new rules (without id and created_at)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMockRule {
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub method: Option<String>,
    pub url_pattern: String,
    pub url_match_type: MatchType,
    pub response: MockResponse,
    pub delay_ms: Option<u64>,
}

// For updating existing rules (with id but created_at is ignored)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMockRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub method: Option<String>,
    pub url_pattern: String,
    pub url_match_type: MatchType,
    pub response: MockResponse,
    pub delay_ms: Option<u64>,
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
pub struct MockResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Clone)]
pub struct MockManager {
    rules: Arc<RwLock<HashMap<String, MockRule>>>,
}

impl MockManager {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_rule(&self, create_rule: CreateMockRule) -> String {
        let id = Uuid::new_v4().to_string();

        let rule = MockRule {
            id: id.clone(),
            name: create_rule.name,
            enabled: create_rule.enabled,
            priority: create_rule.priority,
            method: create_rule.method,
            url_pattern: create_rule.url_pattern,
            url_match_type: create_rule.url_match_type,
            response: create_rule.response,
            delay_ms: create_rule.delay_ms,
            created_at: chrono::Utc::now(),
        };

        let mut rules = self.rules.write();
        rules.insert(id.clone(), rule);

        id
    }

    pub fn update_rule(&self, update_rule: UpdateMockRule) -> bool {
        let mut rules = self.rules.write();
        if let Some(existing) = rules.get(&update_rule.id) {
            let rule = MockRule {
                id: update_rule.id.clone(),
                name: update_rule.name,
                enabled: update_rule.enabled,
                priority: update_rule.priority,
                method: update_rule.method,
                url_pattern: update_rule.url_pattern,
                url_match_type: update_rule.url_match_type,
                response: update_rule.response,
                delay_ms: update_rule.delay_ms,
                created_at: existing.created_at, // Keep original creation time
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

    pub fn get_rule(&self, id: &str) -> Option<MockRule> {
        let rules = self.rules.read();
        rules.get(id).cloned()
    }

    pub fn get_all_rules(&self) -> Vec<MockRule> {
        let rules = self.rules.read();
        let mut all_rules: Vec<_> = rules.values().cloned().collect();
        all_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        all_rules
    }

    pub fn find_matching_rule(&self, method: &str, url: &str) -> Option<MockRule> {
        let rules = self.rules.read();
        let mut matching_rules: Vec<_> = rules
            .values()
            .filter(|rule| rule.enabled && self.matches(rule, method, url))
            .cloned()
            .collect();

        // Sort by priority (higher first)
        matching_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        matching_rules.into_iter().next()
    }

    fn matches(&self, rule: &MockRule, method: &str, url: &str) -> bool {
        // Check method
        if let Some(ref rule_method) = rule.method {
            if rule_method != method {
                return false;
            }
        }

        // Check URL pattern
        match rule.url_match_type {
            MatchType::Exact => url == rule.url_pattern,
            MatchType::Contains => url.contains(&rule.url_pattern),
            MatchType::StartsWith => url.starts_with(&rule.url_pattern),
            MatchType::EndsWith => url.ends_with(&rule.url_pattern),
            MatchType::Regex => {
                if let Ok(re) = Regex::new(&rule.url_pattern) {
                    re.is_match(url)
                } else {
                    false
                }
            }
        }
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
        rules.clear();
    }
}
