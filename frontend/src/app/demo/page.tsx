"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { api, setSession } from "@/lib/api";

const FEATURES = [
  {
    icon: "📡",
    title: "实时监控",
    desc: "统计概览、按 Agent 聚合、WebSocket 实时流，Agent 运行状态一目了然",
  },
  {
    icon: "📊",
    title: "分析洞察",
    desc: "成功率、延迟趋势、成本拆解，多维度分析 Agent 运行质量",
  },
  {
    icon: "🔄",
    title: "恢复观测",
    desc: "重试 span、恢复 trace、按 Agent 失败统计，故障恢复过程可视化",
  },
  {
    icon: "🔔",
    title: "阈值告警",
    desc: "错误率 / p95 / 平均延迟超阈值自动触发，30s 评估、10min 冷却",
  },
];

export default function DemoPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const startDemo = async () => {
    setLoading(true);
    setError("");
    try {
      const p = await api.createProject("demo-project");
      setSession(p.api_key, p.id);
      router.push("/");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="mx-auto max-w-4xl p-8">
      <div className="text-center">
        <h1 className="text-4xl font-bold">TracePilot</h1>
        <p className="mt-3 text-xl text-gray-600">Agent 运行时可观测平台</p>
        <p className="mt-2 text-gray-500">
          接入、存储、查看 Agent 运行时的执行轨迹——Agent 的 Datadog
        </p>
      </div>

      <div className="mt-12 text-center">
        <button
          onClick={startDemo}
          disabled={loading}
          className="bg-blue-600 px-8 py-3 text-lg text-white disabled:opacity-50"
        >
          {loading ? "启动中…" : "🚀 一键启动演示"}
        </button>
        {error && <p className="mt-4 text-red-600">{error}</p>}
        <p className="mt-3 text-sm text-gray-500">
          自动创建演示项目并跳转到 trace 列表，跑 seed_demo.py 即可灌入演示数据
        </p>
      </div>

      <div className="mt-16 grid grid-cols-2 gap-6">
        {FEATURES.map((f) => (
          <div key={f.title} className="border bg-white p-6">
            <div className="text-3xl">{f.icon}</div>
            <h3 className="mt-3 text-lg font-semibold">{f.title}</h3>
            <p className="mt-1 text-sm text-gray-500">{f.desc}</p>
          </div>
        ))}
      </div>

      <div className="mt-12 border bg-gray-50 p-6">
        <h3 className="text-lg font-semibold">演示流程</h3>
        <ol className="mt-3 space-y-2 text-sm text-gray-600">
          <li>1. 点击「一键启动演示」创建演示项目</li>
          <li>2. 运行 <code className="bg-gray-200 px-1">python scripts/seed_demo.py</code> 灌入演示数据</li>
          <li>3. 在 trace 列表点击任意 trace 查看瀑布图</li>
          <li>4. 切换到「监控」「分析」「恢复」「告警」查看各维度数据</li>
        </ol>
      </div>
    </main>
  );
}
