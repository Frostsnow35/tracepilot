// recovery 模块：重试/故障恢复策略观测（FR-6）。
// 统计失败 span、重试 span、恢复/失败 trace，按 agent 聚合。

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use super::{iam, Module};
use crate::error::AppError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct RecoveryQuery {
    pub project_id: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

pub struct RecoveryModule;

impl Module for RecoveryModule {
    fn name(&self) -> &'static str {
        "recovery"
    }
    fn depends_on(&self) -> &'static [&'static str] {
        &["storage"]
    }
    fn router(&self) -> Router<AppState> {
        Router::new()
            .route("/stats", get(stats))
            .route("/agents", get(agents))
            .route("/traces", get(traces))
    }
}

async fn stats(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<RecoveryQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    iam::authenticate(&headers, &state)?;
    let stats = state.storage.recovery_stats(&params.project_id).await?;
    Ok(Json(stats))
}

async fn agents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<RecoveryQuery>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    iam::authenticate(&headers, &state)?;
    let agents = state.storage.recovery_agents(&params.project_id).await?;
    Ok(Json(agents))
}

async fn traces(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<RecoveryQuery>,
) -> Result<Json<Vec<tracepilot_core::Trace>>, AppError> {
    iam::authenticate(&headers, &state)?;
    let traces = state
        .storage
        .recovery_traces(&params.project_id, params.limit)
        .await?;
    Ok(Json(traces))
}
