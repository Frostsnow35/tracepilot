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

    def monitor_stats(self, project_id: str) -> dict:
        """FR-4：统计概览（total/success/error/avg/p50/p95）。"""
        r = httpx.get(
            f"{self.base_url}/api/monitor/stats",
            params={"project_id": project_id},
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def monitor_agents(self, project_id: str) -> list[dict]:
        """FR-4：按 agent 聚合统计。"""
        r = httpx.get(
            f"{self.base_url}/api/monitor/agents",
            params={"project_id": project_id},
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def analytics_summary(self, project_id: str) -> dict:
        """FR-5：成功率 + 延迟分位 + span 类型分布。"""
        r = httpx.get(
            f"{self.base_url}/api/analytics/summary",
            params={"project_id": project_id},
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def analytics_latency(self, project_id: str) -> list[dict]:
        """FR-5：小时级延迟时序。"""
        r = httpx.get(
            f"{self.base_url}/api/analytics/latency",
            params={"project_id": project_id},
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def analytics_cost(self, project_id: str) -> list[dict]:
        """FR-5：按 span 类型的成本拆解。"""
        r = httpx.get(
            f"{self.base_url}/api/analytics/cost",
            params={"project_id": project_id},
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def recovery_stats(self, project_id: str) -> dict:
        """FR-6：恢复观测汇总（失败/重试 span、恢复/失败 trace）。"""
        r = httpx.get(
            f"{self.base_url}/api/recovery/stats",
            params={"project_id": project_id},
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def recovery_agents(self, project_id: str) -> list[dict]:
        """FR-6：按 agent 聚合恢复观测。"""
        r = httpx.get(
            f"{self.base_url}/api/recovery/agents",
            params={"project_id": project_id},
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def recovery_traces(self, project_id: str, limit: int = 50) -> list[dict]:
        """FR-6：含失败/重试的 trace 列表。"""
        r = httpx.get(
            f"{self.base_url}/api/recovery/traces",
            params={"project_id": project_id, "limit": limit},
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def alert_rules(self, project_id: str) -> list[dict]:
        """FR-8：列出告警规则。"""
        r = httpx.get(
            f"{self.base_url}/api/alert/rules",
            params={"project_id": project_id},
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def create_alert_rule(
        self, project_id: str, metric: str, operator: str, threshold: float
    ) -> dict:
        """FR-8：创建告警规则（metric ∈ error_rate/p95_ms/avg_ms，operator ∈ </>）。"""
        r = httpx.post(
            f"{self.base_url}/api/alert/rules",
            json={
                "project_id": project_id,
                "metric": metric,
                "operator": operator,
                "threshold": threshold,
            },
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def delete_alert_rule(self, project_id: str, rule_id: str) -> dict:
        """FR-8：删除告警规则。"""
        r = httpx.delete(
            f"{self.base_url}/api/alert/rules/{rule_id}",
            params={"project_id": project_id},
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()

    def alert_events(self, project_id: str, limit: int = 50) -> list[dict]:
        """FR-8：列出告警触发事件。"""
        r = httpx.get(
            f"{self.base_url}/api/alert/events",
            params={"project_id": project_id, "limit": limit},
            headers=self._auth(),
        )
        r.raise_for_status()
        return r.json()
