use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use std::sync::Arc;

use crate::{
    auth::AuthUser,
    error::AppError,
    models::{Chat,CreateChatRequest, SendMessageRequest},
};
use crate::models::message::Message;
use crate::AppState;

pub async fn get_chats(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let chats = sqlx::query_as!(
        Chat,
        r#"
        SELECT c.id, c.name, c.is_group, c.created_at, c.updated_at
        FROM chats c
        JOIN chat_participants cp ON cp.chat_id = c.id
        WHERE cp.user_id = $1
        ORDER BY c.updated_at DESC
        "#,
        auth_user.id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(chats))
}

pub async fn get_chat(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(chat_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let chat = sqlx::query_as!(
        Chat,
        r#"
        SELECT c.id, c.name, c.is_group, c.created_at, c.updated_at
        FROM chats c
        WHERE c.id = $1
        "#,
        chat_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Chat not found".into()))?;

    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2) as exists",
        chat_id,
        auth_user.id
    )
    .fetch_one(&state.pool)
    .await?
    .exists;

    if is_participant != Some(true) {
        return Err(AppError::Forbidden("Not a participant in this chat".into()));
    }

    Ok(Json(chat))
}

pub async fn create_chat(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(req): Json<CreateChatRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut tx = state.pool.begin().await?;

    let chat = sqlx::query_as!(
        Chat,
        r#"
        INSERT INTO chats (name, is_group)
        VALUES ($1, $2)
        RETURNING id, name, is_group, created_at, updated_at
        "#,
        req.name,
        req.is_group
    )
    .fetch_one(&mut *tx)
    .await?;

    for user_id in req.initial_members.clone().unwrap_or_default() {
        sqlx::query!(
            "INSERT INTO chat_participants (chat_id, user_id) VALUES ($1, $2)",
            chat.id,
            user_id
        )
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query!(
        "INSERT INTO chat_participants (chat_id, user_id) VALUES ($1, $2)",
        chat.id,
        auth_user.id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(chat)))
}

pub async fn get_messages(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(chat_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2) as exists",
        chat_id,
        auth_user.id
    )
    .fetch_one(&state.pool)
    .await?
    .exists;

    if is_participant != Some(true) {
        return Err(AppError::Forbidden("Not a participant in this chat".into()));
    }

    let messages = sqlx::query_as!(
        Message,
        r#"
        SELECT id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at
        FROM messages
        WHERE chat_id = $1
        ORDER BY created_at ASC
        "#,
        chat_id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(messages))
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(chat_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<impl IntoResponse, AppError> {
    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2) as exists",
        chat_id,
        auth_user.id
    )
    .fetch_one(&state.pool)
    .await?
    .exists;

    if is_participant != Some(true) {
        return Err(AppError::Forbidden("Not a participant in this chat".into()));
    }

    let message = sqlx::query_as!(
        Message,
        r#"
        INSERT INTO messages (chat_id, sender_id, content, media_url, media_type, reply_to_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at
        "#,
        chat_id,
        auth_user.id,
        req.content,
        req.media_url,
        None::<String>,
        None::<Uuid>
    )
    .fetch_one(&state.pool)
    .await?;

    sqlx::query!("UPDATE chats SET updated_at = NOW() WHERE id = $1", chat_id)
        .execute(&state.pool)
        .await?;

    Ok((StatusCode::CREATED, Json(message)))
}

pub async fn delete_message(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path((chat_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2) as exists",
        chat_id,
        auth_user.id
    )
    .fetch_one(&state.pool)
    .await?
    .exists;

    if is_participant != Some(true) {
        return Err(AppError::Forbidden("Not a participant in this chat".into()));
    }

    let is_sender = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM messages WHERE id = $1 AND sender_id = $2) as exists",
        message_id,
        auth_user.id
    )
    .fetch_one(&state.pool)
    .await?
    .exists;

    if is_sender != Some(true) {
        return Err(AppError::Forbidden("Not the sender of this message".into()));
    }

    sqlx::query!("DELETE FROM messages WHERE id = $1", message_id)
        .execute(&state.pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
