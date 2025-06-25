use crate::{
    auth::AuthUser,
    auth::Claims,
    error::AppError,
    models::ChatMessage,
    models::ForwardMessageRequest,
    models::message::Message,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde:: Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MessageReadResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub read_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ReadReceiptResponse {
    pub id: Uuid,
    pub username: String,
    pub read_at: chrono::DateTime<Utc>,
}

pub async fn delete_message(
    State(pool): State<PgPool>,
    claims: Claims,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let message = sqlx::query_as!(Message, "SELECT * FROM messages WHERE id = $1", message_id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound("Message not found".into()))?;

    let user_id = Uuid::parse_str(&claims.sub)?;

    if message.sender_id != user_id {
        return Err(AppError::Forbidden(
            "Cannot delete another user's message".into(),
        ));
    }

    sqlx::query!(
        "UPDATE messages SET updated_at = $1 WHERE id = $2",
        Utc::now(),
        message_id
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn forward_messages(
    State(pool): State<PgPool>,
    claims: Claims,
    Json(req): Json<ForwardMessageRequest>,
) -> Result<StatusCode, AppError> {
    let mut tx = pool.begin().await?;

    let user_id = Uuid::parse_str(&claims.sub)?;

    for chat_id in req.chat_ids.iter() {
        let message = sqlx::query_as!(
            Message,
            "SELECT * FROM messages WHERE id = $1",
            req.message_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("Message not found".into()))?;

        sqlx::query!(
            "INSERT INTO messages (chat_id, sender_id, content, media_url)
             VALUES ($1, $2, $3, $4)",
            chat_id,
            user_id,
            message.content,
            message.media_url
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(StatusCode::CREATED)
}

pub async fn mark_message_as_read(
    State(pool): State<PgPool>,
    claims: Claims,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)?;

    sqlx::query!(
        "INSERT INTO message_reads (message_id, user_id, read_at)
         VALUES ($1, $2, NOW())
         ON CONFLICT (message_id, user_id) DO UPDATE SET read_at = NOW()",
        message_id,
        user_id
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_message_reads(
    State(pool): State<PgPool>,
    claims: Claims,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Vec<MessageReadResponse>>, AppError> {
    let message = sqlx::query_as!(Message, "SELECT * FROM messages WHERE id = $1", message_id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound("Message not found".into()))?;

    // Check if user has access to the chat
    let chat_access = sqlx::query!(
        "SELECT 1 as exists FROM chat_participants\n         WHERE chat_id = $1 AND user_id = $2",
        message.chat_id,
        Uuid::parse_str(&claims.sub)?
    )
    .fetch_optional(&pool)
    .await?;

    if chat_access.is_none() {
        return Err(AppError::Forbidden("No access to this chat".into()));
    }

    let reads = sqlx::query_as!(
        MessageReadResponse,
        "SELECT id, user_id, read_at
         FROM message_reads
         WHERE message_id = $1
         ORDER BY read_at ASC",
        message_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(reads))
}

pub async fn forward_message(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path((chat_id, message_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ForwardMessageRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check if user is a participant in source chat
    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2)",
        chat_id,
        auth_user.id
    )
    .fetch_one(&pool)
    .await?
    .exists;

    if !is_participant.unwrap_or(false) {
        return Err(AppError::Forbidden(
            "Not a participant in source chat".into(),
        ));
    }

    // Check if user is a participant in target chat (first chat_id in req.chat_ids)
    let target_chat_id = req
        .chat_ids
        .get(0)
        .ok_or(AppError::BadRequest("No target chat_id provided".into()))?;
    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2)",
        target_chat_id,
        auth_user.id
    )
    .fetch_one(&pool)
    .await?
    .exists;

    if !is_participant.unwrap_or(false) {
        return Err(AppError::Forbidden(
            "Not a participant in target chat".into(),
        ));
    }

    // Get original message
    let original_message = sqlx::query_as!(
        ChatMessage,
        r#"
        SELECT id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at
        FROM messages
        WHERE id = $1
        "#,
        req.message_id
    )
    .fetch_one(&pool)
    .await?;

    // Insert forwarded message into target chat
    sqlx::query!(
        "INSERT INTO messages (chat_id, sender_id, content, media_url)
         VALUES ($1, $2, $3, $4)",
        target_chat_id,
        auth_user.id,
        original_message.content,
        original_message.media_url
    )
    .execute(&pool)
    .await?;

    Ok((StatusCode::CREATED, "Message forwarded"))
}

pub async fn mark_as_read(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path((chat_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    // Check if user is a participant
    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2)",
        chat_id,
        auth_user.id
    )
    .fetch_one(&pool)
    .await?
    .exists;

    if !is_participant.unwrap_or(false) {
        return Err(AppError::Forbidden("Not a participant in this chat".into()));
    }

    // Mark message as read
    sqlx::query!(
        r#"
        INSERT INTO message_reads (message_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT (message_id, user_id) DO NOTHING
        "#,
        message_id,
        auth_user.id
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_read_receipts(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path((chat_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    // Check if user is a participant
    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2)",
        chat_id,
        auth_user.id
    )
    .fetch_one(&pool)
    .await?
    .exists;

    if !is_participant.unwrap_or(false) {
        return Err(AppError::Forbidden("Not a participant in this chat".into()));
    }

    // Get read receipts
    let read_receipts = sqlx::query_as!(
        ReadReceiptResponse,
        r#"
        SELECT 
            u.id as id,
            u.username as username,
            mr.read_at as read_at
        FROM message_reads mr
        JOIN users u ON u.id = mr.user_id
        WHERE mr.message_id = $1
        ORDER BY mr.read_at ASC
        "#,
        message_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(read_receipts))
}
