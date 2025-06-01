use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    models::{ForwardMessageRequest, MessageActionResponse},
    auth::Claims,
    error::AppError,
    models::{Message, MessageRead},
    auth::AuthUser,
    models::{ChatMessage},
};

#[derive(Debug, Deserialize)]
pub struct ForwardMessageRequest {
    message_ids: Vec<Uuid>,
    target_chat_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct MessageReadResponse {
    id: Uuid,
    user_id: Uuid,
    read_at: chrono::DateTime<Utc>,
}

pub async fn delete_message(
    State(pool): State<PgPool>,
    claims: Claims,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let message = sqlx::query_as!(
        Message,
        "SELECT * FROM messages WHERE id = $1",
        message_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound("Message not found".into()))?;

    if message.sender_id != claims.user_id {
        return Err(AppError::Forbidden("Cannot delete another user's message".into()));
    }

    sqlx::query!(
        "UPDATE messages SET deleted_at = $1 WHERE id = $2",
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

    for message_id in req.message_ids {
        let message = sqlx::query_as!(
            Message,
            "SELECT * FROM messages WHERE id = $1 AND deleted_at IS NULL",
            message_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("Message not found".into()))?;

        sqlx::query!(
            "INSERT INTO messages (chat_id, sender_id, content, media_url, media_type, reply_to_id)
             VALUES ($1, $2, $3, $4, $5, $6)",
            req.target_chat_id,
            claims.user_id,
            message.content,
            message.media_url,
            message.media_type,
            message.reply_to_id
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
    sqlx::query!(
        "INSERT INTO message_reads (message_id, user_id)
         VALUES ($1, $2)
         ON CONFLICT (message_id, user_id) DO NOTHING",
        message_id,
        claims.user_id
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
    let message = sqlx::query_as!(
        Message,
        "SELECT * FROM messages WHERE id = $1",
        message_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound("Message not found".into()))?;

    // Check if user has access to the chat
    let chat_access = sqlx::query!(
        "SELECT 1 FROM chat_participants
         WHERE chat_id = $1 AND user_id = $2",
        message.chat_id,
        claims.user_id
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
    Path((chat_id, message_id)): Path<(i32, i32)>,
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

    if !is_participant {
        return Err(AppError::Forbidden("Not a participant in source chat".into()));
    }

    // Check if user is a participant in target chat
    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2)",
        req.target_chat_id,
        auth_user.id
    )
    .fetch_one(&pool)
    .await?
    .exists;

    if !is_participant {
        return Err(AppError::Forbidden("Not a participant in target chat".into()));
    }

    // Get original message
    let original_message = sqlx::query_as!(
        ChatMessage,
        r#"
        SELECT m.*,
            json_build_object(
                'id', u.id,
                'username', u.username,
                'email', u.email
            ) as sender
        FROM messages m
        JOIN users u ON u.id = m.sender_id
        WHERE m.id = $1
        "#,
        message_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Message not found".into()))?;

    // Create forwarded message
    let message = sqlx::query_as!(
        ChatMessage,
        r#"
        INSERT INTO messages (chat_id, sender_id, content, media_type, reply_to_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
        req.target_chat_id,
        auth_user.id,
        original_message.content,
        original_message.media_type,
        None::<i32>
    )
    .fetch_one(&pool)
    .await?;

    // Update target chat's updated_at
    sqlx::query!(
        "UPDATE chats SET updated_at = NOW() WHERE id = $1",
        req.target_chat_id
    )
    .execute(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(message)))
}

pub async fn mark_as_read(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path((chat_id, message_id)): Path<(i32, i32)>,
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

    if !is_participant {
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
    Path((chat_id, message_id)): Path<(i32, i32)>,
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

    if !is_participant {
        return Err(AppError::Forbidden("Not a participant in this chat".into()));
    }

    // Get read receipts
    let read_receipts = sqlx::query!(
        r#"
        SELECT 
            u.id,
            u.username,
            mr.created_at as read_at
        FROM message_reads mr
        JOIN users u ON u.id = mr.user_id
        WHERE mr.message_id = $1
        ORDER BY mr.created_at ASC
        "#,
        message_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(read_receipts))
} 