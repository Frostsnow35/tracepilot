// ingest 模块：接收 trace/span（FR-1）。
// 链路：鉴权 → 批量校验 → 写 trace → 写 spans → 返回 trace_id。

use axum::{extract::State, http::HeaderMap, routing::post, Json, Router};
use serde::Deserialize;
use tracepilot_core::{Span, Trace};

use super::{iam, Module};
use crate::error::AppError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct IngestPayload {
    pub trace: Trace,
    #[serde(default)]
    pub spans: Vec<Span>,
}

pub struct IngestModule;

impl Module for IngestModule {
    fn name(&self) -> &'static str {
        "ingest"
    }
    fn depends_on(&self) -> &'static [&'static str] {
        &["storage", "iam"]
    }
    fn router(&self) -> Router<AppState> {
        Router::new().route("/traces", post(create_trace))
    }
}

async fn create_trace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<IngestPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 鉴权 + 项目隔离：trace 的 project_id 必须属于该 API Key。
    // insert_trace 用 body.trace 自带 project_id，authorize 已校验其与 Key 所属项目一致。
    iam::authorize(&headers, &body.trace.project_id, &state).await?;

    if body.spans.len() > state.config.ingest_batch_max {
        return Err(AppError::BadRequest(format!(
            "spans {} exceeds batch max {}",
            body.spans.len(),
            state.config.ingest_batch_max
        )));
    }

    state
        .storage
        .insert_trace_with_spans(&body.trace, &body.spans)
        .await?;

    tracing::info!(
        trace_id = %body.trace.id,
        spans = body.spans.len(),
        "trace ingested"
    );
    Ok(Json(serde_json::json!({
        "trace_id": body.trace.id,
        "spans": body.spans.len()
    })))
}
