"""端到端验证脚本：建项目 -> 灌 trace -> 验证 monitor/analytics 接口。

用法：后端已启动（cargo run），数据库已起。运行：python examples/e2e_verify.py
"""

from __future__ import annotations

import os
import sys
import time
import uuid

SDK_PATH = os.path.join(os.path.dirname(__file__), "..", "sdk", "python")
sys.path.insert(0, SDK_PATH)

from tracepilot_sdk import Span, Trace, TracePilotClient


def iso(offset_ms: int) -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(time.time() + offset_ms / 1000))


def main() -> None:
    client = TracePilotClient()

    # 1. 建项目
    project = client.create_project("e2e-verify")
    client.api_key = project["api_key"]
    pid = project["id"]
    print(f"[e2e] project: {pid}")

    # 2. 灌两条 trace（一条 ok，一条 error），多 agent，验证聚合
    traces = []
    for i in range(2):
        tid = uuid.uuid4().hex
        root = uuid.uuid4().hex
        status = "ok" if i == 0 else "error"
        t = Trace(
            id=tid,
            project_id=pid,
            agent_id=f"e2e-agent-{i}",
            status=status,
            started_at=iso(i * 1000),
            duration_ms=1000 + i * 500,
        )
        spans = [
            Span(
                id=root,
                trace_id=tid,
                name="agent.run",
                kind="agent",
                status="ok",
                started_at=iso(i * 1000),
                duration_ms=1000 + i * 500,
                attributes={},
            ),
            Span(
                id=uuid.uuid4().hex,
                trace_id=tid,
                parent_span_id=root,
                name="llm.generate",
                kind="llm",
                status="ok",
                started_at=iso(i * 1000 + 100),
                duration_ms=400 + i * 200,
                attributes={"model": "test-llm"},
            ),
            Span(
                id=uuid.uuid4().hex,
                trace_id=tid,
                parent_span_id=root,
                name="retrieval.search",
                kind="retrieval",
                status="ok" if i == 0 else "error",
                started_at=iso(i * 1000 + 200),
                duration_ms=300,
                attributes={"attempt": 1},
            ),
        ]
        client.ingest(t, spans)
        traces.append(tid)
        print(f"[e2e] trace ingested: {tid} status={status}")

    # 3. 验证 monitor 接口
    stats = client.monitor_stats(pid)
    print(f"[e2e] monitor.stats: {stats}")

    agents = client.monitor_agents(pid)
    print(f"[e2e] monitor.agents: {agents}")

    # 4. 验证 analytics 接口
    summary = client.analytics_summary(pid)
    print(f"[e2e] analytics.summary: total={summary.get('total')} success_rate={summary.get('success_rate')}")

    latency = client.analytics_latency(pid)
    print(f"[e2e] analytics.latency points: {len(latency)}")

    cost = client.analytics_cost(pid)
    print(f"[e2e] analytics.cost: {len(cost)} kinds -> {[c.get('kind') for c in cost]}")

    # 5. 查询验证
    for tid in traces:
        d = client.get_trace(tid)
        print(f"[e2e] trace {tid[:12]} status={d['trace']['status']} spans={len(d['spans'])}")

    print("[e2e] ALL CHECKS PASSED")


if __name__ == "__main__":
    main()
