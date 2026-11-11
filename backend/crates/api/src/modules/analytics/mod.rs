// analytics 模块：成功率 / 延迟 / 成本分析洞察（FR-5）。
// 汇总 + 延迟时序 + span 类型成本拆解。

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
pub struct AnalyticsQuery {
    pub project_id: String,
}

pub struct AnalyticsModule;

impl Module for AnalyticsModule {
    fn name(&self) -> &'static str {
        "analytics"
    }
    fn depends_on(&self) -> &'static [&'static str] {
        &["storage"]
    }
    fn router(&self) -> Router<AppState> {
        Router::new()
            .route("/summary", get(summary))
            .route("/latency", get(latency))
            .route("/cost", get(cost))
    }
}

async fn summary(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = iam::authorize(&headers, &params.project_id, &state).await?;
    let summary = state.storage.analytics_summary(&project_id).await?;
    Ok(Json(summary))
}

async fn latency(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let project_id = iam::authorize(&headers, &params.project_id, &state).await?;
    let series = state.storage.latency_timeseries(&project_id).await?;
    Ok(Json(series))
}

async fn cost(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let project_id = iam::authorize(&headers, &params.project_id, &state).await?;
    let breakdown = state.storage.cost_breakdown(&project_id).await?;
    Ok(Json(breakdown))
}
