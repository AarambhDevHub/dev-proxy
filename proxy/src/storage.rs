use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedRequest {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub response: Option<RecordedResponse>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOptions {
    pub search: Option<String>,
    pub method: Option<String>,
    pub status: Option<u16>,
    pub min_duration: Option<u64>,
    pub max_duration: Option<u64>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct Storage {
    pub recordings: Arc<RwLock<HashMap<String, RecordedRequest>>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            recordings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn store_request(&self, mut request: RecordedRequest) -> String {
        let id = Uuid::new_v4().to_string();
        request.id = id.clone();

        let mut recordings = self.recordings.write();
        recordings.insert(id.clone(), request);

        id
    }

    pub fn update_response(&self, id: &str, response: RecordedResponse, duration_ms: u64) {
        let mut recordings = self.recordings.write();
        if let Some(request) = recordings.get_mut(id) {
            request.response = Some(response);
            request.duration_ms = Some(duration_ms);
        }
    }

    pub fn get_all(&self) -> Vec<RecordedRequest> {
        let recordings = self.recordings.read();
        let mut requests: Vec<_> = recordings.values().cloned().collect();
        requests.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        requests
    }

    pub fn get_filtered(&self, filters: &FilterOptions) -> Vec<RecordedRequest> {
        let recordings = self.recordings.read();
        let mut requests: Vec<_> = recordings
            .values()
            .filter(|req| self.matches_filters(req, filters))
            .cloned()
            .collect();

        requests.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        requests
    }

    fn matches_filters(&self, request: &RecordedRequest, filters: &FilterOptions) -> bool {
        // Search filter
        if let Some(ref search) = filters.search {
            let search_lower = search.to_lowercase();
            let url_match = request.url.to_lowercase().contains(&search_lower);
            let method_match = request.method.to_lowercase().contains(&search_lower);

            let body_match = if let Some(ref body) = request.body {
                if let Ok(body_str) = String::from_utf8(body.clone()) {
                    body_str.to_lowercase().contains(&search_lower)
                } else {
                    false
                }
            } else {
                false
            };

            let response_body_match = if let Some(ref response) = request.response {
                if let Some(ref body) = response.body {
                    if let Ok(body_str) = String::from_utf8(body.clone()) {
                        body_str.to_lowercase().contains(&search_lower)
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            if !(url_match || method_match || body_match || response_body_match) {
                return false;
            }
        }

        // Method filter
        if let Some(ref method) = filters.method {
            if !method.is_empty() && request.method != *method {
                return false;
            }
        }

        // Status code filter
        if let Some(status) = filters.status {
            match &request.response {
                Some(response) if response.status == status => {}
                _ => return false,
            }
        }

        // Duration filters
        if let Some(min_duration) = filters.min_duration {
            match request.duration_ms {
                Some(duration) if duration >= min_duration => {}
                _ => return false,
            }
        }

        if let Some(max_duration) = filters.max_duration {
            match request.duration_ms {
                Some(duration) if duration <= max_duration => {}
                _ => return false,
            }
        }

        // Time range filters
        if let Some(from_time) = filters.from_time {
            if request.timestamp < from_time {
                return false;
            }
        }

        if let Some(to_time) = filters.to_time {
            if request.timestamp > to_time {
                return false;
            }
        }

        true
    }

    pub fn get_by_id(&self, id: &str) -> Option<RecordedRequest> {
        let recordings = self.recordings.read();
        recordings.get(id).cloned()
    }

    pub fn clear(&self) {
        let mut recordings = self.recordings.write();
        recordings.clear();
    }

    pub fn get_stats(&self) -> RecordingStats {
        let recordings = self.recordings.read();

        let total = recordings.len();
        let mut success = 0;
        let mut redirects = 0;
        let mut client_errors = 0;
        let mut server_errors = 0;
        let mut total_duration = 0u64;
        let mut min_duration = u64::MAX;
        let mut max_duration = 0u64;

        for req in recordings.values() {
            if let Some(ref response) = req.response {
                let status = response.status;
                if status >= 200 && status < 300 {
                    success += 1;
                } else if status >= 300 && status < 400 {
                    redirects += 1;
                } else if status >= 400 && status < 500 {
                    client_errors += 1;
                } else if status >= 500 {
                    server_errors += 1;
                }
            }

            if let Some(duration) = req.duration_ms {
                total_duration += duration;
                min_duration = min_duration.min(duration);
                max_duration = max_duration.max(duration);
            }
        }

        let avg_duration = if total > 0 {
            total_duration / total as u64
        } else {
            0
        };

        RecordingStats {
            total,
            success,
            redirects,
            client_errors,
            server_errors,
            avg_duration_ms: avg_duration,
            min_duration_ms: if min_duration == u64::MAX {
                0
            } else {
                min_duration
            },
            max_duration_ms: max_duration,
        }
    }

    pub fn get_analytics(&self) -> Analytics {
        let recordings = self.recordings.read();

        let mut method_counts: HashMap<String, usize> = HashMap::new();
        let mut status_counts: HashMap<u16, usize> = HashMap::new();
        let mut endpoint_stats: HashMap<String, EndpointStats> = HashMap::new();
        let mut timeline: Vec<TimelinePoint> = Vec::new();

        // Group by time intervals (last hour, by minute)
        let now = Utc::now();
        let one_hour_ago = now - Duration::hours(1);

        for req in recordings.values() {
            // Method distribution
            *method_counts.entry(req.method.clone()).or_insert(0) += 1;

            // Status code distribution
            if let Some(ref response) = req.response {
                *status_counts.entry(response.status).or_insert(0) += 1;
            }

            // Endpoint statistics
            let endpoint = extract_endpoint(&req.url);
            let entry = endpoint_stats
                .entry(endpoint.clone())
                .or_insert(EndpointStats {
                    endpoint: endpoint.clone(),
                    count: 0,
                    avg_duration: 0,
                    errors: 0,
                    total_duration: 0,
                });

            entry.count += 1;
            if let Some(duration) = req.duration_ms {
                entry.total_duration += duration;
            }
            if let Some(ref response) = req.response {
                if response.status >= 400 {
                    entry.errors += 1;
                }
            }

            // Timeline (last hour)
            if req.timestamp >= one_hour_ago {
                timeline.push(TimelinePoint {
                    timestamp: req.timestamp,
                    duration_ms: req.duration_ms.unwrap_or(0),
                    status: req.response.as_ref().map(|r| r.status).unwrap_or(0),
                    method: req.method.clone(),
                });
            }
        }

        // Calculate average durations for endpoints
        for stats in endpoint_stats.values_mut() {
            if stats.count > 0 {
                stats.avg_duration = stats.total_duration / stats.count as u64;
            }
        }

        // Sort endpoints by count
        let mut top_endpoints: Vec<_> = endpoint_stats.into_values().collect();
        top_endpoints.sort_by(|a, b| b.count.cmp(&a.count));
        top_endpoints.truncate(10); // Top 10

        // Sort timeline
        timeline.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Analytics {
            method_distribution: method_counts,
            status_distribution: status_counts,
            top_endpoints,
            timeline,
        }
    }
}

fn extract_endpoint(url: &str) -> String {
    // Extract path without query parameters
    if let Some(path_end) = url.find('?') {
        url[..path_end].to_string()
    } else {
        url.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStats {
    pub total: usize,
    pub success: usize,
    pub redirects: usize,
    pub client_errors: usize,
    pub server_errors: usize,
    pub avg_duration_ms: u64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analytics {
    pub method_distribution: HashMap<String, usize>,
    pub status_distribution: HashMap<u16, usize>,
    pub top_endpoints: Vec<EndpointStats>,
    pub timeline: Vec<TimelinePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStats {
    pub endpoint: String,
    pub count: usize,
    pub avg_duration: u64,
    pub errors: usize,
    pub total_duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    pub status: u16,
    pub method: String,
}
