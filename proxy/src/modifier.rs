use parking_lot::RwLock;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub match_request: RequestMatch,
    pub modifications: Vec<Modification>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModifierRule {
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub match_request: RequestMatch,
    pub modifications: Vec<Modification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModifierRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub match_request: RequestMatch,
    pub modifications: Vec<Modification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMatch {
    pub method: Option<String>,
    pub url_pattern: String,
    pub url_match_type: MatchType,
    pub status_codes: Option<Vec<u16>>,
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
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Modification {
    #[serde(rename = "replace_body")]
    ReplaceBody {
        pattern: String,
        replacement: String,
        use_regex: bool,
    },

    #[serde(rename = "add_header")]
    AddHeader { name: String, value: String },

    #[serde(rename = "remove_header")]
    RemoveHeader { name: String },

    #[serde(rename = "change_status")]
    ChangeStatus { status: u16 },

    #[serde(rename = "inject_delay")]
    InjectDelay { delay_ms: u64 },

    #[serde(rename = "modify_json")]
    ModifyJson {
        path: String,
        value: serde_json::Value,
    },
}

#[derive(Clone)]
pub struct ResponseModifier {
    rules: Arc<RwLock<HashMap<String, ModifierRule>>>,
}

impl ResponseModifier {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_rule(&self, create_rule: CreateModifierRule) -> String {
        let id = Uuid::new_v4().to_string();

        let rule = ModifierRule {
            id: id.clone(),
            name: create_rule.name,
            enabled: create_rule.enabled,
            priority: create_rule.priority,
            match_request: create_rule.match_request,
            modifications: create_rule.modifications,
            created_at: chrono::Utc::now(),
        };

        let mut rules = self.rules.write();
        rules.insert(id.clone(), rule);

        id
    }

    pub fn update_rule(&self, update_rule: UpdateModifierRule) -> bool {
        let mut rules = self.rules.write();
        if let Some(existing) = rules.get(&update_rule.id) {
            let rule = ModifierRule {
                id: update_rule.id.clone(),
                name: update_rule.name,
                enabled: update_rule.enabled,
                priority: update_rule.priority,
                match_request: update_rule.match_request,
                modifications: update_rule.modifications,
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

    pub fn get_rule(&self, id: &str) -> Option<ModifierRule> {
        let rules = self.rules.read();
        rules.get(id).cloned()
    }

    pub fn get_all_rules(&self) -> Vec<ModifierRule> {
        let rules = self.rules.read();
        let mut all_rules: Vec<_> = rules.values().cloned().collect();
        all_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        all_rules
    }

    pub fn find_matching_rules(
        &self,
        method: &str,
        url: &str,
        status: Option<u16>,
    ) -> Vec<ModifierRule> {
        let rules = self.rules.read();
        let mut matching_rules: Vec<_> = rules
            .values()
            .filter(|rule| rule.enabled && self.matches(rule, method, url, status))
            .cloned()
            .collect();

        matching_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        matching_rules
    }

    fn matches(&self, rule: &ModifierRule, method: &str, url: &str, status: Option<u16>) -> bool {
        // Check method
        if let Some(ref rule_method) = rule.match_request.method {
            if rule_method != method {
                return false;
            }
        }

        // Check status codes
        if let Some(ref status_codes) = rule.match_request.status_codes {
            if let Some(status_code) = status {
                if !status_codes.contains(&status_code) {
                    return false;
                }
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

    pub async fn apply_modifications(
        &self,
        method: &str,
        url: &str,
        status: u16,
        headers: &mut std::collections::HashMap<String, String>,
        body: &mut Vec<u8>,
    ) -> u16 {
        let rules = self.find_matching_rules(method, url, Some(status));
        let mut final_status = status;

        for rule in rules {
            for modification in &rule.modifications {
                match modification {
                    Modification::ReplaceBody {
                        pattern,
                        replacement,
                        use_regex,
                    } => {
                        if let Ok(body_str) = String::from_utf8(body.clone()) {
                            let modified = if *use_regex {
                                if let Ok(re) = Regex::new(pattern) {
                                    re.replace_all(&body_str, replacement.as_str()).to_string()
                                } else {
                                    body_str
                                }
                            } else {
                                body_str.replace(pattern, replacement)
                            };
                            *body = modified.into_bytes();
                        }
                    }

                    Modification::AddHeader { name, value } => {
                        headers.insert(name.clone(), value.clone());
                    }

                    Modification::RemoveHeader { name } => {
                        headers.remove(name);
                    }

                    Modification::ChangeStatus { status: new_status } => {
                        final_status = *new_status;
                    }

                    Modification::InjectDelay { delay_ms } => {
                        tokio::time::sleep(std::time::Duration::from_millis(*delay_ms)).await;
                    }

                    Modification::ModifyJson { path, value } => {
                        if let Ok(body_str) = String::from_utf8(body.clone()) {
                            if let Ok(mut json) =
                                serde_json::from_str::<serde_json::Value>(&body_str)
                            {
                                let path_parts: Vec<&str> = path.split('.').collect();

                                if path_parts.is_empty() {
                                    continue;
                                }

                                // Helper function to set nested value
                                fn set_nested_value(
                                    current: &mut serde_json::Value,
                                    keys: &[&str],
                                    value: &serde_json::Value,
                                ) -> bool {
                                    if keys.is_empty() {
                                        return false;
                                    }

                                    if keys.len() == 1 {
                                        if let Some(obj) = current.as_object_mut() {
                                            obj.insert(keys[0].to_string(), value.clone());
                                            return true;
                                        }
                                        return false;
                                    }

                                    // Navigate deeper
                                    if let Some(next) = current.get_mut(keys[0]) {
                                        set_nested_value(next, &keys[1..], value)
                                    } else {
                                        false
                                    }
                                }

                                set_nested_value(&mut json, &path_parts, value);

                                if let Ok(modified_json) = serde_json::to_string(&json) {
                                    *body = modified_json.into_bytes();
                                }
                            }
                        }
                    }
                }
            }
        }

        final_status
    }
}
