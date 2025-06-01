use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;
use crate::{
    AppState,
    error::AppError,
    models::{
        message::Message as ChatMessage,
    },
    websocket::validation::{WebSocketMessage, WebSocketResponse},
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();

    // Spawn a task to forward messages from the broadcast channel to the WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Err(_) = sender.send(Message::Text(msg)).await {
                break;
            }
        }
    });

    // Spawn a task to handle incoming messages
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(event) = serde_json::from_str::<WebSocketMessage>(&text) {
                        if let Err(e) = event.validate() {
                            let response = WebSocketResponse::<()>::error("VALIDATION_ERROR", &e.to_string());
                            if let Err(_) = sender.send(Message::Text(serde_json::to_string(&response).unwrap())).await {
                                break;
                            }
                            continue;
                        }

                        let response = match event {
                            WebSocketMessage::DirectMessage(message) => {
                                match handle_direct_message(&state_clone, message).await {
                                    Ok(msg) => WebSocketResponse::success(msg),
                                    Err(e) => WebSocketResponse::<ChatMessage>::error("MESSAGE_ERROR", &e.to_string()),
                                }
                            }
                            WebSocketMessage::GroupMessage { group_id, message } => {
                                match handle_group_message(&state_clone, group_id, message).await {
                                    Ok(msg) => WebSocketResponse::success(msg),
                                    Err(e) => WebSocketResponse::<ChatMessage>::error("GROUP_MESSAGE_ERROR", &e.to_string()),
                                }
                            }
                            WebSocketMessage::Typing { user_id, chat_id } => {
                                match handle_typing(&state_clone, user_id, chat_id).await {
                                    Ok(_) => WebSocketResponse::<()>::success(()),
                                    Err(e) => WebSocketResponse::<()>::error("TYPING_ERROR", &e.to_string()),
                                }
                            }
                            WebSocketMessage::GroupTyping { group_id, user_id } => {
                                match handle_group_typing(&state_clone, group_id, user_id).await {
                                    Ok(_) => WebSocketResponse::<()>::success(()),
                                    Err(e) => WebSocketResponse::<()>::error("GROUP_TYPING_ERROR", &e.to_string()),
                                }
                            }
                            WebSocketMessage::Read { user_id, chat_id, message_id } => {
                                match handle_read_receipt(&state_clone, user_id, chat_id, message_id).await {
                                    Ok(_) => WebSocketResponse::<()>::success(()),
                                    Err(e) => WebSocketResponse::<()>::error("READ_ERROR", &e.to_string()),
                                }
                            }
                            WebSocketMessage::GroupRead { group_id, user_id, message_id } => {
                                match handle_group_read_receipt(&state_clone, group_id, user_id, message_id).await {
                                    Ok(_) => WebSocketResponse::<()>::success(()),
                                    Err(e) => WebSocketResponse::<()>::error("GROUP_READ_ERROR", &e.to_string()),
                                }
                            }
                        };

                        if let Err(_) = sender.send(Message::Text(serde_json::to_string(&response).unwrap())).await {
                            break;
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

async fn handle_direct_message(state: &Arc<AppState>, message: ChatMessage) -> Result<ChatMessage, AppError> {
    // Save message to database
    let saved_message = sqlx::query_as!(
        ChatMessage,
        r#"
        INSERT INTO messages (id, sender_id, receiver_id, content, media_url, created_at, updated_at, is_edited, is_deleted)
        VALUES ($1, $2, $3, $4, $5, NOW(), NULL, false, false)
        RETURNING *
        "#,
        message.id,
        message.sender_id,
        message.receiver_id,
        message.content,
        message.media_url
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
    message: ChatMessage,
) -> Result<ChatMessage, AppError> {
    // Verify user is a member of the group
    let is_member = sqlx::query!(
        r#"
        SELECT 1 FROM group_members
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
        ChatMessage,
        r#"
        INSERT INTO messages (id, sender_id, receiver_id, content, media_url, created_at, updated_at, is_edited, is_deleted)
        VALUES ($1, $2, $3, $4, $5, NOW(), NULL, false, false)
        RETURNING *
        "#,
        message.id,
        message.sender_id,
        group_id,
        message.content,
        message.media_url
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
    state.ws_tx.send(serde_json::to_string(&WebSocketMessage::Typing { user_id, chat_id })?)?;
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
        SELECT 1 FROM group_members
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

    state.ws_tx.send(serde_json::to_string(&WebSocketMessage::GroupTyping { group_id, user_id })?)?;
    Ok(())
}

async fn handle_read_receipt(
    state: &Arc<AppState>,
    user_id: Uuid,
    chat_id: Uuid,
    message_id: Uuid,
) -> Result<(), AppError> {
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
        SELECT 1 FROM group_members
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

    state.ws_tx.send(serde_json::to_string(&WebSocketMessage::GroupRead {
        group_id,
        user_id,
        message_id,
    })?)?;
    Ok(())
} 