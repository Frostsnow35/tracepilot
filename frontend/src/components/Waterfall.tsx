"use client";

import { useMemo } from "react";
import ReactECharts from "echarts-for-react";
import type { Span } from "@/lib/api";

export default function Waterfall({ spans }: { spans: Span[] }) {
  const option = useMemo(() => {
    const sorted = [...spans].sort((a, b) => a.started_at.localeCompare(b.started_at));
    const categories = sorted.map((s) => s.name);
    const data = sorted.map((s) => ({
      value: s.duration_ms,
      itemStyle: {
        color: s.status === "error" ? "#ef4444" : s.status === "ok" ? "#3b82f6" : "#f59e0b",
      },
      _span: s,
    }));
    return {
      tooltip: {
        trigger: "axis",
        axisPointer: { type: "shadow" },
        formatter: (params: { dataIndex: number }[]) => {
          const s = sorted[params[0].dataIndex];
          return `${s.name}<br/>${s.kind} · ${s.status}<br/>${s.duration_ms} ms`;
        },
      },
      grid: { left: 220, right: 24, top: 16, bottom: 32 },
      xAxis: { type: "value", name: "ms", splitLine: { lineStyle: { type: "dashed" } } },
      yAxis: {
        type: "category",
        data: categories,
        axisLabel: { fontSize: 11 },
      },
      series: [{ type: "bar", data, barMaxWidth: 28 }],
    };
  }, [spans]);

  return <ReactECharts option={option} style={{ height: 360 }} notMerge />;
}
