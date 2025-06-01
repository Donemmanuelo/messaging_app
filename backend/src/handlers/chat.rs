use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    auth::AuthUser,
    error::AppError,
    models::{Chat, ChatMessage, CreateChatRequest, SendMessageRequest},
};

pub async fn get_chats(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let chats = sqlx::query_as!(
        Chat,
        r#"
        SELECT c.*, 
            (SELECT json_agg(json_build_object(
                'id', u.id,
                'username', u.username,
                'email', u.email
            ))
            FROM users u
            JOIN chat_participants cp ON cp.user_id = u.id
            WHERE cp.chat_id = c.id) as participants
        FROM chats c
        JOIN chat_participants cp ON cp.chat_id = c.id
        WHERE cp.user_id = $1
        ORDER BY c.updated_at DESC
        "#,
        auth_user.id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(chats))
}

pub async fn get_chat(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(chat_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let chat = sqlx::query_as!(
        Chat,
        r#"
        SELECT c.*, 
            (SELECT json_agg(json_build_object(
                'id', u.id,
                'username', u.username,
                'email', u.email
            ))
            FROM users u
            JOIN chat_participants cp ON cp.user_id = u.id
            WHERE cp.chat_id = c.id) as participants
        FROM chats c
        WHERE c.id = $1
        "#,
        chat_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Chat not found".into()))?;

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

    Ok(Json(chat))
}

pub async fn create_chat(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Json(req): Json<CreateChatRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Start transaction
    let mut tx = pool.begin().await?;

    // Create chat
    let chat = sqlx::query_as!(
        Chat,
        r#"
        INSERT INTO chats (name, is_group)
        VALUES ($1, $2)
        RETURNING *
        "#,
        req.name,
        req.is_group
    )
    .fetch_one(&mut tx)
    .await?;

    // Add participants
    for user_id in req.participant_ids {
        sqlx::query!(
            "INSERT INTO chat_participants (chat_id, user_id) VALUES ($1, $2)",
            chat.id,
            user_id
        )
        .execute(&mut tx)
        .await?;
    }

    // Add creator as participant
    sqlx::query!(
        "INSERT INTO chat_participants (chat_id, user_id) VALUES ($1, $2)",
        chat.id,
        auth_user.id
    )
    .execute(&mut tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(chat)))
}

pub async fn get_messages(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(chat_id): Path<i32>,
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

    let messages = sqlx::query_as!(
        ChatMessage,
        r#"
        SELECT m.*,
            json_build_object(
                'id', u.id,
                'username', u.username,
                'email', u.email
            ) as sender,
            (SELECT json_agg(json_build_object(
                'id', r.id,
                'emoji', r.emoji,
                'count', (SELECT COUNT(*) FROM message_reactions WHERE message_id = m.id AND emoji = r.emoji),
                'users', (SELECT json_agg(json_build_object(
                    'id', u.id,
                    'username', u.username
                ))
                FROM users u
                JOIN message_reactions mr ON mr.user_id = u.id
                WHERE mr.message_id = m.id AND mr.emoji = r.emoji)
            ))
            FROM (
                SELECT DISTINCT id, emoji
                FROM message_reactions
                WHERE message_id = m.id
            ) r) as reactions,
            (SELECT json_agg(json_build_object(
                'id', u.id,
                'username', u.username
            ))
            FROM users u
            JOIN message_reads mr ON mr.user_id = u.id
            WHERE mr.message_id = m.id) as read_by
        FROM messages m
        JOIN users u ON u.id = m.sender_id
        WHERE m.chat_id = $1
        ORDER BY m.created_at DESC
        LIMIT 50
        "#,
        chat_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(messages))
}

pub async fn send_message(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(chat_id): Path<i32>,
    Json(req): Json<SendMessageRequest>,
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

    // Create message
    let message = sqlx::query_as!(
        ChatMessage,
        r#"
        INSERT INTO messages (chat_id, sender_id, content, media_type, reply_to_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
        chat_id,
        auth_user.id,
        req.content,
        req.media_type as _,
        req.reply_to_id
    )
    .fetch_one(&pool)
    .await?;

    // Update chat's updated_at
    sqlx::query!(
        "UPDATE chats SET updated_at = NOW() WHERE id = $1",
        chat_id
    )
    .execute(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(message)))
}

pub async fn delete_message(
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

    // Check if user is the sender
    let is_sender = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM messages WHERE id = $1 AND sender_id = $2)",
        message_id,
        auth_user.id
    )
    .fetch_one(&pool)
    .await?
    .exists;

    if !is_sender {
        return Err(AppError::Forbidden("Not the sender of this message".into()));
    }

    // Delete message
    sqlx::query!("DELETE FROM messages WHERE id = $1", message_id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
} 