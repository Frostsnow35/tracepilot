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
};

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
