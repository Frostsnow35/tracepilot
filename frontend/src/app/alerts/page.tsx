"use client";

import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import {
  api,
  getProjectId,
  type AlertEvent,
  type AlertRule,
} from "@/lib/api";

const METRICS = [
  { value: "error_rate", label: "错误率 (0-1)" },
  { value: "p95_ms", label: "p95 延迟 (ms)" },
  { value: "avg_ms", label: "平均延迟 (ms)" },
];

const OPERATORS = [
  { value: ">", label: "> 超过" },
  { value: "<", label: "< 低于" },
];

export default function AlertsPage() {
  const [projectId, setProjectId] = useState("");
  const [rules, setRules] = useState<AlertRule[]>([]);
  const [events, setEvents] = useState<AlertEvent[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [form, setForm] = useState({ metric: "error_rate", operator: ">", threshold: "0.5" });

  const load = useCallback(async () => {
    const pid = getProjectId();
    if (!pid) return;
    setLoading(true);
    setError("");
    try {
      const [r, e] = await Promise.all([api.alertRules(pid), api.alertEvents(pid)]);
      setRules(r);
      setEvents(e);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    setProjectId(getProjectId());
    load();
    const timer = setInterval(load, 15000);
    return () => clearInterval(timer);
  }, [load]);

  const createRule = async () => {
    const pid = getProjectId();
    if (!pid) return;
    setError("");
    try {
      const threshold = parseFloat(form.threshold);
      if (isNaN(threshold)) throw new Error("阈值必须是数字");
      await api.createAlertRule({
        project_id: pid,
        metric: form.metric,
        operator: form.operator,
        threshold,
      });
      setForm({ metric: "error_rate", operator: ">", threshold: "0.5" });
      load();
    } catch (e) {
      setError(String(e));
    }
  };

  const removeRule = async (ruleId: string) => {
    const pid = getProjectId();
    if (!pid) return;
    setError("");
    try {
      await api.deleteAlertRule(pid, ruleId);
      load();
    } catch (e) {
      setError(String(e));
    }
  };

  const toggleRule = async (rule: AlertRule) => {
    const pid = getProjectId();
    if (!pid) return;
    setError("");
    try {
      await api.updateAlertRule(pid, rule.id, !rule.enabled);
      load();
    } catch (e) {
      setError(String(e));
    }
  };

  if (!projectId) {
    return (
      <main className="mx-auto max-w-5xl p-8">
        <h1 className="text-2xl font-semibold">告警</h1>
        <p className="mt-4 text-gray-500">
          还没有项目，先到 <Link href="/" className="text-blue-600 hover:underline">首页</Link> 创建一个。
        </p>
      </main>
    );
  }

  return (
    <main className="mx-auto max-w-5xl p-8">
      <h1 className="text-2xl font-semibold">告警</h1>

      {error && <p className="mt-4 text-red-600">{error}</p>}

      <div className="mt-6 border bg-white p-4">
        <h2 className="text-lg">新建规则</h2>
        <p className="mt-1 text-sm text-gray-500">
          指标每 30 秒评估一次，触发后 10 分钟内不重复触发。
        </p>
        <div className="mt-3 flex flex-wrap items-end gap-3">
          <div>
            <label className="text-xs text-gray-500">指标</label>
            <select
              value={form.metric}
              onChange={(e) => setForm({ ...form, metric: e.target.value })}
              className="border px-2 py-1"
            >
              {METRICS.map((m) => (
                <option key={m.value} value={m.value}>
                  {m.label}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label className="text-xs text-gray-500">条件</label>
            <select
              value={form.operator}
              onChange={(e) => setForm({ ...form, operator: e.target.value })}
              className="border px-2 py-1"
            >
              {OPERATORS.map((o) => (
                <option key={o.value} value={o.value}>
                  {o.label}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label className="text-xs text-gray-500">阈值</label>
            <input
              value={form.threshold}
              onChange={(e) => setForm({ ...form, threshold: e.target.value })}
              className="border px-2 py-1"
            />
          </div>
          <button
            onClick={createRule}
            className="bg-blue-600 px-4 py-1.5 text-white hover:opacity-90"
          >
            创建
          </button>
        </div>
      </div>

      <h2 className="mt-8 text-lg">规则 ({rules.length})</h2>
      <table className="mt-3 w-full border-collapse text-sm">
        <thead>
          <tr className="border-b text-left text-gray-500">
            <th className="py-2">指标</th>
            <th className="py-2">条件</th>
            <th className="py-2">阈值</th>
            <th className="py-2">启用</th>
            <th className="py-2">上次触发</th>
            <th className="py-2">操作</th>
          </tr>
        </thead>
        <tbody>
          {rules.map((r) => (
            <tr key={r.id} className="border-b">
              <td className="py-2 font-mono">{r.metric}</td>
              <td className="py-2">
                <span className="font-mono">{r.operator}</span>
              </td>
              <td className="py-2">{r.threshold}</td>
              <td className="py-2">
                <span
                  className={`rounded px-2 py-0.5 text-xs text-white ${
                    r.enabled ? "bg-green-500" : "bg-gray-400"
                  }`}
                >
                  {r.enabled ? "启用" : "停用"}
                </span>
              </td>
              <td className="py-2 text-gray-500">
                {r.last_triggered_at ? r.last_triggered_at.slice(0, 19).replace("T", " ") : "-"}
              </td>
              <td className="py-2">
                <button
                  onClick={() => toggleRule(r)}
                  className={`mr-3 text-xs ${
                    r.enabled ? "text-gray-500" : "text-green-600"
                  } hover:underline`}
                >
                  {r.enabled ? "停用" : "启用"}
                </button>
                <button
                  onClick={() => removeRule(r.id)}
                  className="text-red-600 hover:underline"
                >
                  删除
                </button>
              </td>
            </tr>
          ))}
          {rules.length === 0 && (
            <tr>
              <td colSpan={6} className="py-6 text-center text-gray-400">
                暂无规则{loading ? "（刷新中）" : ""}
              </td>
            </tr>
          )}
        </tbody>
      </table>

      <h2 className="mt-8 text-lg">触发事件 ({events.length})</h2>
      <table className="mt-3 w-full border-collapse text-sm">
        <thead>
          <tr className="border-b text-left text-gray-500">
            <th className="py-2">时间</th>
            <th className="py-2">指标</th>
            <th className="py-2">实际值</th>
            <th className="py-2">阈值</th>
          </tr>
        </thead>
        <tbody>
          {events.map((e) => (
            <tr key={e.id} className="border-b">
              <td className="py-2 text-gray-500">
                {e.triggered_at.slice(0, 19).replace("T", " ")}
              </td>
              <td className="py-2 font-mono">{e.metric}</td>
              <td className="py-2 text-red-600">{e.value}</td>
              <td className="py-2">
                {e.operator} {e.threshold}
              </td>
            </tr>
          ))}
          {events.length === 0 && (
            <tr>
              <td colSpan={4} className="py-6 text-center text-gray-400">
                暂无触发事件
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </main>
  );
}
