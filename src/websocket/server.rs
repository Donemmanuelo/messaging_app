use warp::ws::{Message, WebSocket};
use futures::stream::StreamExt;
use futures::sink::SinkExt;
use std::collections::HashMap;
use log::{info, error};

pub struct WsServer {
    clients: HashMap<String, warp::ws::WsSender>,
}

impl WsServer {
    pub fn new() -> Self {
        info!("WebSocket server initialized");
        WsServer {
            clients: HashMap::new(),
        }
    }

    pub async fn handle_connection(&mut self, ws: WebSocket, user_id: String) {
        info!("WebSocket connection established for user: {}", user_id);
        let (tx, mut rx) = ws.split();
        self.clients.insert(user_id.clone(), tx);

        while let Some(result) = rx.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(e) => {
                    error!("WebSocket error for user {}: {}", user_id, e);
                    break;
                }
            };

            if let Ok(text) = msg.to_str() {
                info!("Received message from user {}: {}", user_id, text);
                // Broadcast the message to all connected clients
                for (client_id, client_tx) in self.clients.iter_mut() {
                    if client_id != &user_id {
                        if let Err(e) = client_tx.send(Message::text(text)).await {
                            error!("Failed to send message to user {}: {}", client_id, e);
                        }
                    }
                }
            }
        }

        if let Some(tx) = self.clients.remove(&user_id) {
            info!("WebSocket connection closed for user: {}", user_id);
            let _ = tx.close().await;
        }
    }

    pub async fn broadcast_message(&mut self, user_id: String, message: String) {
        info!("Broadcasting message from user {}: {}", user_id, message);
        for (client_id, client_tx) in self.clients.iter_mut() {
            if client_id != &user_id {
                if let Err(e) = client_tx.send(Message::text(message.clone())).await {
                    error!("Failed to send message to user {}: {}", client_id, e);
                }
            }
        }
    }
}