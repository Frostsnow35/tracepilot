// query 模块：trace 查询（FR-3）。
// 列表 + 详情（含 span 树，供前端瀑布图渲染）。

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use super::{iam, Module};
use crate::error::AppError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    pub project_id: String,
    pub limit: Option<i64>,
}

pub struct QueryModule;

impl Module for QueryModule {
    fn name(&self) -> &'static str {
        "query"
    }
    fn depends_on(&self) -> &'static [&'static str] {
        &["storage"]
    }
    fn router(&self) -> Router<AppState> {
        Router::new()
            .route("/traces", get(list_traces))
            .route("/traces/:id", get(get_trace_detail))
    }
}

async fn list_traces(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<tracepilot_core::Trace>>, AppError> {
    iam::authenticate(&headers, &state)?;
    let limit = params.limit.unwrap_or(50).clamp(1, 500);
    let traces = state.storage.list_traces(&params.project_id, limit).await?;
    Ok(Json(traces))
}

async fn get_trace_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    iam::authenticate(&headers, &state)?;
    let trace = state
        .storage
        .get_trace(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("trace {id}")))?;
    let spans = state.storage.get_spans_by_trace(&id).await?;
    Ok(Json(json!({ "trace": trace, "spans": spans })))
}
