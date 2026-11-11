"use client";

import { useEffect, useState } from "react";
import { api, getKey, getProjectId, setSession } from "@/lib/api";

export default function Settings() {
  const [key, setKey] = useState("");
  const [projectId, setProjectId] = useState("");
  const [newName, setNewName] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    setKey(getKey());
    setProjectId(getProjectId());
  }, []);

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

  const copyKey = async () => {
    try {
      await navigator.clipboard.writeText(key);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      setError("复制失败，请手动选择文本复制");
    }
  };

  return (
    <main className="mx-auto max-w-3xl p-8">
      <h1 className="text-2xl font-semibold">设置</h1>
      <p className="text-gray-500">项目与 API Key 管理（FR-7）</p>

      {error && <p className="mt-4 text-red-600">{error}</p>}

      <section className="mt-6">
        <h2 className="text-lg">当前项目</h2>
        {projectId ? (
          <div className="mt-3 space-y-2 rounded border bg-white p-4 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-500">项目 ID</span>
              <code className="text-gray-900">{projectId}</code>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500">API Key</span>
              <code className="flex items-center gap-2">
                <span className="text-gray-900">{key.slice(0, 12)}…</span>
                <button
                  onClick={copyKey}
                  className="text-xs text-blue-600 hover:underline"
                >
                  {copied ? "已复制" : "复制"}
                </button>
              </code>
            </div>
            <p className="text-xs text-gray-400">
              API Key 用于 SDK 接入 trace，请妥善保管。泄露后可在下方重建项目获取新 Key。
            </p>
          </div>
        ) : (
          <p className="mt-2 text-sm text-gray-500">尚无项目，请在下方创建。</p>
        )}
      </section>

      <section className="mt-8">
        <h2 className="text-lg">创建新项目</h2>
        <p className="mt-1 text-sm text-gray-500">
          创建后生成新的 API Key 并切换到该项目。
        </p>
        <div className="mt-3 flex gap-2">
          <input
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            placeholder="项目名"
            className="border px-3 py-2"
          />
          <button
            onClick={create}
            disabled={loading}
            className="bg-blue-600 px-4 py-2 text-white disabled:opacity-50"
          >
            {loading ? "创建中" : "创建"}
          </button>
        </div>
      </section>
    </main>
  );
}
