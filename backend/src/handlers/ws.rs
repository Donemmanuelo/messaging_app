use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::Mutex;
use uuid;
use sqlx;
use crate::{middleware::auth::AuthUser, models::message::Message as ChatMessage, AppState};
use crate::services::web_push::{PushSubscription as WebPushSubscription, PushSubscriptionKeys, VapidConfig, send_web_push};

// In-memory map of user_id -> sender (for demo; use a real connection manager in production)
lazy_static::lazy_static! {
    static ref USER_SENDERS: Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>> = Mutex::new(HashMap::new());
}

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
    let user_id = auth_user.id.to_string();

    // Create a channel for outgoing messages to this socket
    let (outgoing_tx, mut outgoing_rx) = tokio::sync::mpsc::unbounded_channel();

    // Register this user's sender
    {
        let mut map = USER_SENDERS.lock().await;
        map.insert(user_id.clone(), outgoing_tx.clone());
    }

    // Task to forward messages from outgoing_rx to the WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = outgoing_rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Task to forward messages from the broadcast channel to outgoing_tx
    let outgoing_tx_clone = outgoing_tx.clone();
    let broadcast_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let _ = outgoing_tx_clone.send(Message::Text(msg));
        }
    });

    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            // Try to parse as signaling message
            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                if let Some(msg_type) = json.get("type").and_then(|v| v.as_str()) {
                    if msg_type == "call-offer" && json.get("to").is_some() {
                        // Relay to intended recipient
                        let to = json.get("to").unwrap().as_str().unwrap_or("");
                        let map = USER_SENDERS.lock().await;
                        if let Some(tx) = map.get(to) {
                            let mut relay = json.clone();
                            relay["from"] = Value::String(user_id.clone());
                            let _ = tx.send(Message::Text(relay.to_string()));
                        }
                        // Send push notification to recipient
                        drop(map); // release lock before DB
                        if let Ok(uuid_to) = uuid::Uuid::parse_str(to) {
                            let subs = sqlx::query("SELECT endpoint, keys::text as keys_str FROM push_subscriptions WHERE user_id = $1")
                                .bind(uuid_to)
                                .fetch_all(&state.pool)
                                .await
                                .unwrap_or_default();
                            let vapid = VapidConfig::from_env();
                            for sub in subs {
                                // Use sqlx::Row::get instead of try_get, and handle errors gracefully
                                use sqlx::Row;
                                let endpoint: String = match sub.try_get("endpoint") {
                                    Ok(val) => val,
                                    Err(_) => continue,
                                };
                                let keys_str: String = match sub.try_get("keys_str") {
                                    Ok(val) => val,
                                    Err(_) => continue,
                                };
                                if let Ok(keys) = serde_json::from_str::<PushSubscriptionKeys>(&keys_str) {
                                    let push_sub = WebPushSubscription { endpoint, keys };
                                    let payload = serde_json::json!({
                                        "title": "Incoming Call",
                                        "body": "You have an incoming call!",
                                        "data": { "type": "call-offer" },
                                    }).to_string();
                                    let _ = send_web_push(&push_sub, &payload, &vapid).await;
                                }
                            }
                        }
                        continue;
                    }
                    // Group call signaling relay
                    else if msg_type == "group-call-offer" {
                        if let (Some(to), Some(from)) = (json.get("to"), json.get("from")) {
                            let to_id = to.as_str().unwrap_or("");
                            let from_id = from.as_str().unwrap_or("");
                            let map = USER_SENDERS.lock().await;
                            for (user_id, tx) in map.iter() {
                                if user_id != from_id && (to_id.is_empty() || user_id == to_id) {
                                    let _ = tx.send(Message::Text(text.clone()));
                                }
                            }
                            drop(map);
                            // Send push notification to all group members except sender
                            if let (Ok(group_uuid), Ok(from_uuid)) = (
                                uuid::Uuid::parse_str(to_id),
                                uuid::Uuid::parse_str(from_id),
                            ) {
                                // Find all group members except sender
                                let members = sqlx::query!(
                                    "SELECT user_id FROM group_members WHERE group_id = $1 AND user_id != $2",
                                    group_uuid,
                                    from_uuid
                                )
                                .fetch_all(&state.pool)
                                .await
                                .unwrap_or_default();
                                let vapid = VapidConfig::from_env();
                                for member in members {
                                    let subs = sqlx::query("SELECT endpoint, keys::text as keys_str FROM push_subscriptions WHERE user_id = $1")
                                        .bind(member.user_id)
                                        .fetch_all(&state.pool)
                                        .await
                                        .unwrap_or_default();
                                    for sub in subs {
                                        // Use sqlx::Row::get instead of try_get, and handle errors gracefully
                                        use sqlx::Row;
                                        let endpoint: String = match sub.get("endpoint") {
                                            val => val,
                                        };
                                        let keys_str: String = match sub.get("keys_str") {
                                            val => val,
                                        };
                                        if let Ok(keys) = serde_json::from_str::<PushSubscriptionKeys>(&keys_str) {
                                            let push_sub = WebPushSubscription { endpoint, keys };
                                            let payload = serde_json::json!({
                                                "title": "Incoming Group Call",
                                                "body": "You have an incoming group call!",
                                                "data": { "type": "group-call-offer", "group_id": to_id },
                                            }).to_string();
                                            let _ = send_web_push(&push_sub, &payload, &vapid).await;
                                        }
                                    }
                                }
                            }
                        }
                        continue;
                    }
                    else if matches!(msg_type, "group-call-answer" | "group-ice-candidate") {
                        if let (Some(to), Some(from)) = (json.get("to"), json.get("from")) {
                            let to_id = to.as_str().unwrap_or("");
                            let from_id = from.as_str().unwrap_or("");
                            let map = USER_SENDERS.lock().await;
                            for (user_id, tx) in map.iter() {
                                if user_id != from_id && (to_id.is_empty() || user_id == to_id) {
                                    let _ = tx.send(Message::Text(text.clone()));
                                }
                            }
                        }
                        continue;
                    }
                }
            }
            // Fallback: treat as chat message
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
    broadcast_task.abort();
    // Remove user from map
    let mut map = USER_SENDERS.lock().await;
    map.remove(&user_id);
}
