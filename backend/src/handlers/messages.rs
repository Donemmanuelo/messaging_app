use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    AppState,
    error::AppError,
    models::{
        message::{
            Message, MessageResponse, GroupMessageResponse,
            CreateMessageRequest, UpdateMessageRequest,
        },
    },
    auth::Claims,
};
use std::sync::Arc;

const MAX_MESSAGE_LENGTH: usize = 4000;

#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    pub before: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(receiver_id): Path<Uuid>,
    Json(req): Json<CreateMessageRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    // Input validation
    if req.content.trim().is_empty() {
        return Err(AppError::BadRequest("Message content cannot be empty".into()));
    }
    if req.content.len() > MAX_MESSAGE_LENGTH {
        return Err(AppError::BadRequest(format!(
            "Message content exceeds maximum length of {} characters",
            MAX_MESSAGE_LENGTH
        )));
    }

    let message_id = Uuid::new_v4();

    // Save message to database
    let message = sqlx::query_as!(
        Message,
        r#"
        INSERT INTO messages (id, sender_id, receiver_id, content, media_url, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        RETURNING *
        "#,
        message_id,
        claims.sub,
        receiver_id,
        req.content,
        req.media_url
    )
    .fetch_one(&state.pool)
    .await?;

    // Get sender info
    let sender = sqlx::query!(
        r#"
        SELECT display_name, avatar_url
        FROM users
        WHERE id = $1
        "#,
        claims.sub
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(MessageResponse {
        id: message.id,
        sender_id: message.sender_id,
        receiver_id: message.receiver_id,
        content: message.content,
        media_url: message.media_url,
        created_at: message.created_at,
        updated_at: message.updated_at,
        is_edited: message.is_edited,
        is_deleted: message.is_deleted,
        sender_name: sender.display_name,
        sender_avatar: sender.avatar_url,
    }))
}

