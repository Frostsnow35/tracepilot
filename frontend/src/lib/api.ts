const BASE = process.env.NEXT_PUBLIC_API_BASE || "http://localhost:8000";

const K_KEY = "tp_key";
const K_PROJECT = "tp_project";

export function getKey(): string {
  if (typeof window === "undefined") return "";
  return localStorage.getItem(K_KEY) || "";
}

export function getProjectId(): string {
  if (typeof window === "undefined") return "";
  return localStorage.getItem(K_PROJECT) || "";
}

export function setSession(key: string, projectId: string) {
  localStorage.setItem(K_KEY, key);
  localStorage.setItem(K_PROJECT, projectId);
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...((options.headers as Record<string, string>) || {}),
  };
  const key = getKey();
  if (key) headers["X-TracePilot-Key"] = key;

  const r = await fetch(`${BASE}${path}`, { ...options, headers });
  if (!r.ok) throw new Error(`${r.status} ${r.statusText}`);
  return r.json() as Promise<T>;
}

export const api = {
  createProject: (name: string) =>
    request<{ id: string; name: string; api_key: string }>("/api/iam/projects", {
      method: "POST",
      body: JSON.stringify({ name }),
    }),
  listTraces: (projectId: string, limit = 50) =>
    request<Trace[]>(`/api/query/traces?project_id=${projectId}&limit=${limit}`),
  getTrace: (traceId: string) => request<TraceDetail>(`/api/query/traces/${traceId}`),

  monitorStats: (projectId: string) =>
    request<MonitorStats>(`/api/monitor/stats?project_id=${projectId}`),
  monitorAgents: (projectId: string) =>
    request<AgentStats[]>(`/api/monitor/agents?project_id=${projectId}`),
  analyticsSummary: (projectId: string) =>
    request<AnalyticsSummary>(`/api/analytics/summary?project_id=${projectId}`),
  analyticsLatency: (projectId: string) =>
    request<LatencyPoint[]>(`/api/analytics/latency?project_id=${projectId}`),
  analyticsCost: (projectId: string) =>
    request<CostBreakdown[]>(`/api/analytics/cost?project_id=${projectId}`),
};

/// WebSocket 基址：http(s):// 换成 ws(s)://（浏览器 WS 无法带自定义 header，
/// api_key 走 query 由后端校验）。
export function wsBase(): string {
  return BASE.replace(/^http/, "ws");
}

export interface Trace {
  id: string;
  project_id: string;
  agent_id: string | null;
  status: string;
  started_at: string;
  duration_ms: number;
}

export interface Span {
  id: string;
  trace_id: string;
  parent_span_id: string | null;
  name: string;
  kind: string;
  status: string;
  started_at: string;
  duration_ms: number;
  attributes: Record<string, unknown>;
}

export interface TraceDetail {
  trace: Trace;
  spans: Span[];
}

export interface MonitorStats {
  total: number;
  success: number;
  error: number;
  avg_ms: number | null;
  p50_ms: number | null;
  p95_ms: number | null;
}

export interface AgentStats {
  agent: string;
  total: number;
  success: number;
  avg_ms: number | null;
  p95_ms: number | null;
}

export interface SpanKindStat {
  kind: string;
  total: number;
  success: number;
  avg_ms: number | null;
}

export interface AnalyticsSummary {
  total: number;
  success: number;
  success_rate: number;
  avg_ms: number | null;
  p50_ms: number | null;
  p95_ms: number | null;
  p99_ms: number | null;
  span_kinds: SpanKindStat[];
}

export interface LatencyPoint {
  bucket: string;
  total: number;
  avg_ms: number | null;
  p95_ms: number | null;
  success: number;
}

export interface CostBreakdown {
  kind: string;
  total: number;
  total_ms: number;
  avg_ms: number | null;
}

export interface TraceEvent {
  trace_id: string;
  project_id: string;
  agent_id: string | null;
  status: string;
  started_at: string;
  duration_ms: number;
}
