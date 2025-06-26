use crate::{
    error::AppError,
    models::message::{
        CreateMessageRequest, GroupMessageResponse, MessageResponse, UpdateMessageRequest,
    },
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use crate::middleware::AuthUser;
use crate::services::web_push::{PushSubscription as WebPushSubscription, PushSubscriptionKeys, VapidConfig, send_web_push};
use sqlx::Row;

const MAX_MESSAGE_LENGTH: usize = 4000;

#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    pub before: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct MessageWithSender {
    id: Uuid,
    chat_id: Uuid,
    sender_id: Uuid,
    content: Option<String>,
    media_url: Option<String>,
    media_type: Option<String>,
    reply_to_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    sender_name: String,
    sender_avatar: Option<String>,
}

#[derive(sqlx::FromRow)]
struct GroupMessageWithSenderAndGroup {
    id: Uuid,
    chat_id: Uuid,
    sender_id: Uuid,
    content: Option<String>,
    media_url: Option<String>,
    media_type: Option<String>,
    reply_to_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    sender_name: String,
    sender_avatar: Option<String>,
    group_name: String,
    group_avatar: Option<String>,
}

#[derive(sqlx::FromRow)]
struct MessageInsertResult {
    id: Uuid,
    chat_id: Uuid,
    sender_id: Uuid,
    content: Option<String>,
    media_url: Option<String>,
    media_type: Option<String>,
    reply_to_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct GroupMessageInsertResult {
    id: Uuid,
    chat_id: Uuid,
    sender_id: Uuid,
    content: String,
    media_url: Option<String>,
    media_type: Option<String>,
    reply_to_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[axum::debug_handler]
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(chat_id): Path<Uuid>,
    Json(req): Json<CreateMessageRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Input validation
    if req.content.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Message content cannot be empty".into(),
        ));
    }
    if req.content.len() > MAX_MESSAGE_LENGTH {
        return Err(AppError::BadRequest(format!(
            "Message content exceeds maximum length of {} characters",
            MAX_MESSAGE_LENGTH
        )));
    }

    let message_id = Uuid::new_v4();

    // Save message to database
    let message = sqlx::query_as::<_, MessageInsertResult>(
        r#"
        INSERT INTO messages (id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        RETURNING id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at
        "#
    )
    .bind(message_id)
    .bind(chat_id)
    .bind(auth_user.id)
    .bind(req.content)
    .bind(req.media_url)
    .bind(req.media_type)
    .bind(req.reply_to_id)
    .fetch_one(&state.pool)
    .await?;

    // Get sender info
    let sender = sqlx::query!(
        r#"
        SELECT display_name, avatar_url FROM users WHERE id = $1
        "#,
        auth_user.id
    )
    .fetch_one(&state.pool)
    .await?;

    // Send push notification to all chat participants except sender
    let participants = sqlx::query!(
        "SELECT user_id FROM chat_participants WHERE chat_id = $1 AND user_id != $2",
        chat_id,
        auth_user.id
    )
    .fetch_all(&state.pool)
    .await?;
    let vapid = VapidConfig::from_env();
    for participant in participants {
        let subs = sqlx::query(
            "SELECT endpoint, keys::text as keys_str FROM push_subscriptions WHERE user_id = $1"
        )
        .bind(participant.user_id)
        .fetch_all(&state.pool)
        .await?;
        for sub in subs {
            let endpoint: String = sub.try_get("endpoint")?;
            let keys_str: String = sub.try_get("keys_str")?;
            if let Ok(keys) = serde_json::from_str::<PushSubscriptionKeys>(&keys_str) {
                let push_sub = WebPushSubscription {
                    endpoint,
                    keys,
                };
                let payload = serde_json::json!({
                    "title": "New Message",
                    "body": sender.display_name.clone().unwrap_or("New message".to_string()),
                    "data": { "chat_id": chat_id.to_string() },
                }).to_string();
                let _ = send_web_push(&push_sub, &payload, &vapid).await;
            }
        }
    }

    
    Ok(Json(MessageResponse {
        id: message.id,
        sender_id: message.sender_id,
        receiver_id: chat_id,
        content: message.content.as_deref().unwrap_or("").to_string(),
        media_url: message.media_url,
        media_type: message.media_type,
        reply_to_id: message.reply_to_id,
        created_at: message.created_at,
        updated_at: message.updated_at,
        is_edited: false,
        is_deleted: false,
        sender_name: sender.display_name.unwrap_or_else(|| "Unknown".to_string()),
        sender_avatar: sender.avatar_url,
    }))
}

