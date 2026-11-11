"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { api, getKey, getProjectId, wsBase, type AgentStats, type MonitorStats, type TraceEvent } from "@/lib/api";

export default function MonitorPage() {
  const [projectId, setProjectId] = useState("");
  const [stats, setStats] = useState<MonitorStats | null>(null);
  const [agents, setAgents] = useState<AgentStats[]>([]);
  const [events, setEvents] = useState<TraceEvent[]>([]);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const eventsRef = useRef<TraceEvent[]>([]);

  useEffect(() => {
    const pid = getProjectId();
    setProjectId(pid);
    if (!pid || !getKey()) return;

    const load = async () => {
      setLoading(true);
      setError("");
      try {
        const [s, a] = await Promise.all([api.monitorStats(pid), api.monitorAgents(pid)]);
        setStats(s);
        setAgents(a);
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    };

    load();
    const timer = setInterval(load, 5000);

    // WebSocket 实时流：api_key 走 query（浏览器 WS 无法设 header）。
    const ws = new WebSocket(`${wsBase()}/api/monitor/stream?project_id=${pid}&api_key=${getKey()}`);
    wsRef.current = ws;
    ws.onopen = () => setConnected(true);
    ws.onclose = () => setConnected(false);
    ws.onerror = () => setConnected(false);
    ws.onmessage = (msg) => {
      try {
        const ev: TraceEvent = JSON.parse(msg.data);
        eventsRef.current = [ev, ...eventsRef.current].slice(0, 20);
        setEvents(eventsRef.current);
      } catch {
        // 非 JSON 消息，忽略。
      }
    };

    return () => {
      clearInterval(timer);
      ws.close();
    };
  }, []);

  // 新 trace 到达时刷新统计卡片（事件驱动的补充，主刷新靠 5s 轮询）。
  useEffect(() => {
    if (events.length > 0 && projectId) {
      api.monitorStats(projectId).then(setStats).catch(() => {});
      api.monitorAgents(projectId).then(setAgents).catch(() => {});
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [events.length]);

  const fmtMs = (v: number | null) => (v == null ? "-" : `${Math.round(v)} ms`);

  if (!projectId) {
    return (
      <main className="mx-auto max-w-5xl p-8">
        <h1 className="text-2xl font-semibold">实时监控</h1>
        <p className="mt-4 text-gray-500">
          还没有项目，先到 <Link href="/" className="text-blue-600 hover:underline">首页</Link> 创建一个。
        </p>
      </main>
    );
  }

  return (
    <main className="mx-auto max-w-5xl p-8">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">实时监控</h1>
        <span
          className={`rounded px-2 py-0.5 text-xs ${
            connected ? "bg-green-500 text-white" : "bg-gray-200 text-gray-600"
          }`}
        >
          {connected ? "实时推送已连接" : "未连接"}
        </span>
      </div>

      {error && <p className="mt-4 text-red-600">{error}</p>}

      <div className="mt-6 grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-6">
        <StatCard label="总 trace" value={stats ? String(stats.total) : "-"} />
        <StatCard label="成功" value={stats ? String(stats.success) : "-"} tone="ok" />
        <StatCard label="失败" value={stats ? String(stats.error) : "-"} tone="err" />
        <StatCard label="平均延迟" value={stats ? fmtMs(stats.avg_ms) : "-"} />
        <StatCard label="p50" value={stats ? fmtMs(stats.p50_ms) : "-"} />
        <StatCard label="p95" value={stats ? fmtMs(stats.p95_ms) : "-"} />
      </div>

      <h2 className="mt-8 text-lg">按 Agent 聚合</h2>
      <table className="mt-3 w-full border-collapse text-sm">
        <thead>
          <tr className="border-b text-left text-gray-500">
            <th className="py-2">Agent</th>
            <th className="py-2">Total</th>
            <th className="py-2">Success</th>
            <th className="py-2">Avg</th>
            <th className="py-2">p95</th>
          </tr>
        </thead>
        <tbody>
          {agents.map((a) => (
            <tr key={a.agent} className="border-b">
              <td className="py-2 font-mono">{a.agent}</td>
              <td className="py-2">{a.total}</td>
              <td className="py-2">{a.success}</td>
              <td className="py-2">{fmtMs(a.avg_ms)}</td>
              <td className="py-2">{fmtMs(a.p95_ms)}</td>
            </tr>
          ))}
          {agents.length === 0 && (
            <tr>
              <td colSpan={5} className="py-6 text-center text-gray-400">
                暂无数据
              </td>
            </tr>
          )}
        </tbody>
      </table>

      <h2 className="mt-8 text-lg">实时流（最近 20 条）</h2>
      <table className="mt-3 w-full border-collapse text-sm">
        <thead>
          <tr className="border-b text-left text-gray-500">
            <th className="py-2">Trace ID</th>
            <th className="py-2">Agent</th>
            <th className="py-2">Status</th>
            <th className="py-2">Duration</th>
          </tr>
        </thead>
        <tbody>
          {events.map((ev, i) => (
            <tr key={ev.trace_id} className={`border-b ${i === 0 ? "bg-green-50" : ""}`}>
              <td className="py-2">
                <Link href={`/traces/${ev.trace_id}`} className="text-blue-600 hover:underline">
                  {ev.trace_id.slice(0, 12)}…
                </Link>
              </td>
              <td className="py-2">{ev.agent_id || "-"}</td>
              <td className="py-2">
                <span
                  className={`rounded px-2 py-0.5 text-xs text-white ${
                    ev.status === "ok" ? "bg-green-500" : "bg-red-500"
                  }`}
                >
                  {ev.status}
                </span>
              </td>
              <td className="py-2">{ev.duration_ms} ms</td>
            </tr>
          ))}
          {events.length === 0 && (
            <tr>
              <td colSpan={4} className="py-6 text-center text-gray-400">
                等待新 trace…{loading ? "（刷新中）" : ""}
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </main>
  );
}

function StatCard({
  label,
  value,
  tone,
}: {
  label: string;
  value: string;
  tone?: "ok" | "err";
}) {
  const color =
    tone === "ok" ? "text-green-600" : tone === "err" ? "text-red-600" : "text-gray-900";
  return (
    <div className="border bg-white p-3">
      <div className="text-xs text-gray-500">{label}</div>
      <div className={`mt-1 text-xl font-semibold ${color}`}>{value}</div>
    </div>
  );
}
