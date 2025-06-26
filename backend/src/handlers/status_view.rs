use axum::{extract::{State, Path, Json}, response::IntoResponse, http::StatusCode};
use std::sync::Arc;
use crate::AppState;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct MarkStatusSeenRequest {
    pub status_id: Uuid,
    pub user_id: Uuid,
}

pub async fn mark_status_seen(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MarkStatusSeenRequest>,
) -> impl IntoResponse {
    let _ = sqlx::query!(
        "INSERT INTO status_views (status_id, user_id) VALUES ($1, $2) ON CONFLICT (status_id, user_id) DO NOTHING",
        req.status_id, req.user_id
    )
    .execute(&state.pool)
    .await;
    StatusCode::NO_CONTENT
}

pub async fn get_status_views(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let views = sqlx::query!(
        "SELECT status_id FROM status_views WHERE user_id = $1",
        user_id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let status_ids: Vec<Uuid> = views.into_iter().filter_map(|v| v.status_id).collect();
    Json(status_ids)
} 