#!/usr/bin/env python3
"""TracePilot 演示数据生成器：向运行中的实例灌入多组真实形态的 trace。

用法：
    python scripts/seed_demo.py [--base-url http://localhost:8000] [--count 20]

产生的数据覆盖：成功链路、LLM 调用、tool 调用、检索失败+重试、Agent 整体失败、
恢复成功的 trace（失败 span + 重试 span + 最终 ok）、多 Agent 混合。
"""

import argparse
import random
import sys
import time
import uuid
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "sdk" / "python"))

from tracepilot_sdk.client import TracePilotClient  # noqa: E402
from tracepilot_sdk.models import Span, Trace  # noqa: E402

AGENT_NAMES = ["research-bot", "code-reviewer", "data-analyst"]
KINDS = ["llm", "tool", "retrieval", "agent", "chain"]


def rnd_id(prefix: str) -> str:
    return f"{prefix}-{uuid.uuid4().hex[:16]}"


def now_iso() -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def make_span(
    trace_id: str,
    parent: str | None,
    name: str,
    kind: str,
    status: str,
    duration_ms: int,
    attempt: int = 1,
) -> Span:
    attributes = {"attempt": attempt, "kind": kind}
    if kind == "llm":
        attributes["model"] = random.choice(["gpt-4o", "claude-3.5", "qwen-max"])
        attributes["tokens"] = random.randint(500, 4000)
    if kind == "tool":
        attributes["tool"] = random.choice(["search", "code_exec", "http_get"])
    return Span(
        id=rnd_id("span"),
        trace_id=trace_id,
        parent_span_id=parent,
        name=name,
        kind=kind,
        status=status,
        started_at=now_iso(),
        duration_ms=duration_ms,
        attributes=attributes,
    )


def make_trace(agent: str, scenario: str) -> tuple[Trace, list[Span]]:
    """按场景生成一条 trace 及其 spans。"""
    trace_id = rnd_id("trace")
    spans: list[Span] = []

    def add(parent: str | None, name: str, kind: str, status: str, dur: int, attempt: int = 1) -> Span:
        s = make_span(trace_id, parent, name, kind, status, dur, attempt)
        spans.append(s)
        return s

    if scenario == "ok":
        root = add(None, "agent.run", "agent", "ok", 800)
        add(root.id, "llm.call", "llm", "ok", random.randint(800, 2000))
        add(root.id, "tool.search", "tool", "ok", random.randint(100, 500))
    elif scenario == "retry-recovered":
        root = add(None, "agent.run", "agent", "ok", 1500)
        add(root.id, "retrieval.search", "retrieval", "error", 300, attempt=1)
        add(root.id, "retrieval.search", "retrieval", "ok", 400, attempt=2)
        add(root.id, "llm.call", "llm", "ok", 1200)
    elif scenario == "failed":
        root = add(None, "agent.run", "agent", "error", 600)
        add(root.id, "llm.call", "llm", "error", 500)
    elif scenario == "multi-step":
        root = add(None, "agent.run", "agent", "ok", 3000)
        plan = add(root.id, "llm.plan", "llm", "ok", 800)
        add(root.id, "tool.code_exec", "tool", "ok", 600)
        add(plan.id, "llm.call", "llm", "ok", 1500)
        add(root.id, "llm.summarize", "llm", "ok", 900)
    else:
        add(None, "agent.run", "agent", "ok", 500)

    status = "ok" if scenario in ("ok", "retry-recovered", "multi-step") else "error"
    trace = Trace(
        id=trace_id,
        project_id="",  # 由 client 填充
        agent_id=agent,
        status=status,
        started_at=now_iso(),
        duration_ms=sum(s.duration_ms for s in spans),
    )
    return trace, spans


def main() -> None:
    parser = argparse.ArgumentParser(description="Seed TracePilot with demo traces")
    parser.add_argument("--base-url", default="http://localhost:8000")
    parser.add_argument("--count", type=int, default=20)
    args = parser.parse_args()

    # 用固定 dev key 创建/复用项目
    client = TracePilotClient(args.base_url, "tp_local_dev_key_2026")
    project = client.create_project("demo-project")
    project_id = project["id"]
    client.api_key = project["api_key"]
    print(f"[seed] project: {project_id}")

    scenarios = ["ok", "retry-recovered", "failed", "multi-step"]
    for i in range(args.count):
        agent = AGENT_NAMES[i % len(AGENT_NAMES)]
        scenario = scenarios[i % len(scenarios)]
        trace, spans = make_trace(agent, scenario)
        trace.project_id = project_id
        client.ingest(trace, spans)
        print(f"[seed] {i+1}/{args.count} agent={agent} scenario={scenario} spans={len(spans)}")
        time.sleep(0.1)

    print(f"[seed] done: {args.count} traces ingested into {project_id}")
    print(f"[seed] open http://localhost:3000 and select project 'demo-project'")


if __name__ == "__main__":
    main()
