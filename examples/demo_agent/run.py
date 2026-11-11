"""示例 Agent：模拟一个搜索增强问答 Agent，产生含失败+重试的真实 trace。

用法：先启动后端（cargo run）与数据库，再运行本脚本。
端到端闭环：创建项目 -> 产生 trace -> 接入 -> 查询验证。
"""

from __future__ import annotations

import os
import sys
import time
import uuid

# 引入本地 SDK（无需安装）
SDK_PATH = os.path.join(os.path.dirname(__file__), "..", "..", "sdk", "python")
sys.path.insert(0, SDK_PATH)

from tracepilot_sdk import Span, Trace, TracePilotClient


def iso(offset_ms: int) -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(time.time() + offset_ms / 1000))


def main() -> None:
    client = TracePilotClient()

    # 1. 创建项目，拿到 api_key 与 project_id
    project = client.create_project("demo-search-agent")
    client.api_key = project["api_key"]
    project_id = project["id"]
    print(f"[demo] project created: {project_id}")

    # 2. 构造 trace：agent -> retrieval(失败+重试) -> llm
    trace_id = uuid.uuid4().hex
    root = uuid.uuid4().hex
    trace = Trace(
        id=trace_id,
        project_id=project_id,
        agent_id="demo-search-agent",
        status="unknown",
        started_at=iso(0),
        duration_ms=0,
    )
    spans = [
        Span(
            id=root,
            trace_id=trace_id,
            name="agent.run",
            kind="agent",
            status="ok",
            started_at=iso(0),
            duration_ms=1200,
            attributes={"goal": "回答用户问题"},
        ),
        Span(
            id=uuid.uuid4().hex,
            trace_id=trace_id,
            parent_span_id=root,
            name="retrieval.search",
            kind="retrieval",
            status="error",
            started_at=iso(100),
            duration_ms=300,
            attributes={"attempt": 1, "error": "timeout"},
        ),
        Span(
            id=uuid.uuid4().hex,
            trace_id=trace_id,
            parent_span_id=root,
            name="retrieval.search.retry",
            kind="retrieval",
            status="ok",
            started_at=iso(500),
            duration_ms=280,
            attributes={"attempt": 2},
        ),
        Span(
            id=uuid.uuid4().hex,
            trace_id=trace_id,
            parent_span_id=root,
            name="llm.generate",
            kind="llm",
            status="ok",
            started_at=iso(900),
            duration_ms=620,
            attributes={"model": "demo-llm", "tokens": 128},
        ),
    ]
    trace.duration_ms = 1200
    trace.status = "ok"

    # 3. 接入
    result = client.ingest(trace, spans)
    print(f"[demo] trace ingested: {result}")

    # 4. 查询验证
    detail = client.get_trace(trace_id)
    print(f"[demo] trace status={detail['trace']['status']} spans={len(detail['spans'])}")
    for s in detail["spans"]:
        print(f"  - {s['name']:24s} [{s['kind']:9s}] {s['status']}")


if __name__ == "__main__":
    main()