pub async fn send_group_message(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(group_id): Path<Uuid>,
    Json(req): Json<CreateMessageRequest>,
) -> Result<Json<GroupMessageResponse>, AppError> {
    // Input validation
    if req.content.trim().is_empty() {
        return Err(AppError::BadRequest("Message content cannot be empty".into()));
    }
    if req.content.len() > MAX_MESSAGE_LENGTH {
        return Err(AppError::BadRequest(format!(
            "Message content exceeds maximum length of {} characters",
            MAX_MESSAGE_LENGTH
        )));
    }

    // Verify user is a member of the group
    let is_member = sqlx::query!(
        r#"
        SELECT 1 FROM group_members
        WHERE group_id = $1 AND user_id = $2
        "#,
        group_id,
        claims.sub
    )
    .fetch_optional(&state.pool)
    .await?
    .is_some();

    if !is_member {
        return Err(AppError::Forbidden("Not a member of this group".into()));
    }

    let message_id = Uuid::new_v4();

    // Save message to database
    let message = sqlx::query_as!(
        Message,
        r#"
        INSERT INTO messages (id, sender_id, receiver_id, content, media_url, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        RETURNING *
        "#,
        message_id,
        claims.sub,
        group_id,
        req.content,
        req.media_url
    )
    .fetch_one(&state.pool)
    .await?;

    // Get sender info
    let sender = sqlx::query!(
        r#"
        SELECT display_name, avatar_url
        FROM users
        WHERE id = $1
        "#,
        claims.sub
    )
    .fetch_one(&state.pool)
    .await?;

    // Get group info
    let group = sqlx::query!(
        r#"
        SELECT name, avatar_url
        FROM groups
        WHERE id = $1
        "#,
        group_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(GroupMessageResponse {
        id: message.id,
        sender_id: message.sender_id,
        group_id: message.receiver_id,
        content: message.content,
        media_url: message.media_url,
        created_at: message.created_at,
        updated_at: message.updated_at,
        is_edited: message.is_edited,
        is_deleted: message.is_deleted,
        sender_name: sender.display_name,
        sender_avatar: sender.avatar_url,
        group_name: group.name,
        group_avatar: group.avatar_url,
    }))
}

pub async fn get_messages(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(receiver_id): Path<Uuid>,
    Query(query): Query<MessageQuery>,
) -> Result<Json<Vec<MessageResponse>>, AppError> {
    let messages = sqlx::query!(
        r#"
        SELECT 
            m.*,
            u.display_name as sender_name,
            u.avatar_url as sender_avatar
        FROM messages m
        JOIN users u ON u.id = m.sender_id
        WHERE 
            ((m.sender_id = $1 AND m.receiver_id = $2) OR
            (m.sender_id = $2 AND m.receiver_id = $1))
            AND ($3::timestamptz IS NULL OR m.created_at < $3)
        ORDER BY m.created_at DESC
        LIMIT $4
        "#,
        claims.sub,
        receiver_id,
        query.before,
        query.limit.unwrap_or(50)
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(messages.into_iter().map(|m| MessageResponse {
        id: m.id,
        sender_id: m.sender_id,
        receiver_id: m.receiver_id,
        content: m.content,
        media_url: m.media_url,
        created_at: m.created_at,
        updated_at: m.updated_at,
        is_edited: m.is_edited,
        is_deleted: m.is_deleted,
        sender_name: m.sender_name,
        sender_avatar: m.sender_avatar,
    }).collect()))
}

pub async fn get_group_messages(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(group_id): Path<Uuid>,
    Query(query): Query<MessageQuery>,
) -> Result<Json<Vec<GroupMessageResponse>>, AppError> {
    // Verify user is a member of the group
    let is_member = sqlx::query!(
        r#"
        SELECT 1 FROM group_members
        WHERE group_id = $1 AND user_id = $2
        "#,
        group_id,
        claims.sub
    )
    .fetch_optional(&state.pool)
    .await?
    .is_some();

    if !is_member {
        return Err(AppError::Forbidden("Not a member of this group".into()));
    }

    let messages = sqlx::query!(
        r#"
        SELECT 
            m.*,
            u.display_name as sender_name,
            u.avatar_url as sender_avatar,
            g.name as group_name,
            g.avatar_url as group_avatar
        FROM messages m
        JOIN users u ON u.id = m.sender_id
        JOIN groups g ON g.id = m.receiver_id
        WHERE 
            m.receiver_id = $1
            AND ($2::timestamptz IS NULL OR m.created_at < $2)
        ORDER BY m.created_at DESC
        LIMIT $3
        "#,
        group_id,
        query.before,
        query.limit.unwrap_or(50)
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(messages.into_iter().map(|m| GroupMessageResponse {
        id: m.id,
        sender_id: m.sender_id,
        group_id: m.receiver_id,
        content: m.content,
        media_url: m.media_url,
        created_at: m.created_at,
        updated_at: m.updated_at,
        is_edited: m.is_edited,
        is_deleted: m.is_deleted,
        sender_name: m.sender_name,
        sender_avatar: m.sender_avatar,
        group_name: m.group_name,
        group_avatar: m.group_avatar,
    }).collect()))
}

pub async fn update_message(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(message_id): Path<Uuid>,
    Json(req): Json<UpdateMessageRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    // Input validation
    if req.content.trim().is_empty() {
        return Err(AppError::BadRequest("Message content cannot be empty".into()));
    }
    if req.content.len() > MAX_MESSAGE_LENGTH {
        return Err(AppError::BadRequest(format!(
            "Message content exceeds maximum length of {} characters",
            MAX_MESSAGE_LENGTH
        )));
    }

    // Get the message
    let message = sqlx::query_as!(
        Message,
        r#"
        SELECT * FROM messages
        WHERE id = $1
        "#,
        message_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Message not found".into()))?;

    // Check if user is the sender
    if message.sender_id != claims.sub {
        return Err(AppError::Forbidden("Cannot edit another user's message".into()));
    }

    // Update the message
    let updated_message = sqlx::query_as!(
        Message,
        r#"
        UPDATE messages
        SET content = $1, updated_at = NOW(), is_edited = true
        WHERE id = $2
        RETURNING *
        "#,
        req.content,
        message_id
    )
    .fetch_one(&state.pool)
    .await?;

    // Get sender info
    let sender = sqlx::query!(
        r#"
        SELECT display_name, avatar_url
        FROM users
        WHERE id = $1
        "#,
        claims.sub
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(MessageResponse {
        id: updated_message.id,
        sender_id: updated_message.sender_id,
        receiver_id: updated_message.receiver_id,
        content: updated_message.content,
        media_url: updated_message.media_url,
        created_at: updated_message.created_at,
        updated_at: updated_message.updated_at,
        is_edited: updated_message.is_edited,
        is_deleted: updated_message.is_deleted,
        sender_name: sender.display_name,
        sender_avatar: sender.avatar_url,
    }))
}

pub async fn delete_message(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Get the message
    let message = sqlx::query_as!(
        Message,
        r#"
        SELECT * FROM messages
        WHERE id = $1
        "#,
        message_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Message not found".into()))?;

    // Check if user is the sender
    if message.sender_id != claims.sub {
        return Err(AppError::Forbidden("Cannot delete another user's message".into()));
    }

    // Soft delete the message
    sqlx::query!(
        r#"
        UPDATE messages
        SET is_deleted = true, content = '', media_url = NULL
        WHERE id = $1
        "#,
        message_id
    )
    .execute(&state.pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
} 