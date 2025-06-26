use crate::{
    error::AppError,
    models::message::Message,
    websocket::validation::WebSocketMessage,
    AppState,
};
use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::SinkExt;
use std::sync::Arc;
use uuid::Uuid;
use crate::middleware::AuthUser;


pub async fn ws_handler(ws: WebSocketUpgrade, State(_state): State<Arc<AppState>>, _auth_user: AuthUser) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, _state, _auth_user))
}

pub async fn handle_socket(mut socket: WebSocket, _state: Arc<AppState>, _auth_user: AuthUser) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let WsMessage::Text(text) = msg {
            let _ = socket.send(WsMessage::Text(format!("echo: {}", text))).await;
        }
    }
}

async fn handle_direct_message(
    state: &Arc<AppState>,
    message: Message,
) -> Result<Message, AppError> {
    // Save message to database
    let saved_message = sqlx::query_as!(
        Message,
        r#"
        INSERT INTO messages (id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        RETURNING id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at
        "#,
        message.id,
        message.chat_id,
        message.sender_id,
        message.content,
        message.media_url,
        message.media_type,
        message.reply_to_id
    )
    .fetch_one(&state.pool)
    .await?;

    // Broadcast message to all connected clients
    state.ws_tx.send(serde_json::to_string(&WebSocketMessage::DirectMessage(saved_message.clone()))?)?;

    Ok(saved_message)
}

async fn handle_group_message(
    state: &Arc<AppState>,
    group_id: Uuid,
    message: Message,
) -> Result<Message, AppError> {
    // Verify user is a member of the group
    let is_member = sqlx::query!(
        r#"
        SELECT 1 as exists FROM group_members
        WHERE group_id = $1 AND user_id = $2
        "#,
        group_id,
        message.sender_id
    )
    .fetch_optional(&state.pool)
    .await?
    .is_some();

    if !is_member {
        return Err(AppError::Forbidden("Not a member of this group".into()));
    }

    // Save message to database
    let saved_message = sqlx::query_as!(
        Message,
        r#"
        INSERT INTO messages (id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        RETURNING id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at
        "#,
        message.id,
        group_id,
        message.sender_id,
        message.content,
        message.media_url,
        message.media_type,
        message.reply_to_id
    )
    .fetch_one(&state.pool)
    .await?;

    // Broadcast message to all group members
    state.ws_tx.send(serde_json::to_string(&WebSocketMessage::GroupMessage {
        group_id,
        message: saved_message.clone(),
    })?)?;

    Ok(saved_message)
}

async fn handle_typing(
    state: &Arc<AppState>,
    user_id: Uuid,
    chat_id: Uuid,
) -> Result<(), AppError> {
    // Broadcast message to all connected clients
    state.ws_tx.send(serde_json::to_string(&WebSocketMessage::Typing {
        user_id,
        chat_id,
    })?)?;
    Ok(())
}

async fn handle_group_typing(
    state: &Arc<AppState>,
    group_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    // Verify user is a member of the group
    let is_member = sqlx::query!(
        r#"
        SELECT 1 as exists FROM group_members
        WHERE group_id = $1 AND user_id = $2
        "#,
        group_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?
    .is_some();

    if !is_member {
        return Err(AppError::Forbidden("Not a member of this group".into()));
    }

    // Broadcast message to all group members
    state.ws_tx.send(serde_json::to_string(&WebSocketMessage::GroupTyping {
        group_id,
        user_id,
    })?)?;
    Ok(())
}

async fn handle_read_receipt(
    state: &Arc<AppState>,
    user_id: Uuid,
    chat_id: Uuid,
    message_id: Uuid,
) -> Result<(), AppError> {
    // Broadcast message to all connected clients
    state.ws_tx.send(serde_json::to_string(&WebSocketMessage::Read {
        user_id,
        chat_id,
        message_id,
    })?)?;
    Ok(())
}

async fn handle_group_read_receipt(
    state: &Arc<AppState>,
    group_id: Uuid,
    user_id: Uuid,
    message_id: Uuid,
) -> Result<(), AppError> {
    // Verify user is a member of the group
    let is_member = sqlx::query!(
        r#"
        SELECT 1 as exists FROM group_members
        WHERE group_id = $1::uuid AND user_id = $2::uuid
        "#,
        group_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?
    .is_some();

    if !is_member {
        return Err(AppError::Forbidden("Not a member of this group".into()));
    }

    // Broadcast message to all group members
    state.ws_tx.send(serde_json::to_string(&WebSocketMessage::GroupRead {
        group_id,
        user_id,
        message_id,
    })?)?;
    Ok(())
}
