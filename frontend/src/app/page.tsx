"use client";

import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { api, getKey, getProjectId, setSession, type Trace } from "@/lib/api";

export default function Home() {
  const [key, setKey] = useState("");
  const [projectId, setProjectId] = useState("");
  const [traces, setTraces] = useState<Trace[]>([]);
  const [newName, setNewName] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setKey(getKey());
    setProjectId(getProjectId());
  }, []);

  const load = useCallback(async () => {
    if (!key || !projectId) return;
    setLoading(true);
    setError("");
    try {
      setTraces(await api.listTraces(projectId));
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [key, projectId]);

  useEffect(() => {
    if (key && projectId) load();
  }, [load]);

  const create = async () => {
    setLoading(true);
    setError("");
    try {
      const p = await api.createProject(newName || "my-project");
      setSession(p.api_key, p.id);
      setKey(p.api_key);
      setProjectId(p.id);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="mx-auto max-w-5xl p-8">
      <h1 className="text-2xl font-semibold">TracePilot</h1>
      <p className="text-gray-500">Agent 运行时可观测平台</p>

      {error && <p className="mt-4 text-red-600">{error}</p>}

      {!projectId ? (
        <div className="mt-8">
          <h2 className="text-lg">创建项目</h2>
          <div className="mt-4 flex gap-2">
            <input
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="项目名"
              className="border px-3 py-2"
            />
            <button onClick={create} disabled={loading} className="bg-blue-600 px-4 py-2 text-white disabled:opacity-50">
              {loading ? "创建中" : "创建"}
            </button>
          </div>
          <p className="mt-2 text-sm text-gray-500">创建后将生成 API Key，用于 SDK 接入 trace。</p>
        </div>
      ) : (
        <div className="mt-8">
          <div className="flex items-center justify-between">
            <h2 className="text-lg">
              Traces <span className="text-gray-400">({traces.length})</span>
            </h2>
            <button onClick={load} disabled={loading} className="px-3 py-1 text-sm border">
              {loading ? "刷新中" : "刷新"}
            </button>
          </div>
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
                    暂无 trace，跑一个示例 Agent 试试
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}
    </main>
  );
}
