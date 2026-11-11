-- TracePilot 初始迁移：项目/追踪/span，span 为 TimescaleDB hypertable。
-- 时间策略：业务时间 started_at 存 TEXT（自由格式），hypertable 用 recorded_at（入库时间）分块，
-- 避免时区/格式解析问题。

CREATE TABLE IF NOT EXISTS projects (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    api_key_hash  TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS traces (
    id           TEXT PRIMARY KEY,
    project_id   TEXT NOT NULL REFERENCES projects(id),
    agent_id     TEXT,
    status       TEXT NOT NULL,
    started_at   TEXT NOT NULL,
    duration_ms  BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS spans (
    id              TEXT PRIMARY KEY,
    trace_id        TEXT NOT NULL REFERENCES traces(id),
    parent_span_id  TEXT,
    name            TEXT NOT NULL,
    kind            TEXT NOT NULL,
    status          TEXT NOT NULL,
    started_at      TEXT NOT NULL,
    duration_ms     BIGINT NOT NULL,
    attributes      JSONB NOT NULL DEFAULT '{}'::jsonb,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT create_hypertable('spans', 'recorded_at', if_not_exists => TRUE);

CREATE INDEX IF NOT EXISTS spans_trace_id_idx ON spans (trace_id);
CREATE INDEX IF NOT EXISTS spans_started_at_idx ON spans (started_at);
CREATE INDEX IF NOT EXISTS traces_project_started_idx ON traces (project_id, started_at DESC);
