// storage 模块：trace/span 的读写（FR-2）。
// 只封装 SQL，不暴露 web 层；其他模块通过 Storage 访问数据。

use axum::Router;
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
        Ok(rows.into_iter().map(|r| Span {
            id: r.get("id"),
            trace_id: r.get("trace_id"),
            parent_span_id: r.get("parent_span_id"),
            name: r.get("name"),
            kind: r.get("kind"),
            status: r.get("status"),
            started_at: r.get("started_at"),
            duration_ms: r.get("duration_ms"),
            attributes: r.get("attributes"),
        }).collect())
    }

    pub async fn list_traces(&self, project_id: &str, limit: i64) -> Result<Vec<Trace>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, project_id, agent_id, status, started_at, duration_ms
             FROM traces WHERE project_id = $1
             ORDER BY started_at DESC LIMIT $2",
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| Trace {
            id: r.get("id"),
            project_id: r.get("project_id"),
            agent_id: r.get("agent_id"),
            status: r.get("status"),
            started_at: r.get("started_at"),
            duration_ms: r.get("duration_ms"),
        }).collect())
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
