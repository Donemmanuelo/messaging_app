use crate::models::message::Message;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid;
use std::sync::Arc;
use crate::AppState;

#[derive(Deserialize)]
pub struct SendMessageInput {
    pub chat_id: uuid::Uuid,
    pub sender_id: uuid::Uuid,
    pub content: String,
    pub media_url: Option<String>,
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(input): Json<SendMessageInput>,
) -> impl IntoResponse {
    let msg = sqlx::query_as!(Message,
        "INSERT INTO messages (chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at",
        input.chat_id,
        input.sender_id,
        input.content,
        input.media_url,
        None::<String>,
        None::<uuid::Uuid>,
        Utc::now(),
        Utc::now()
    )
    .fetch_one(&state.pool)
    .await
    .unwrap();
    (StatusCode::CREATED, Json(msg))
}

pub async fn fetch_messages(
    State(state): State<Arc<AppState>>,
    Path(chat_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let messages = sqlx::query_as!(
        Message,
        "SELECT id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at FROM messages WHERE chat_id = $1 ORDER BY created_at ASC",
        chat_id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();
    Json(messages)
}
