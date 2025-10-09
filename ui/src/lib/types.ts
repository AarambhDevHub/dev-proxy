export interface RecordedRequest {
  id: string;
  timestamp: string;
  method: string;
  url: string;
  headers: Record<string, string>;
  body?: number[];
  response?: RecordedResponse;
  duration_ms?: number;
}

export interface RecordedResponse {
  status: number;
  headers: Record<string, string>;
  body?: number[];
}

export interface RecordingStats {
  total: number;
  success: number;
  redirects: number;
  client_errors: number;
  server_errors: number;
  avg_duration_ms: number;
  min_duration_ms: number;
  max_duration_ms: number;
}

export interface Analytics {
  method_distribution: Record<string, number>;
  status_distribution: Record<string, number>;
  top_endpoints: EndpointStats[];
  timeline: TimelinePoint[];
}

export interface EndpointStats {
  endpoint: string;
  count: number;
  avg_duration: number;
  errors: number;
  total_duration: number;
}

export interface TimelinePoint {
  timestamp: string;
  duration_ms: number;
  status: number;
  method: string;
}

export interface FilterOptions {
  search: string;
  method: string;
  status?: number;
  minDuration?: number;
  maxDuration?: number;
}

export interface MockRule {
  id: string;
  name: string;
  enabled: boolean;
  priority: number;
  method?: string;
  url_pattern: string;
  url_match_type: "exact" | "contains" | "regex" | "startswith" | "endswith";
  response: MockResponse;
  delay_ms?: number;
  created_at: string;
}

// For creating new rules (no id or created_at)
export interface CreateMockRule {
  name: string;
  enabled: boolean;
  priority: number;
  method?: string;
  url_pattern: string;
  url_match_type: "exact" | "contains" | "regex" | "startswith" | "endswith";
  response: MockResponse;
  delay_ms?: number;
}

export interface MockResponse {
  status: number;
  headers: Record<string, string>;
  body: string;
}

export interface ModifierRule {
  id: string;
  name: string;
  enabled: boolean;
  priority: number;
  match_request: RequestMatch;
  modifications: Modification[];
  created_at: string;
}

export interface CreateModifierRule {
  name: string;
  enabled: boolean;
  priority: number;
  match_request: RequestMatch;
  modifications: Modification[];
}

export interface UpdateModifierRule {
  id: string;
  name: string;
  enabled: boolean;
  priority: number;
  match_request: RequestMatch;
  modifications: Modification[];
}

export interface RequestMatch {
  method?: string;
  url_pattern: string;
  url_match_type: "exact" | "contains" | "regex" | "startswith" | "endswith";
  status_codes?: number[];
}

export type Modification =
  | {
      type: "replace_body";
      pattern: string;
      replacement: string;
      use_regex: boolean;
    }
  | {
      type: "add_header";
      name: string;
      value: string;
    }
  | {
      type: "remove_header";
      name: string;
    }
  | {
      type: "change_status";
      status: number;
    }
  | {
      type: "inject_delay";
      delay_ms: number;
    }
  | {
      type: "modify_json";
      path: string;
      value: any;
    };

export interface RateLimitRule {
  id: string;
  name: string;
  enabled: boolean;
  priority: number;
  match_request: RateLimitMatch;
  limit: RateLimit;
  response: RateLimitResponse;
  created_at: string;
}

export interface CreateRateLimitRule {
  name: string;
  enabled: boolean;
  priority: number;
  match_request: RateLimitMatch;
  limit: RateLimit;
  response: RateLimitResponse;
}

export interface RateLimitMatch {
  method?: string;
  url_pattern: string;
  url_match_type: "exact" | "contains" | "regex" | "startswith" | "endswith";
  key_type: KeyType;
}

export type KeyType =
  | "global"
  | "ipaddress"
  | { header: { name: string } }
  | { custom: { pattern: string } };

export interface RateLimit {
  max_requests: number;
  window_seconds: number;
  burst_size?: number;
}

export interface RateLimitResponse {
  status: number;
  headers: Record<string, string>;
  body: string;
  delay_ms?: number;
}

export interface BucketStats {
  total_buckets: number;
  active_limits: number;
}

export interface LatencyRule {
  id: string;
  name: string;
  enabled: boolean;
  priority: number;
  match_request: LatencyMatch;
  delay: DelayConfig;
  created_at: string;
}

export interface CreateLatencyRule {
  name: string;
  enabled: boolean;
  priority: number;
  match_request: LatencyMatch;
  delay: DelayConfig;
}

export interface LatencyMatch {
  method?: string;
  url_pattern: string;
  url_match_type: "exact" | "contains" | "regex" | "startswith" | "endswith";
  apply_to: "request" | "response" | "both";
}

export type DelayConfig =
  | { type: "fixed"; delay_ms: number }
  | { type: "random"; min_ms: number; max_ms: number }
  | { type: "normal"; mean_ms: number; std_dev_ms: number }
  | {
      type: "spike";
      base_delay_ms: number;
      spike_delay_ms: number;
      spike_probability: number;
    };

export interface LatencyStats {
  total_injections: number;
  total_delay_ms: number;
  min_delay_ms: number;
  max_delay_ms: number;
  avg_delay_ms: number;
  by_rule: Record<string, RuleStats>;
}

export interface RuleStats {
  rule_id: string;
  rule_name: string;
  hits: number;
  total_delay_ms: number;
  avg_delay_ms: number;
}

export function formatBody(body?: number[]): string {
  if (!body || body.length === 0) return "";
  try {
    const text = new TextDecoder().decode(new Uint8Array(body));
    try {
      return JSON.stringify(JSON.parse(text), null, 2);
    } catch {
      return text;
    }
  } catch {
    return "[Binary Data]";
  }
}

export function formatDuration(ms?: number): string {
  if (ms === undefined) return "-";
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

export function getStatusColor(status?: number): string {
  if (!status) return "gray";
  if (status >= 200 && status < 300) return "green";
  if (status >= 300 && status < 400) return "blue";
  if (status >= 400 && status < 500) return "yellow";
  return "red";
}

export function getMethodColor(method: string): string {
  switch (method) {
    case "GET":
      return "blue";
    case "POST":
      return "green";
    case "PUT":
      return "yellow";
    case "DELETE":
      return "red";
    case "PATCH":
      return "purple";
    default:
      return "gray";
  }
}
