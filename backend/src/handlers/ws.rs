use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;
use futures_util::{SinkExt, StreamExt};

use crate::{
    AppState,
    middleware::auth::AuthUser,
    models::Message as ChatMessage,
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, auth_user))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>, auth_user: AuthUser) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws.subscribe();

    // Spawn a task to forward messages from the broadcast channel to the WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Err(_) = sender.send(Message::Text(msg)).await {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(message) = serde_json::from_str::<ChatMessage>(&text) {
                // Broadcast the message to all connected clients
                if let Err(_) = state.ws.broadcast_message(&message) {
                    break;
                }
            }
        }
    }

    // Clean up
    send_task.abort();
} 