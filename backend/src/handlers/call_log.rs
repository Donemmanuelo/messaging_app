use axum::{extract::{State, Path, Json}, response::IntoResponse, http::StatusCode};
use std::sync::Arc;
use crate::{AppState};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct LogCallRequest {
    pub caller_id: Uuid,
    pub callee_id: Uuid,
    pub call_type: String,
    pub status: String,
    pub group_id: Option<Uuid>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct CallLogResponse {
    pub id: i32,
    pub caller_id: Option<Uuid>,
    pub caller_name: Option<String>,
    pub caller_avatar: Option<String>,
    pub callee_id: Option<Uuid>,
    pub callee_name: Option<String>,
    pub callee_avatar: Option<String>,
    pub call_type: String,
    pub started_at: chrono::NaiveDateTime,
    pub ended_at: Option<chrono::NaiveDateTime>,
    pub status: String,
    pub group_id: Option<Uuid>,
}

pub async fn log_call(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LogCallRequest>,
) -> impl IntoResponse {
    let started_at = req.started_at.unwrap_or_else(chrono::Utc::now).naive_utc();
    let ended_at = req.ended_at.map(|dt| dt.naive_utc());
    let _ = sqlx::query!(
        "INSERT INTO call_logs (caller_id, callee_id, call_type, status, group_id, started_at, ended_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        req.caller_id, req.callee_id, req.call_type, req.status, req.group_id, started_at, ended_at
    )
    .execute(&state.pool)
    .await;
    StatusCode::NO_CONTENT
}

pub async fn get_call_history(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let calls = sqlx::query_as!(
        CallLogResponse,
        r#"
        SELECT
            cl.id,
            cl.caller_id,
            caller.display_name as caller_name,
            caller.avatar_url as caller_avatar,
            cl.callee_id,
            callee.display_name as callee_name,
            callee.avatar_url as callee_avatar,
            cl.call_type,
            cl.started_at,
            cl.ended_at,
            cl.status,
            cl.group_id
        FROM call_logs cl
        LEFT JOIN users caller ON cl.caller_id = caller.id
        LEFT JOIN users callee ON cl.callee_id = callee.id
        WHERE cl.caller_id = $1 OR cl.callee_id = $1
        ORDER BY cl.started_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    Json(calls)
} 