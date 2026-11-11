// alert 模块：阈值告警（FR-8）。
// 规则 CRUD + 触发事件列表；规则评估由 main.rs 后台任务周期执行。

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{delete, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use super::{iam, Module};
use crate::error::AppError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct AlertQuery {
    pub project_id: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Deserialize)]
pub struct CreateRuleReq {
    pub project_id: String,
    pub metric: String,
    pub operator: String,
    pub threshold: f64,
}

#[derive(Deserialize)]
pub struct UpdateRuleReq {
    pub project_id: String,
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct RuleResp {
    pub id: String,
    pub project_id: String,
    pub metric: String,
    pub operator: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<String>,
}

/// 校验规则字段：metric ∈ {error_rate, p95_ms, avg_ms}，operator ∈ {>, <}。
fn validate_rule(req: &CreateRuleReq) -> Result<(), AppError> {
    const METRICS: &[&str] = &["error_rate", "p95_ms", "avg_ms"];
    const OPERATORS: &[&str] = &["<", ">"];
    if !METRICS.contains(&req.metric.as_str()) {
        return Err(AppError::BadRequest(format!(
            "invalid metric: {} (allowed: {})",
            req.metric,
            METRICS.join(", ")
        )));
    }
    if !OPERATORS.contains(&req.operator.as_str()) {
        return Err(AppError::BadRequest(format!(
            "invalid operator: {} (allowed: {})",
            req.operator,
            OPERATORS.join(", ")
        )));
    }
    if req.threshold < 0.0 {
        return Err(AppError::BadRequest("threshold must be >= 0".into()));
    }
    Ok(())
}

pub struct AlertModule;

impl Module for AlertModule {
    fn name(&self) -> &'static str {
        "alert"
    }
    fn depends_on(&self) -> &'static [&'static str] {
        &["storage"]
    }
    fn router(&self) -> Router<AppState> {
        Router::new()
            .route("/rules", get(list_rules).post(create_rule))
            .route("/rules/:id", delete(delete_rule).patch(update_rule))
            .route("/events", get(list_events))
    }
}

async fn list_rules(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AlertQuery>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let project_id = iam::authorize(&headers, &params.project_id, &state).await?;
    let rules = state.storage.list_alert_rules(&project_id).await?;
    Ok(Json(rules))
}

async fn create_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateRuleReq>,
) -> Result<Json<RuleResp>, AppError> {
    let project_id = iam::authorize(&headers, &req.project_id, &state).await?;
    validate_rule(&req)?;
    let rule = state
        .storage
        .create_alert_rule(&project_id, &req.metric, &req.operator, req.threshold)
        .await?;
    Ok(Json(RuleResp {
        id: rule
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        project_id: req.project_id,
        metric: req.metric,
        operator: req.operator,
        threshold: req.threshold,
        enabled: rule
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        last_triggered_at: rule
            .get("last_triggered_at")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    }))
}

async fn delete_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AlertQuery>,
    Path(rule_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = iam::authorize(&headers, &params.project_id, &state).await?;
    let deleted = state
        .storage
        .delete_alert_rule(&project_id, &rule_id)
        .await?;
    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err(AppError::NotFound("alert rule not found".into()))
    }
}

async fn update_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(rule_id): Path<String>,
    Json(req): Json<UpdateRuleReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = iam::authorize(&headers, &req.project_id, &state).await?;
    let updated = state
        .storage
        .set_alert_rule_enabled(&project_id, &rule_id, req.enabled)
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("alert rule not found".into()));
    }
    Ok(Json(
        serde_json::json!({"updated": true, "enabled": req.enabled}),
    ))
}

async fn list_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AlertQuery>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let project_id = iam::authorize(&headers, &params.project_id, &state).await?;
    let events = state
        .storage
        .list_alert_events(&project_id, params.limit)
        .await?;
    Ok(Json(events))
}
