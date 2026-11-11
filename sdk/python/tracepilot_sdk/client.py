"""TracePilot Python SDK：创建项目、接入 trace、查询。"""

from __future__ import annotations

from typing import Optional

import httpx

from .models import Span, Trace


class TracePilotClient:
    def __init__(
        self,
        base_url: str = "http://localhost:8000",
        api_key: Optional[str] = None,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self._headers: dict[str, str] = {"Content-Type": "application/json"}
        if api_key:
            self._headers["X-TracePilot-Key"] = api_key

    def _auth(self) -> dict[str, str]:
        if self.api_key:
            self._headers["X-TracePilot-Key"] = self.api_key
        return self._headers

    def create_project(self, name: str) -> dict:
        """创建项目，返回 {id, name, api_key}。"""
        r = httpx.post(
            f"{self.base_url}/api/iam/projects",
            json={"name": name},
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def ingest(self, trace: Trace, spans: list[Span]) -> dict:
        """上报一条 trace 及其 spans。"""
        payload = {
            "trace": trace.to_dict(),
            "spans": [s.to_dict() for s in spans],
        }
        r = httpx.post(
            f"{self.base_url}/api/ingest/traces",
            json=payload,
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def list_traces(self, project_id: str, limit: int = 50) -> list[dict]:
        r = httpx.get(
            f"{self.base_url}/api/query/traces",
            params={"project_id": project_id, "limit": limit},
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def get_trace(self, trace_id: str) -> dict:
        r = httpx.get(
            f"{self.base_url}/api/query/traces/{trace_id}",
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()
