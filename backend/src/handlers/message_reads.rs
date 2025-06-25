use axum::{Json, extract::{State, Path}, response::IntoResponse, http::StatusCode};
use serde::Deserialize;
use sqlx::PgPool;
use crate::models::message::MessageRead;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use crate::state::AppState;
use crate::error::AppError;

#[derive(Deserialize)]
pub struct MarkReadInput {
    pub message_id: Uuid,
    pub user_id: Uuid,
}

pub async fn mark_read(
    State(state): State<Arc<AppState>>,
    Json(input): Json<MarkReadInput>,
) -> Result<impl IntoResponse, AppError> {
    let read = sqlx::query_as!(MessageRead,
        "INSERT INTO message_reads (message_id, user_id, read_at) VALUES ($1, $2, $3) ON CONFLICT (message_id, user_id) DO UPDATE SET read_at = $3 RETURNING id, message_id, user_id, read_at",
        input.message_id, input.user_id, Utc::now()
    )
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::OK, Json(read)))
}

pub async fn get_reads(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let reads = sqlx::query_as!(MessageRead,
        "SELECT * FROM message_reads WHERE message_id = $1",
        message_id
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(reads))
} 