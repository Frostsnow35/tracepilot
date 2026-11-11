"""WebSocket 实时流验证：连接 /api/monitor/stream，然后灌一条 trace，确认收到事件。

用法：后端已启动，运行：python examples/ws_verify.py
"""

from __future__ import annotations

import json
import os
import sys
import threading
import time
import uuid

SDK_PATH = os.path.join(os.path.dirname(__file__), "..", "sdk", "python")
sys.path.insert(0, SDK_PATH)

from tracepilot_sdk import Span, Trace, TracePilotClient

import websocket  # websocket-client


def iso(offset_ms: int) -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(time.time() + offset_ms / 1000))


def main() -> None:
    client = TracePilotClient()
    project = client.create_project("ws-verify")
    api_key = project["api_key"]
    client.api_key = api_key
    pid = project["id"]
    print(f"[ws] project: {pid}")

    received: list[dict] = []
    done = threading.Event()

    def on_message(ws, message):
        ev = json.loads(message)
        received.append(ev)
        print(f"[ws] received: trace={ev['trace_id'][:12]} agent={ev.get('agent_id')} status={ev['status']}")
        done.set()

    ws_url = (
        f"ws://localhost:8000/api/monitor/stream?project_id={pid}&api_key={api_key}"
    )
    ws = websocket.WebSocketApp(
        ws_url,
        on_message=on_message,
        on_error=lambda ws, e: print(f"[ws] error: {e}"),
        on_close=lambda ws, code, msg: print(f"[ws] closed: {code}"),
    )
    t = threading.Thread(target=ws.run_forever, daemon=True)
    t.start()
    time.sleep(1)  # 等 WS 建连

    # 灌一条 trace，触发广播
    tid = uuid.uuid4().hex
    root = uuid.uuid4().hex
    t_obj = Trace(
        id=tid,
        project_id=pid,
        agent_id="ws-agent",
        status="ok",
        started_at=iso(0),
        duration_ms=42,
    )
    spans = [
        Span(
            id=root,
            trace_id=tid,
            name="agent.run",
            kind="agent",
            status="ok",
            started_at=iso(0),
            duration_ms=42,
            attributes={},
        )
    ]
    client.ingest(t_obj, spans)
    print(f"[ws] trace ingested: {tid}")

    # 等广播到达（后台轮询间隔 5s，给足 12s）
    ok = done.wait(timeout=12)
    ws.close()
    if ok and len(received) >= 1:
        ev = received[0]
        assert ev["trace_id"] == tid, f"trace_id mismatch: {ev['trace_id']} vs {tid}"
        print("[ws] ALL CHECKS PASSED")
    else:
        print(f"[ws] FAILED: received={len(received)}")
        sys.exit(1)


if __name__ == "__main__":
    main()
