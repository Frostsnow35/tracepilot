-- TracePilot 告警迁移（FR-8）：规则表 + 触发事件表。
-- 规则按项目隔离，指标为 error_rate / p95_ms / avg_ms；
-- 事件表记录每次触发（带冷却去重，由后台评估任务写入）。

CREATE TABLE IF NOT EXISTS alert_rules (
    id              TEXT PRIMARY KEY,
    project_id      TEXT NOT NULL REFERENCES projects(id),
    metric          TEXT NOT NULL,
    operator        TEXT NOT NULL DEFAULT '>',
    threshold       DOUBLE PRECISION NOT NULL,
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    last_triggered_at TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS alert_rules_project_idx ON alert_rules (project_id);

CREATE TABLE IF NOT EXISTS alert_events (
    id              TEXT PRIMARY KEY,
    project_id      TEXT NOT NULL REFERENCES projects(id),
    rule_id         TEXT NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    metric          TEXT NOT NULL,
    value           DOUBLE PRECISION NOT NULL,
    threshold       DOUBLE PRECISION NOT NULL,
    triggered_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS alert_events_project_idx ON alert_events (project_id, triggered_at DESC);
