// storage 模块：trace/span 的读写（FR-2）。
// 只封装 SQL，不暴露 web 层；其他模块通过 Storage 访问数据。

use axum::Router;
use serde_json::json;
use sqlx::{PgPool, Row};
use tracepilot_core::{Span, Trace};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Clone)]
pub struct Storage {
    pool: PgPool,
}

impl Storage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn insert_trace(&self, trace: &Trace) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO traces (id, project_id, agent_id, status, started_at, duration_ms)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&trace.id)
        .bind(&trace.project_id)
        .bind(&trace.agent_id)
        .bind(&trace.status)
        .bind(&trace.started_at)
        .bind(trace.duration_ms)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_span(&self, span: &Span) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO spans (id, trace_id, parent_span_id, name, kind, status, started_at, duration_ms, attributes)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(&span.id)
        .bind(&span.trace_id)
        .bind(&span.parent_span_id)
        .bind(&span.name)
        .bind(&span.kind)
        .bind(&span.status)
        .bind(&span.started_at)
        .bind(span.duration_ms)
        .bind(span.attributes.clone())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_trace(&self, id: &str) -> Result<Option<Trace>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, project_id, agent_id, status, started_at, duration_ms
             FROM traces WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Trace {
            id: r.get("id"),
            project_id: r.get("project_id"),
            agent_id: r.get("agent_id"),
            status: r.get("status"),
            started_at: r.get("started_at"),
            duration_ms: r.get("duration_ms"),
        }))
    }

    pub async fn get_spans_by_trace(&self, trace_id: &str) -> Result<Vec<Span>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, trace_id, parent_span_id, name, kind, status, started_at, duration_ms, attributes
             FROM spans WHERE trace_id = $1 ORDER BY started_at, id",
        )
        .bind(trace_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| Span {
                id: r.get("id"),
                trace_id: r.get("trace_id"),
                parent_span_id: r.get("parent_span_id"),
                name: r.get("name"),
                kind: r.get("kind"),
                status: r.get("status"),
                started_at: r.get("started_at"),
                duration_ms: r.get("duration_ms"),
                attributes: r.get("attributes"),
            })
            .collect())
    }

    pub async fn list_traces(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<Trace>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, project_id, agent_id, status, started_at, duration_ms
             FROM traces WHERE project_id = $1
             ORDER BY started_at DESC LIMIT $2",
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| Trace {
                id: r.get("id"),
                project_id: r.get("project_id"),
                agent_id: r.get("agent_id"),
                status: r.get("status"),
                started_at: r.get("started_at"),
                duration_ms: r.get("duration_ms"),
            })
            .collect())
    }

    /// 监控概览（FR-4）：总量 / 成功 / 失败 / 平均延迟 / p50 / p95。
    pub async fn monitor_stats(&self, project_id: &str) -> Result<serde_json::Value, sqlx::Error> {
        let row = sqlx::query(
            "SELECT count(*) AS total,
                    count(*) FILTER (WHERE status = 'ok') AS success,
                    count(*) FILTER (WHERE status != 'ok') AS error,
                    avg(duration_ms)::float8 AS avg_ms,
                    percentile_cont(0.5) WITHIN GROUP (ORDER BY duration_ms) AS p50,
                    percentile_cont(0.95) WITHIN GROUP (ORDER BY duration_ms) AS p95
             FROM traces WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(json!({
            "total": row.get::<i64, _>("total"),
            "success": row.get::<i64, _>("success"),
            "error": row.get::<i64, _>("error"),
            "avg_ms": row.get::<f64, _>("avg_ms"),
            "p50_ms": row.get::<f64, _>("p50"),
            "p95_ms": row.get::<f64, _>("p95"),
        }))
    }

    /// 按 agent 聚合（FR-4）：每个 agent 的总量 / 成功 / 平均延迟 / p95。
    pub async fn agent_stats(
        &self,
        project_id: &str,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT coalesce(agent_id, 'unknown') AS agent,
                    count(*) AS total,
                    count(*) FILTER (WHERE status = 'ok') AS success,
                    avg(duration_ms)::float8 AS avg_ms,
                    percentile_cont(0.95) WITHIN GROUP (ORDER BY duration_ms) AS p95
             FROM traces WHERE project_id = $1
             GROUP BY coalesce(agent_id, 'unknown')
             ORDER BY total DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                json!({
                    "agent": r.get::<String, _>("agent"),
                    "total": r.get::<i64, _>("total"),
                    "success": r.get::<i64, _>("success"),
                    "avg_ms": r.get::<f64, _>("avg_ms"),
                    "p95_ms": r.get::<f64, _>("p95"),
                })
            })
            .collect())
    }

    /// 分析汇总（FR-5）：成功率 + 延迟分位 + span 类型分布。
    pub async fn analytics_summary(
        &self,
        project_id: &str,
    ) -> Result<serde_json::Value, sqlx::Error> {
        let trace_row = sqlx::query(
            "SELECT count(*) AS total,
                    count(*) FILTER (WHERE status = 'ok') AS success,
                    avg(duration_ms)::float8 AS avg_ms,
                    percentile_cont(0.5) WITHIN GROUP (ORDER BY duration_ms) AS p50,
                    percentile_cont(0.95) WITHIN GROUP (ORDER BY duration_ms) AS p95,
                    percentile_cont(0.99) WITHIN GROUP (ORDER BY duration_ms) AS p99
             FROM traces WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        let total = trace_row.get::<i64, _>("total");
        let success = trace_row.get::<i64, _>("success");

        let kind_rows = sqlx::query(
            "SELECT kind,
                    count(*) AS total,
                    count(*) FILTER (WHERE status = 'ok') AS success,
                    avg(duration_ms)::float8 AS avg_ms
             FROM spans
             WHERE trace_id IN (SELECT id FROM traces WHERE project_id = $1)
             GROUP BY kind
             ORDER BY total DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        let kinds: Vec<serde_json::Value> = kind_rows
            .into_iter()
            .map(|r| {
                json!({
                    "kind": r.get::<String, _>("kind"),
                    "total": r.get::<i64, _>("total"),
                    "success": r.get::<i64, _>("success"),
                    "avg_ms": r.get::<f64, _>("avg_ms"),
                })
            })
            .collect();

        Ok(json!({
            "total": total,
            "success": success,
            "success_rate": if total > 0 { (success as f64) / (total as f64) } else { 0.0 },
            "avg_ms": trace_row.get::<f64, _>("avg_ms"),
            "p50_ms": trace_row.get::<f64, _>("p50"),
            "p95_ms": trace_row.get::<f64, _>("p95"),
            "p99_ms": trace_row.get::<f64, _>("p99"),
            "span_kinds": kinds,
        }))
    }

    /// 延迟时序（FR-5）：按小时桶聚合，供前端画趋势图。
    pub async fn latency_timeseries(
        &self,
        project_id: &str,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT date_trunc('hour', started_at::timestamp)::text AS bucket,
                    count(*) AS total,
                    avg(duration_ms)::float8 AS avg_ms,
                    percentile_cont(0.95) WITHIN GROUP (ORDER BY duration_ms) AS p95,
                    count(*) FILTER (WHERE status = 'ok') AS success
             FROM traces
             WHERE project_id = $1
             GROUP BY bucket
             ORDER BY bucket",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                json!({
                    "bucket": r.get::<String, _>("bucket"),
                    "total": r.get::<i64, _>("total"),
                    "avg_ms": r.get::<f64, _>("avg_ms"),
                    "p95_ms": r.get::<f64, _>("p95"),
                    "success": r.get::<i64, _>("success"),
                })
            })
            .collect())
    }

    /// 成本拆解（FR-5）：按 span 类型的总耗时占比（proxy：耗时即成本代理）。
    pub async fn cost_breakdown(
        &self,
        project_id: &str,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT kind,
                    count(*) AS total,
                    sum(duration_ms)::bigint AS total_ms,
                    avg(duration_ms)::float8 AS avg_ms
             FROM spans
             WHERE trace_id IN (SELECT id FROM traces WHERE project_id = $1)
             GROUP BY kind
             ORDER BY total_ms DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                json!({
                    "kind": r.get::<String, _>("kind"),
                    "total": r.get::<i64, _>("total"),
                    "total_ms": r.get::<i64, _>("total_ms"),
                    "avg_ms": r.get::<f64, _>("avg_ms"),
                })
            })
            .collect())
    }

    /// 列出 recorded_at 之后的新 trace（monitor 后台轮询水位线）。
    pub async fn list_traces_since(
        &self,
        project_id: &str,
        after: chrono::DateTime<chrono::Utc>,
        limit: i64,
    ) -> Result<Vec<Trace>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, project_id, agent_id, status, started_at, duration_ms
             FROM traces
             WHERE project_id = $1 AND recorded_at > $2
             ORDER BY recorded_at
             LIMIT $3",
        )
        .bind(project_id)
        .bind(after)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| Trace {
                id: r.get("id"),
                project_id: r.get("project_id"),
                agent_id: r.get("agent_id"),
                status: r.get("status"),
                started_at: r.get("started_at"),
                duration_ms: r.get("duration_ms"),
            })
            .collect())
    }

    /// 恢复观测汇总（FR-6）：失败 span 数、重试 span 数、恢复/失败 trace 数。
    /// 重试判定：span attributes->>'attempt' 为数值且 > 1。
    pub async fn recovery_stats(&self, project_id: &str) -> Result<serde_json::Value, sqlx::Error> {
        let row = sqlx::query(
            "SELECT
                    count(*) AS total_spans,
                    count(*) FILTER (WHERE s.status != 'ok') AS error_spans,
                    count(*) FILTER (WHERE (s.attributes->>'attempt')::int > 1) AS retry_spans,
                    count(DISTINCT t.id) FILTER (WHERE t.status = 'ok' AND EXISTS (
                        SELECT 1 FROM spans s2 WHERE s2.trace_id = t.id AND s2.status != 'ok'
                    )) AS recovered_traces,
                    count(DISTINCT t.id) FILTER (WHERE t.status != 'ok') AS failed_traces,
                    count(DISTINCT t.id) AS total_traces
             FROM spans s
             JOIN traces t ON t.id = s.trace_id
             WHERE t.project_id = $1",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(json!({
            "total_spans": row.get::<i64, _>("total_spans"),
            "error_spans": row.get::<i64, _>("error_spans"),
            "retry_spans": row.get::<i64, _>("retry_spans"),
            "recovered_traces": row.get::<i64, _>("recovered_traces"),
            "failed_traces": row.get::<i64, _>("failed_traces"),
            "total_traces": row.get::<i64, _>("total_traces"),
        }))
    }

    /// 按 agent 聚合恢复观测（FR-6）。
    pub async fn recovery_agents(
        &self,
        project_id: &str,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT coalesce(t.agent_id, 'unknown') AS agent,
                    count(*) FILTER (WHERE s.status != 'ok') AS error_spans,
                    count(*) FILTER (WHERE (s.attributes->>'attempt')::int > 1) AS retry_spans,
                    count(DISTINCT t.id) FILTER (WHERE t.status != 'ok') AS failed_traces,
                    count(DISTINCT t.id) AS total_traces
             FROM spans s
             JOIN traces t ON t.id = s.trace_id
             WHERE t.project_id = $1
             GROUP BY coalesce(t.agent_id, 'unknown')
             ORDER BY error_spans DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                json!({
                    "agent": r.get::<String, _>("agent"),
                    "error_spans": r.get::<i64, _>("error_spans"),
                    "retry_spans": r.get::<i64, _>("retry_spans"),
                    "failed_traces": r.get::<i64, _>("failed_traces"),
                    "total_traces": r.get::<i64, _>("total_traces"),
                })
            })
            .collect())
    }

    /// 列出含失败或重试 span 的 trace（FR-6 前端列表）。
    pub async fn recovery_traces(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<Trace>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT t.id, t.project_id, t.agent_id, t.status, t.started_at, t.duration_ms
             FROM traces t
             WHERE t.project_id = $1
               AND (
                    t.status != 'ok'
                    OR EXISTS (SELECT 1 FROM spans s WHERE s.trace_id = t.id AND s.status != 'ok')
                    OR EXISTS (SELECT 1 FROM spans s WHERE s.trace_id = t.id AND (s.attributes->>'attempt')::int > 1)
               )
             ORDER BY t.started_at DESC
             LIMIT $2",
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| Trace {
                id: r.get("id"),
                project_id: r.get("project_id"),
                agent_id: r.get("agent_id"),
                status: r.get("status"),
                started_at: r.get("started_at"),
                duration_ms: r.get("duration_ms"),
            })
            .collect())
    }

    // ---- alert（FR-8）：规则 CRUD + 事件列表 + 评估用查询 ----

    /// 列出某项目所有规则（含 disabled）。
    pub async fn list_alert_rules(
        &self,
        project_id: &str,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, metric, operator, threshold, enabled, last_triggered_at
             FROM alert_rules WHERE project_id = $1 ORDER BY created_at DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(rule_json).collect())
    }

    /// 创建规则。metric ∈ {error_rate, p95_ms, avg_ms}，operator ∈ {>, <}。
    pub async fn create_alert_rule(
        &self,
        project_id: &str,
        metric: &str,
        operator: &str,
        threshold: f64,
    ) -> Result<serde_json::Value, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO alert_rules (id, project_id, metric, operator, threshold)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&id)
        .bind(project_id)
        .bind(metric)
        .bind(operator)
        .bind(threshold)
        .execute(&self.pool)
        .await?;
        let row = sqlx::query(
            "SELECT id, metric, operator, threshold, enabled, last_triggered_at
             FROM alert_rules WHERE id = $1",
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await?;
        Ok(json!({
            "id": row.get::<String, _>("id"),
            "project_id": project_id,
            "metric": row.get::<String, _>("metric"),
            "operator": row.get::<String, _>("operator"),
            "threshold": row.get::<f64, _>("threshold"),
            "enabled": row.get::<bool, _>("enabled"),
            "last_triggered_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_triggered_at").map(|d| d.to_rfc3339()),
        }))
    }

    /// 删除规则。返回是否真的删了一行。
    pub async fn delete_alert_rule(
        &self,
        project_id: &str,
        rule_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query("DELETE FROM alert_rules WHERE id = $1 AND project_id = $2")
            .bind(rule_id)
            .bind(project_id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    /// 记录一次告警触发事件，同时刷新规则的 last_triggered_at。
    pub async fn record_alert_event(
        &self,
        project_id: &str,
        rule_id: &str,
        metric: &str,
        value: f64,
        threshold: f64,
    ) -> Result<(), sqlx::Error> {
        let event_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO alert_events (id, project_id, rule_id, metric, value, threshold)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&event_id)
        .bind(project_id)
        .bind(rule_id)
        .bind(metric)
        .bind(value)
        .bind(threshold)
        .execute(&self.pool)
        .await?;
        sqlx::query("UPDATE alert_rules SET last_triggered_at = now() WHERE id = $1")
            .bind(rule_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 列出告警触发事件。
    pub async fn list_alert_events(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT e.id, e.rule_id, e.metric, e.value, e.threshold, e.triggered_at,
                    r.metric AS rule_metric, r.operator AS rule_operator
             FROM alert_events e
             JOIN alert_rules r ON r.id = e.rule_id
             WHERE e.project_id = $1
             ORDER BY e.triggered_at DESC
             LIMIT $2",
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                json!({
                    "id": r.get::<String, _>("id"),
                    "rule_id": r.get::<String, _>("rule_id"),
                    "metric": r.get::<String, _>("metric"),
                    "value": r.get::<f64, _>("value"),
                    "threshold": r.get::<f64, _>("threshold"),
                    "operator": r.get::<String, _>("rule_operator"),
                    "triggered_at": r.get::<chrono::DateTime<chrono::Utc>, _>("triggered_at").to_rfc3339(),
                })
            })
            .collect())
    }

    /// 取启用的规则（alert 后台评估任务用）。
    pub async fn enabled_alert_rules(
        &self,
        project_id: &str,
    ) -> Result<Vec<(String, String, String, f64)>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, metric, operator, threshold FROM alert_rules
             WHERE project_id = $1 AND enabled = TRUE",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    r.get::<String, _>("id"),
                    r.get::<String, _>("metric"),
                    r.get::<String, _>("operator"),
                    r.get::<f64, _>("threshold"),
                )
            })
            .collect())
    }

    /// 评估告警规则：算指标值，超阈值则记事件。返回本次触发数。
    /// 冷却：规则 10 分钟内不重复触发（由 last_triggered_at 判断，调用方传 now）。
    pub async fn evaluate_alerts(
        &self,
        project_id: &str,
        now: chrono::DateTime<chrono::Utc>,
        cooldown: chrono::Duration,
    ) -> Result<usize, sqlx::Error> {
        let rules = self.enabled_alert_rules(project_id).await?;
        let mut triggered = 0;
        for (rule_id, metric, operator, threshold) in rules {
            // 冷却检查：last_triggered_at 距今不足 cooldown 则跳过。
            let last: Option<chrono::DateTime<chrono::Utc>> =
                sqlx::query_scalar("SELECT last_triggered_at FROM alert_rules WHERE id = $1")
                    .bind(&rule_id)
                    .fetch_optional(&self.pool)
                    .await?
                    .flatten();
            if let Some(t) = last {
                if now - t < cooldown {
                    continue;
                }
            }

            let value: Option<f64> = match metric.as_str() {
                "error_rate" => {
                    let row = sqlx::query(
                        "SELECT count(*) FILTER (WHERE status != 'ok')::float8 / NULLIF(count(*), 0)::float8 AS v
                         FROM traces WHERE project_id = $1",
                    )
                    .bind(project_id)
                    .fetch_one(&self.pool)
                    .await?;
                    row.get::<Option<f64>, _>("v")
                }
                "p95_ms" => {
                    let row = sqlx::query(
                        "SELECT percentile_cont(0.95) WITHIN GROUP (ORDER BY duration_ms) AS v
                         FROM traces WHERE project_id = $1",
                    )
                    .bind(project_id)
                    .fetch_one(&self.pool)
                    .await?;
                    row.get::<Option<f64>, _>("v")
                }
                "avg_ms" => {
                    let row = sqlx::query(
                        "SELECT avg(duration_ms)::float8 AS v FROM traces WHERE project_id = $1",
                    )
                    .bind(project_id)
                    .fetch_one(&self.pool)
                    .await?;
                    row.get::<Option<f64>, _>("v")
                }
                _ => None,
            };

            let Some(value) = value else { continue };
            let breached = match operator.as_str() {
                ">" => value > threshold,
                "<" => value < threshold,
                _ => false,
            };
            if breached {
                self.record_alert_event(project_id, &rule_id, &metric, value, threshold)
                    .await?;
                triggered += 1;
            }
        }
        Ok(triggered)
    }
}

fn rule_json(r: sqlx::postgres::PgRow) -> serde_json::Value {
    json!({
        "id": r.get::<String, _>("id"),
        "metric": r.get::<String, _>("metric"),
        "operator": r.get::<String, _>("operator"),
        "threshold": r.get::<f64, _>("threshold"),
        "enabled": r.get::<bool, _>("enabled"),
        "last_triggered_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_triggered_at").map(|d| d.to_rfc3339()),
    })
}

pub struct StorageModule;

impl super::Module for StorageModule {
    fn name(&self) -> &'static str {
        "storage"
    }
    fn router(&self) -> Router<AppState> {
        // 纯数据层，无对外路由；通过 Storage 服务供其他模块调用。
        Router::new()
    }
}
