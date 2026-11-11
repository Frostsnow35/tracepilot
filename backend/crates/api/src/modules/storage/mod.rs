// storage 模块：trace/span 的读写（FR-2）。
// 只封装 SQL，不暴露 web 层；其他模块通过 Storage 访问数据。

use axum::Router;
use serde_json::json;
use sqlx::{PgPool, Row};
use tracepilot_core::{Span, Trace};

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
                    avg(duration_ms) AS avg_ms,
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
                    avg(duration_ms) AS avg_ms,
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
                    avg(duration_ms) AS avg_ms,
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
                    avg(duration_ms) AS avg_ms
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
            "SELECT date_trunc('hour', started_at::timestamp) AS bucket,
                    count(*) AS total,
                    avg(duration_ms) AS avg_ms,
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
                    sum(duration_ms) AS total_ms,
                    avg(duration_ms) AS avg_ms
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
