// iam 模块：项目/API Key 管理（FR-7）+ 接入鉴权。

use axum::{extract::State, http::HeaderMap, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::Module;
use crate::error::AppError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateProjectReq {
    pub name: String,
}

#[derive(Serialize)]
pub struct ProjectResp {
    pub id: String,
    pub name: String,
    pub api_key: String,
}

pub struct IamModule;

impl Module for IamModule {
    fn name(&self) -> &'static str {
        "iam"
    }
    fn router(&self) -> Router<AppState> {
        Router::new().route("/projects", post(create_project))
    }
}

async fn create_project(
    State(state): State<AppState>,
    Json(body): Json<CreateProjectReq>,
) -> Result<Json<ProjectResp>, AppError> {
    let id = Uuid::new_v4().to_string();
    let api_key = format!("tp_{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let hash = hex::encode(Sha256::digest(api_key.as_bytes()));

    sqlx::query("INSERT INTO projects (id, name, api_key_hash) VALUES ($1, $2, $3)")
        .bind(&id)
        .bind(&body.name)
        .bind(&hash)
        .execute(state.storage.pool())
        .await?;

    tracing::info!(project_id = %id, "project created");
    Ok(Json(ProjectResp {
        id,
        name: body.name,
        api_key,
    }))
}

/// 校验请求头中的 API Key。MVP 只校验存在性与非空；
/// 后续接入项目隔离时改为哈希比对 + 项目上下文注入。
pub fn authenticate(headers: &HeaderMap, state: &AppState) -> Result<(), AppError> {
    let key = headers
        .get(&state.config.api_key_header)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;
    if key.trim().is_empty() {
        return Err(AppError::Unauthorized);
    }
    Ok(())
}
