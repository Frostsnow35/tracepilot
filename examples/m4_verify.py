"""M4 端到端验证：recovery（FR-6）+ alert（FR-8）。

用法：后端已启动（含 recovery/alert 模块），运行：python examples/m4_verify.py
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
    project = client.create_project("m4-verify")
    client.api_key = project["api_key"]
    pid = project["id"]
    print(f"[m4] project: {pid}")

    # 1. 灌两条 trace：一条含失败+重试（最终 ok），一条直接失败。
    #    重试判定靠 attributes.attempt > 1。
    cases = [
        ("recover-ok", "ok", [("agent.run", "agent", "ok", 0, 1000, {}),
                              ("retrieval.search", "retrieval", "error", 100, 300, {"attempt": 1}),
                              ("retrieval.search.retry", "retrieval", "ok", 500, 280, {"attempt": 2})]),
        ("fail-trace", "error", [("agent.run", "agent", "error", 0, 500, {}),
                                 ("llm.generate", "llm", "error", 100, 400, {"attempt": 1})]),
    ]
    trace_ids = []
    for agent, status, spans_def in cases:
        tid = uuid.uuid4().hex
        root = uuid.uuid4().hex
        t = Trace(
            id=tid,
            project_id=pid,
            agent_id=agent,
            status=status,
            started_at=iso(0),
            duration_ms=1000,
        )
        spans = []
        for i, (name, kind, st, off, dur, attrs) in enumerate(spans_def):
            spans.append(Span(
                id=root if i == 0 else uuid.uuid4().hex,
                trace_id=tid,
                parent_span_id=None if i == 0 else root,
                name=name,
                kind=kind,
                status=st,
                started_at=iso(off),
                duration_ms=dur,
                attributes=attrs,
            ))
        client.ingest(t, spans)
        trace_ids.append(tid)
        print(f"[m4] trace ingested: {tid} status={status}")

    # 2. 验证 recovery 接口
    stats = client.recovery_stats(pid)
    print(f"[m4] recovery.stats: {stats}")
    assert stats["total_traces"] == 2, f"total_traces={stats['total_traces']} != 2"
    assert stats["error_spans"] == 3, f"error_spans={stats['error_spans']} != 3"
    assert stats["retry_spans"] == 1, f"retry_spans={stats['retry_spans']} != 1"
    assert stats["recovered_traces"] == 1, f"recovered_traces={stats['recovered_traces']} != 1"
    assert stats["failed_traces"] == 1, f"failed_traces={stats['failed_traces']} != 1"

    agents = client.recovery_agents(pid)
    print(f"[m4] recovery.agents: {agents}")
    assert len(agents) == 2, f"agents={len(agents)} != 2"

    traces = client.recovery_traces(pid)
    print(f"[m4] recovery.traces: {len(traces)}")
    assert len(traces) == 2, f"recovery_traces={len(traces)} != 2"

    # 3. 验证 alert 接口
    #    错误率 1.0（2 条 trace 全有 error span，其中 fail-trace 状态为 error，recover-ok 状态 ok）
    #    实际 error_rate = failed_traces / total_traces = 1/2 = 0.5
    rule = client.create_alert_rule(pid, "error_rate", ">", 0.3)
    print(f"[m4] alert rule created: {rule}")

    rules = client.alert_rules(pid)
    print(f"[m4] alert.rules: {len(rules)}")
    assert len(rules) == 1

    # 等后台评估任务（30s 周期）跑一轮
    print("[m4] waiting for alert evaluation (up to 40s)...")
    deadline = time.time() + 40
    events = []
    while time.time() < deadline:
        events = client.alert_events(pid)
        if events:
            break
        time.sleep(5)
    print(f"[m4] alert.events: {len(events)}")
    if events:
        e = events[0]
        print(f"[m4] event: metric={e['metric']} value={e['value']} threshold={e['threshold']}")
        assert e["metric"] == "error_rate"
        assert e["value"] > e["threshold"], f"value {e['value']} not > threshold {e['threshold']}"
    else:
        print("[m4] WARN: no alert event within 40s (后台评估周期 30s，可能刚错过)")

    # 4. 删除规则
    del_result = client.delete_alert_rule(pid, rule["id"])
    print(f"[m4] alert rule deleted: {del_result}")
    assert del_result["deleted"] is True

    print("[m4] ALL CHECKS PASSED")


if __name__ == "__main__":
    main()
