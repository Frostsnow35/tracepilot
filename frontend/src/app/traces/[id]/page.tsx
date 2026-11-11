"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { api, type TraceDetail } from "@/lib/api";
import Waterfall from "@/components/Waterfall";

export default function TraceDetailPage() {
  const { id } = useParams<{ id: string }>();
  const [data, setData] = useState<TraceDetail | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    if (!id) return;
    api.getTrace(id as string).then(setData).catch((e) => setError(String(e)));
  }, [id]);

  if (error) return <main className="p-8 text-red-600">{error}</main>;
  if (!data) return <main className="p-8 text-gray-500">加载中…</main>;

  return (
    <main className="mx-auto max-w-5xl p-8">
      <Link href="/" className="text-sm text-blue-600 hover:underline">
        ← 返回列表
      </Link>
      <h1 className="mt-2 text-xl font-semibold">Trace {data.trace.id.slice(0, 12)}…</h1>
      <div className="mt-2 flex gap-4 text-sm text-gray-600">
        <span>Agent: {data.trace.agent_id || "-"}</span>
        <span>状态: {data.trace.status}</span>
        <span>耗时: {data.trace.duration_ms} ms</span>
      </div>

      <h2 className="mt-6 text-lg">瀑布图</h2>
      <div className="mt-3 rounded border bg-white p-2">
        <Waterfall spans={data.spans} />
      </div>

      <h2 className="mt-6 text-lg">Spans ({data.spans.length})</h2>
      <table className="mt-3 w-full border-collapse text-sm">
        <thead>
          <tr className="border-b text-left text-gray-500">
            <th className="py-2">Name</th>
            <th className="py-2">Kind</th>
            <th className="py-2">Status</th>
            <th className="py-2">Duration</th>
            <th className="py-2">Attributes</th>
          </tr>
        </thead>
        <tbody>
          {data.spans.map((s) => (
            <tr key={s.id} className="border-b">
              <td className="py-2">{s.name}</td>
              <td className="py-2">{s.kind}</td>
              <td className="py-2">
                <span
                  className={`rounded px-2 py-0.5 text-xs text-white ${
                    s.status === "error" ? "bg-red-500" : "bg-green-500"
                  }`}
                >
                  {s.status}
                </span>
              </td>
              <td className="py-2">{s.duration_ms} ms</td>
              <td className="py-2 text-gray-500">{JSON.stringify(s.attributes)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </main>
  );
}
