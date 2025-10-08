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
