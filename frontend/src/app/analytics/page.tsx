"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import ReactECharts from "echarts-for-react";
import {
  api,
  getProjectId,
  type AnalyticsSummary,
  type CostBreakdown,
  type LatencyPoint,
} from "@/lib/api";

export default function AnalyticsPage() {
  const [projectId, setProjectId] = useState("");
  const [summary, setSummary] = useState<AnalyticsSummary | null>(null);
  const [latency, setLatency] = useState<LatencyPoint[]>([]);
  const [cost, setCost] = useState<CostBreakdown[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const pid = getProjectId();
    setProjectId(pid);
    if (!pid) return;

    const load = async () => {
      setLoading(true);
      setError("");
      try {
        const [s, l, c] = await Promise.all([
          api.analyticsSummary(pid),
          api.analyticsLatency(pid),
          api.analyticsCost(pid),
        ]);
        setSummary(s);
        setLatency(l);
        setCost(c);
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    };

    load();
    const timer = setInterval(load, 30000);
    return () => clearInterval(timer);
  }, []);

  if (!projectId) {
    return (
      <main className="mx-auto max-w-5xl p-8">
        <h1 className="text-2xl font-semibold">分析看板</h1>
        <p className="mt-4 text-gray-500">
          还没有项目，先到 <Link href="/" className="text-blue-600 hover:underline">首页</Link> 创建一个。
        </p>
      </main>
    );
  }

  const fmtMs = (v: number | null) => (v == null ? "-" : `${Math.round(v)} ms`);
  const fmtPct = (v: number) => `${(v * 100).toFixed(1)}%`;

  return (
    <main className="mx-auto max-w-5xl p-8">
      <h1 className="text-2xl font-semibold">分析看板</h1>

      {error && <p className="mt-4 text-red-600">{error}</p>}

      {summary && (
        <div className="mt-6 grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-6">
          <StatCard label="总 trace" value={String(summary.total)} />
          <StatCard label="成功率" value={fmtPct(summary.success_rate)} tone="ok" />
          <StatCard label="平均" value={fmtMs(summary.avg_ms)} />
          <StatCard label="p50" value={fmtMs(summary.p50_ms)} />
          <StatCard label="p95" value={fmtMs(summary.p95_ms)} />
          <StatCard label="p99" value={fmtMs(summary.p99_ms)} />
        </div>
      )}

      <h2 className="mt-8 text-lg">延迟趋势（按小时）</h2>
      <div className="mt-3 border bg-white p-2">
        {latency.length > 0 ? (
          <ReactECharts
            style={{ height: 320 }}
            option={{
              tooltip: { trigger: "axis" },
              legend: { data: ["avg", "p95"] },
              grid: { left: 56, right: 24, top: 40, bottom: 40 },
              xAxis: {
                type: "category",
                data: latency.map((p) => p.bucket.slice(11, 16)),
                axisLabel: { fontSize: 11 },
              },
              yAxis: { type: "value", name: "ms", splitLine: { lineStyle: { type: "dashed" } } },
              series: [
                { name: "avg", type: "line", smooth: true, data: latency.map((p) => p.avg_ms) },
                { name: "p95", type: "line", smooth: true, data: latency.map((p) => p.p95_ms) },
              ],
            }}
          />
        ) : (
          <div className="py-16 text-center text-gray-400">暂无数据{loading ? "（刷新中）" : ""}</div>
        )}
      </div>

      <h2 className="mt-8 text-lg">成本拆解（按 span 类型耗时）</h2>
      <div className="mt-3 grid gap-4 md:grid-cols-2">
        <div className="border bg-white p-2">
          {cost.length > 0 ? (
            <ReactECharts
              style={{ height: 300 }}
              option={{
                tooltip: { trigger: "item" },
                legend: { bottom: 0, textStyle: { fontSize: 11 } },
                series: [
                  {
                    type: "pie",
                    radius: ["40%", "65%"],
                    data: cost.map((c) => ({ name: c.kind, value: c.total_ms })),
                    label: { formatter: "{b}: {d}%", fontSize: 11 },
                  },
                ],
              }}
            />
          ) : (
            <div className="py-16 text-center text-gray-400">暂无数据</div>
          )}
        </div>
        <table className="mt-3 w-full border-collapse text-sm">
          <thead>
            <tr className="border-b text-left text-gray-500">
              <th className="py-2">Kind</th>
              <th className="py-2">Total</th>
              <th className="py-2">总耗时</th>
              <th className="py-2">Avg</th>
            </tr>
          </thead>
          <tbody>
            {cost.map((c) => (
              <tr key={c.kind} className="border-b">
                <td className="py-2 font-mono">{c.kind}</td>
                <td className="py-2">{c.total}</td>
                <td className="py-2">{c.total_ms} ms</td>
                <td className="py-2">{fmtMs(c.avg_ms)}</td>
              </tr>
            ))}
            {cost.length === 0 && (
              <tr>
                <td colSpan={4} className="py-6 text-center text-gray-400">
                  暂无数据
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {summary && summary.span_kinds.length > 0 && (
        <>
          <h2 className="mt-8 text-lg">Span 类型分布</h2>
          <table className="mt-3 w-full border-collapse text-sm">
            <thead>
              <tr className="border-b text-left text-gray-500">
                <th className="py-2">Kind</th>
                <th className="py-2">Total</th>
                <th className="py-2">Success</th>
                <th className="py-2">Avg</th>
              </tr>
            </thead>
            <tbody>
              {summary.span_kinds.map((k) => (
                <tr key={k.kind} className="border-b">
                  <td className="py-2 font-mono">{k.kind}</td>
                  <td className="py-2">{k.total}</td>
                  <td className="py-2">{k.success}</td>
                  <td className="py-2">{fmtMs(k.avg_ms)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </>
      )}
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
  tone?: "ok";
}) {
  const color = tone === "ok" ? "text-green-600" : "text-gray-900";
  return (
    <div className="border bg-white p-3">
      <div className="text-xs text-gray-500">{label}</div>
      <div className={`mt-1 text-xl font-semibold ${color}`}>{value}</div>
    </div>
  );
}
