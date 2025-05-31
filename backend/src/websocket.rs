use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::sync::{broadcast, RwLock, mpsc};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::handlers::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub chat_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub content: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingIndicator {
    pub chat_id: Uuid,
    pub user_id: Uuid,
    pub is_typing: bool,
}

pub type Clients = Arc<RwLock<HashMap<Uuid, broadcast::Sender<String>>>>;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

pub async fn websocket_connection(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = broadcast::channel(100);

    // In a real implementation, you'd extract user_id from JWT token
    let user_id = Uuid::new_v4(); // Placeholder

    // Spawn a task to handle outgoing messages
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            if let Message::Text(text) = msg {
                if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                    handle_websocket_message(ws_msg, &state, &tx).await;
                }
            }
        }
    }
}

async fn handle_websocket_message(
    msg: WebSocketMessage,
    state: &AppState,
    tx: &broadcast::Sender<String>,
) {
    match msg.message_type.as_str() {
        "message" => {
            // Save message to database and broadcast to chat participants
            if let Some(chat_id) = msg.chat_id {
                // Save to database
                let message_id = Uuid::new_v4();
                let _ = sqlx::query(
                    r#"
                    INSERT INTO messages (id, chat_id, sender_id, content, message_type, status, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, 'text', 'sent', $5, $6)
                    "#
                )
                .bind(message_id)
                .bind(chat_id)
                .bind(msg.sender_id)
                .bind(&msg.content)
                .bind(msg.timestamp)
                .bind(msg.timestamp)
                .execute(&state.db.pool)
                .await;

                // Broadcast to Redis for other server instances
                let redis_msg = serde_json::to_string(&msg).unwrap();
                let mut redis_conn = state.redis_client.get_async_connection().await.unwrap();
                let _: () = redis::cmd("PUBLISH")
                    .arg(format!("chat:{}", chat_id))
                    .arg(redis_msg)
                    .query_async(&mut redis_conn)
                    .await
                    .unwrap();
            }
        }
        "typing" => {
            // Handle typing indicators
            if let Ok(typing_data) = serde_json::from_str::<TypingIndicator>(&serde_json::to_string(&msg).unwrap()) {
                let typing_msg = serde_json::to_string(&typing_data).unwrap();
                let _ = tx.send(typing_msg);
            }
        }
        _ => {}
    }
}