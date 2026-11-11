// monitor 模块：Agent 实时运行状态监控（FR-4）。
// REST：统计概览 + 按 agent 聚合。
// WebSocket：订阅新 trace 事件实时推送（由 main.rs 后台轮询任务广播）。

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use super::{iam, Module};
use crate::error::AppError;
use crate::state::AppState;

/// WebSocket 推送的新 trace 事件。
#[derive(Debug, Clone, Serialize)]
pub struct TraceEvent {
    pub trace_id: String,
    pub project_id: String,
    pub agent_id: Option<String>,
    pub status: String,
    pub started_at: String,
    pub duration_ms: i64,
}

#[derive(Deserialize)]
pub struct MonitorQuery {
    pub project_id: String,
    /// WebSocket 无法设自定义 header，浏览器从 query 传 api_key。
    pub api_key: Option<String>,
}

pub struct MonitorModule;

impl Module for MonitorModule {
    fn name(&self) -> &'static str {
        "monitor"
    }
    fn depends_on(&self) -> &'static [&'static str] {
        &["storage"]
    }
    fn router(&self) -> Router<AppState> {
        Router::new()
            .route("/stats", get(stats))
            .route("/agents", get(agents))
            .route("/stream", get(stream))
    }
}

async fn stats(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<MonitorQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = iam::authorize(&headers, &params.project_id, &state).await?;
    let stats = state.storage.monitor_stats(&project_id).await?;
    Ok(Json(stats))
}

async fn agents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<MonitorQuery>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let project_id = iam::authorize(&headers, &params.project_id, &state).await?;
    let agents = state.storage.agent_stats(&project_id).await?;
    Ok(Json(agents))
}

async fn stream(
    State(state): State<AppState>,
    Query(params): Query<MonitorQuery>,
    ws: WebSocketUpgrade,
) -> Result<axum::response::Response, AppError> {
    // WebSocket 握手无法携带自定义 header，鉴权改为 query 传 api_key。
    let key = params.api_key.as_deref().ok_or(AppError::Unauthorized)?;
    let project_id = iam::authenticate_key(key, &state).await?;
    // 项目隔离：订阅的 project_id 必须属于该 Key。
    if params.project_id != project_id {
        return Err(AppError::Unauthorized);
    }
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, params.project_id)))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, project_id: String) {
    let mut rx = state.monitor_tx.subscribe();
    loop {
        match rx.recv().await {
            Ok(event) => {
                if event.project_id != project_id {
                    continue;
                }
                let text = serde_json::to_string(&event).unwrap_or_default();
                if socket.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}