#[axum::debug_handler]
pub async fn send_group_message(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(group_id): Path<Uuid>,
    Json(req): Json<CreateMessageRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Input validation
    if req.content.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Message content cannot be empty".into(),
        ));
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
        SELECT 1 as exists FROM group_members
        WHERE group_id = $1 AND user_id = $2
        "#,
        group_id,
        auth_user.id
    )
    .fetch_optional(&state.pool)
    .await?
    .is_some();

    if !is_member {
        return Err(AppError::Forbidden("Not a member of this group".into()));
    }

    let message_id = Uuid::new_v4();

    // Save message to database
    let message = sqlx::query_as::<_, GroupMessageInsertResult>(
        r#"
        INSERT INTO messages (id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        RETURNING id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at
        "#
    )
    .bind(message_id)
    .bind(group_id)
    .bind(auth_user.id)
    .bind(req.content)
    .bind(req.media_url)
    .bind(req.media_type)
    .bind(req.reply_to_id)
    .fetch_one(&state.pool)
    .await?;

    // Get sender info
    let sender = sqlx::query!(
        r#"
        SELECT display_name, avatar_url
        FROM users
        WHERE id = $1
        "#,
        auth_user.id
    )
    .fetch_one(&state.pool)
    .await?;

    // Get group member ids except sender
    let members = sqlx::query!(
        "SELECT user_id FROM group_members WHERE group_id = $1 AND user_id != $2",
        group_id,
        auth_user.id
    )
    .fetch_all(&state.pool)
    .await?;
    let vapid = VapidConfig::from_env();
    for member in members {
        let subs = sqlx::query(
            "SELECT endpoint, keys::text as keys_str FROM push_subscriptions WHERE user_id = $1"
        )
        .bind(member.user_id)
        .fetch_all(&state.pool)
        .await?;
        for sub in subs {
            let endpoint: String = sub.try_get("endpoint")?;
            let keys_str: String = sub.try_get("keys_str")?;
            if let Ok(keys) = serde_json::from_str::<PushSubscriptionKeys>(&keys_str) {
                let push_sub = WebPushSubscription {
                    endpoint,
                    keys,
                };
                let payload = serde_json::json!({
                    "title": "New Group Message",
                    "body": sender.display_name.clone().unwrap_or("New group message".to_string()),
                    "data": { "group_id": group_id.to_string() },
                }).to_string();
                let _ = send_web_push(&push_sub, &payload, &vapid).await;
            }
        }
    }

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
        group_id: group_id,
        content: message.content.to_string(),
        media_url: message.media_url,
        media_type: message.media_type,
        reply_to_id: message.reply_to_id,
        created_at: message.created_at,
        updated_at: message.updated_at,
        is_edited: false,
        is_deleted: false,
        sender_name: sender.display_name.unwrap_or_else(|| "Unknown".to_string()),
        sender_avatar: sender.avatar_url,
        group_name: group.name,
        group_avatar: group.avatar_url,
    }))
}

#[axum::debug_handler]
pub async fn get_messages(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(chat_id): Path<Uuid>,
    Query(query): Query<MessageQuery>,
) -> Result<impl IntoResponse, AppError> {
    let messages = sqlx::query_as::<_, MessageWithSender>(
        r#"
        SELECT 
            m.id as id, m.chat_id as chat_id, m.sender_id as sender_id, m.content as content, m.media_url as media_url, m.media_type as media_type, m.reply_to_id as reply_to_id, m.created_at as created_at, m.updated_at as updated_at,
            u.display_name as sender_name, u.avatar_url as sender_avatar
        FROM messages m
        JOIN users u ON u.id = m.sender_id
        WHERE m.chat_id = $1
        AND ($2::timestamptz IS NULL OR m.created_at < $2)
        ORDER BY m.created_at DESC
        LIMIT $3
        "#
    )
    .bind(chat_id)
    .bind(query.before)
    .bind(query.limit.unwrap_or(50))
    .fetch_all(&state.pool)
    .await?;

    let messages: Vec<_> = messages.into_iter().map(|m| MessageResponse {
        id: m.id,
        sender_id: m.sender_id,
        receiver_id: chat_id,
        content: m.content.as_deref().unwrap_or("").to_string(),
        media_url: m.media_url,
        media_type: m.media_type,
        reply_to_id: m.reply_to_id,
        created_at: m.created_at,
        updated_at: m.updated_at,
        is_edited: false,
        is_deleted: false,
        sender_name: m.sender_name,
        sender_avatar: m.sender_avatar,
    }).collect();

    Ok(Json(messages))
}

