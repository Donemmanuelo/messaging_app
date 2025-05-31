use crate::models::*;
use crate::services::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_chats(
    State(state): State<Arc<AppState>>,
    // TODO: Extract user_id from JWT token
) -> Result<Json<Vec<ChatResponse>>, StatusCode> {
    // For now, using a dummy user_id - in real implementation, extract from JWT
    let user_id = Uuid::new_v4();
    
    let chats = state
        .get_user_chats(user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(chats))
}

pub async fn create_chat(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    // TODO: Extract user_id from JWT token
    let creator_id = Uuid::new_v4();

    let chat = state
        .create_chat(creator_id, req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(chat))
}

pub async fn get_messages(
    State(state): State<Arc<AppState>>,
    Path(chat_id): Path<Uuid>,
) -> Result<Json<Vec<MessageResponse>>, StatusCode> {
    let messages = state
        .get_chat_messages(chat_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(messages))
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Path(chat_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<MessageResponse>, StatusCode> {
    // TODO: Extract user_id from JWT token
    let sender_id = Uuid::new_v4();

    let message = state
        .send_message(chat_id, sender_id, req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(message))
}