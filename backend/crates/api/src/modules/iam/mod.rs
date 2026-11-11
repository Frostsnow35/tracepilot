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

/// 校验请求头中的 API Key，返回所属项目 id（FR-7：API Key + Project 隔离）。
/// Key 经 SHA-256 后与 projects.api_key_hash 比对；调用方须用 authorize()
/// 核对请求携带的 project_id 与此返回值一致，实现项目级隔离。
pub async fn authenticate(headers: &HeaderMap, state: &AppState) -> Result<String, AppError> {
    let key = headers
        .get(&state.config.api_key_header)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;
    authenticate_key(key, state).await
}

/// 从原始 Key 字符串鉴权（WebSocket 场景：浏览器无法设自定义 header，Key 走 query）。
pub async fn authenticate_key(key: &str, state: &AppState) -> Result<String, AppError> {
    if key.trim().is_empty() {
        return Err(AppError::Unauthorized);
    }
    let hash = hex::encode(Sha256::digest(key.as_bytes()));
    let project_id: Option<String> =
        sqlx::query_scalar("SELECT id FROM projects WHERE api_key_hash = $1")
            .bind(&hash)
            .fetch_optional(state.storage.pool())
            .await?;
    project_id.ok_or(AppError::Unauthorized)
}

/// 鉴权 + 项目隔离校验：Key 合法且请求 project_id 属于该 Key 所属项目时返回 project_id。
pub async fn authorize(
    headers: &HeaderMap,
    requested_project: &str,
    state: &AppState,
) -> Result<String, AppError> {
    let project_id = authenticate(headers, state).await?;
    if requested_project != project_id {
        return Err(AppError::Unauthorized);
    }
    Ok(project_id)
}
