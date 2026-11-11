"use client";

import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import {
  api,
  getProjectId,
  type RecoveryAgentStats,
  type RecoveryStats,
  type Trace,
} from "@/lib/api";

export default function RecoveryPage() {
  const [projectId, setProjectId] = useState("");
  const [stats, setStats] = useState<RecoveryStats | null>(null);
  const [agents, setAgents] = useState<RecoveryAgentStats[]>([]);
  const [traces, setTraces] = useState<Trace[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const load = useCallback(async () => {
    const pid = getProjectId();
    if (!pid) return;
    setLoading(true);
    setError("");
    try {
      const [s, a, t] = await Promise.all([
        api.recoveryStats(pid),
        api.recoveryAgents(pid),
        api.recoveryTraces(pid),
      ]);
      setStats(s);
      setAgents(a);
      setTraces(t);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    setProjectId(getProjectId());
    load();
    const timer = setInterval(load, 30000);
    return () => clearInterval(timer);
  }, [load]);

  if (!projectId) {
    return (
      <main className="mx-auto max-w-5xl p-8">
        <h1 className="text-2xl font-semibold">恢复观测</h1>
        <p className="mt-4 text-gray-500">
          还没有项目，先到 <Link href="/" className="text-blue-600 hover:underline">首页</Link> 创建一个。
        </p>
      </main>
    );
  }

  const recoveryRate =
    stats && stats.recovered_traces + stats.failed_traces > 0
      ? ((stats.recovered_traces /
          (stats.recovered_traces + stats.failed_traces)) *
          100
      ).toFixed(1) + "%"
      : "-";

  return (
    <main className="mx-auto max-w-5xl p-8">
      <h1 className="text-2xl font-semibold">恢复观测</h1>

      {error && <p className="mt-4 text-red-600">{error}</p>}

      <div className="mt-6 grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-6">
        <StatCard label="失败 span" value={stats ? String(stats.error_spans) : "-"} tone="err" />
        <StatCard label="重试 span" value={stats ? String(stats.retry_spans) : "-"} />
        <StatCard label="恢复 trace" value={stats ? String(stats.recovered_traces) : "-"} tone="ok" />
        <StatCard label="失败 trace" value={stats ? String(stats.failed_traces) : "-"} tone="err" />
        <StatCard label="恢复率" value={recoveryRate} />
        <StatCard label="总 trace" value={stats ? String(stats.total_traces) : "-"} />
      </div>

      <h2 className="mt-8 text-lg">按 Agent 聚合</h2>
      <table className="mt-3 w-full border-collapse text-sm">
        <thead>
          <tr className="border-b text-left text-gray-500">
            <th className="py-2">Agent</th>
            <th className="py-2">失败 span</th>
            <th className="py-2">重试 span</th>
            <th className="py-2">失败 trace</th>
            <th className="py-2">总 trace</th>
          </tr>
        </thead>
        <tbody>
          {agents.map((a) => (
            <tr key={a.agent} className="border-b">
              <td className="py-2 font-mono">{a.agent}</td>
              <td className="py-2 text-red-600">{a.error_spans}</td>
              <td className="py-2">{a.retry_spans}</td>
              <td className="py-2 text-red-600">{a.failed_traces}</td>
              <td className="py-2">{a.total_traces}</td>
            </tr>
          ))}
          {agents.length === 0 && (
            <tr>
              <td colSpan={5} className="py-6 text-center text-gray-400">
                暂无数据{loading ? "（刷新中）" : ""}
              </td>
            </tr>
          )}
        </tbody>
      </table>

      <h2 className="mt-8 text-lg">含失败/重试的 Trace</h2>
      <table className="mt-3 w-full border-collapse text-sm">
        <thead>
          <tr className="border-b text-left text-gray-500">
            <th className="py-2">Trace ID</th>
            <th className="py-2">Agent</th>
            <th className="py-2">Status</th>
            <th className="py-2">Started</th>
            <th className="py-2">Duration</th>
          </tr>
        </thead>
        <tbody>
          {traces.map((t) => (
            <tr key={t.id} className="border-b">
              <td className="py-2">
                <Link href={`/traces/${t.id}`} className="text-blue-600 hover:underline">
                  {t.id.slice(0, 12)}…
                </Link>
              </td>
              <td className="py-2">{t.agent_id || "-"}</td>
              <td className="py-2">
                <span
                  className={`rounded px-2 py-0.5 text-xs text-white ${
                    t.status === "ok" ? "bg-green-500" : "bg-red-500"
                  }`}
                >
                  {t.status}
                </span>
              </td>
              <td className="py-2 text-gray-500">{t.started_at}</td>
              <td className="py-2">{t.duration_ms} ms</td>
            </tr>
          ))}
          {traces.length === 0 && (
            <tr>
              <td colSpan={5} className="py-6 text-center text-gray-400">
                暂无失败/重试 trace
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
