-- 0002：监控与分析支撑。
-- traces 加 recorded_at（入库时间），monitor 后台轮询用它做水位线，
-- 区分"业务时间 started_at"与"入库时间"，避免重放历史数据。

ALTER TABLE traces ADD COLUMN IF NOT EXISTS recorded_at TIMESTAMPTZ NOT NULL DEFAULT now();

CREATE INDEX IF NOT EXISTS traces_project_recorded_idx ON traces (project_id, recorded_at);
