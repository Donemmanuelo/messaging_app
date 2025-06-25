use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;

use crate::{middleware::auth::AuthUser, models::message::Message as ChatMessage, AppState};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, auth_user))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>, auth_user: AuthUser) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();

    // Spawn a task to forward messages from the broadcast channel to the WebSocket
    let send_task = tokio::spawn(async move {
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
                if let Err(_) = state.ws_tx.send(serde_json::to_string(&message).unwrap()) {
                    break;
                }
            }
        }
    }

    // Clean up
    send_task.abort();
}