#[axum::debug_handler]
pub async fn get_group_messages(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(group_id): Path<Uuid>,
    Query(query): Query<MessageQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Verify user is a member of the group
    let is_member = sqlx::query!(
        r#"
        SELECT 1 as exists FROM group_members
        WHERE group_id = $1 AND user_id = $2
        "#,
        group_id,
        auth_user.id
    )
    .fetch_optional(&state.pool)
    .await?
    .is_some();

    if !is_member {
        return Err(AppError::Forbidden("Not a member of this group".into()));
    }

    let messages = sqlx::query_as::<_, GroupMessageWithSenderAndGroup>(
        r#"
        SELECT 
            m.id as id, m.chat_id as chat_id, m.sender_id as sender_id, m.content as content, m.media_url as media_url, m.media_type as media_type, m.reply_to_id as reply_to_id, m.created_at as created_at, m.updated_at as updated_at,
            u.display_name as sender_name, u.avatar_url as sender_avatar,
            g.name as group_name, g.avatar_url as group_avatar
        FROM messages m
        JOIN users u ON u.id = m.sender_id
        JOIN groups g ON g.id = $1
        WHERE m.chat_id = $1
        AND ($2::timestamptz IS NULL OR m.created_at < $2)
        ORDER BY m.created_at DESC
        LIMIT $3
        "#
    )
    .bind(group_id)
    .bind(query.before)
    .bind(query.limit.unwrap_or(50))
    .fetch_all(&state.pool)
    .await?;

    let messages: Vec<_> = messages.into_iter().map(|m| GroupMessageResponse {
        id: m.id,
        sender_id: m.sender_id,
        group_id: group_id,
        content: m.content.as_deref().unwrap_or("").to_string(),
        media_url: m.media_url,
        media_type: m.media_type,
        reply_to_id: m.reply_to_id,
        created_at: m.created_at,
        updated_at: m.updated_at,
        is_edited: false,
        is_deleted: false,
        sender_name: m.sender_name,
        sender_avatar: m.sender_avatar,
        group_name: m.group_name,
        group_avatar: m.group_avatar,
    }).collect();

    Ok(Json(messages))
}

#[axum::debug_handler]
pub async fn update_message(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path((chat_id, message_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateMessageRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Input validation
    if req.content.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Message content cannot be empty".into(),
        ));
    }
    if req.content.len() > MAX_MESSAGE_LENGTH {
        return Err(AppError::BadRequest(format!(
            "Message content exceeds maximum length of {} characters",
            MAX_MESSAGE_LENGTH
        )));
    }

    // Get the message
    let message = sqlx::query_as!(
        MessageInsertResult,
        r#"
        SELECT id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at
        FROM messages
        WHERE id = $1
        "#,
        message_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Message not found".into()))?;

    // Check if user is the sender
    if message.sender_id != auth_user.id {
        return Err(AppError::Forbidden(
            "Cannot edit another user's message".into(),
        ));
    }

    // Update the message
    let updated_message = sqlx::query_as!(
        MessageInsertResult,
        r#"
        UPDATE messages
        SET content = $1, media_url = $2, media_type = $3, reply_to_id = $4, updated_at = NOW()
        WHERE id = $5
        RETURNING id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at
        "#,
        Some(req.content),
        req.media_url,
        req.media_type,
        req.reply_to_id,
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
        auth_user.id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(MessageResponse {
        id: updated_message.id,
        sender_id: updated_message.sender_id,
        receiver_id: chat_id,
        content: updated_message.content.as_deref().unwrap_or("").to_string(),
        media_url: updated_message.media_url,
        media_type: updated_message.media_type,
        reply_to_id: updated_message.reply_to_id,
        created_at: updated_message.created_at,
        updated_at: updated_message.updated_at,
        is_edited: false,
        is_deleted: false,
        sender_name: sender.display_name.unwrap_or_else(|| "Unknown".to_string()),
        sender_avatar: sender.avatar_url,
    }))
}

#[axum::debug_handler]
pub async fn delete_message(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path((chat_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    // Get the message
    let message = sqlx::query_as!(
        MessageInsertResult,
        r#"
        SELECT id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at
        FROM messages
        WHERE id = $1
        "#,
        message_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Message not found".into()))?;

    // Check if user is the sender
    if message.sender_id != auth_user.id {
        return Err(AppError::Forbidden(
            "Cannot delete another user's message".into(),
        ));
    }

    // Soft delete the message
    sqlx::query!(
        r#"
        UPDATE messages
        SET content = $1, media_url = NULL, media_type = NULL, reply_to_id = NULL
        WHERE id = $2
        "#,
        Some(String::new()),
        message_id
    )
    .execute(&state.pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}
