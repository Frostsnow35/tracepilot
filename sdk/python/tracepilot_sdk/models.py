"""领域数据模型：Trace 与 Span。"""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from typing import Optional


@dataclass
class Trace:
    id: str
    project_id: str
    agent_id: Optional[str] = None
    status: str = "unknown"
    started_at: str = ""
    duration_ms: int = 0

    def to_dict(self) -> dict:
        return asdict(self)


@dataclass
class Span:
    id: str
    trace_id: str
    name: str
    kind: str
    status: str
    started_at: str
    duration_ms: int
    parent_span_id: Optional[str] = None
    attributes: dict = field(default_factory=dict)

    def to_dict(self) -> dict:
        return asdict(self)
