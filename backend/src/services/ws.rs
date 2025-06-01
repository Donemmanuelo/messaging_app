use axum::extract::ws::Message;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct WsService {
    tx: broadcast::Sender<String>,
}

impl WsService {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    pub fn broadcast(&self, message: &str) -> Result<(), broadcast::error::SendError<String>> {
        self.tx.send(message.to_string())
    }

    pub fn broadcast_message(&self, message: &crate::models::Message) -> Result<(), broadcast::error::SendError<String>> {
        let message = json!({
            "type": "message",
            "data": message
        });
        self.broadcast(&message.to_string())
    }

    pub fn broadcast_chat(&self, chat: &crate::models::Chat) -> Result<(), broadcast::error::SendError<String>> {
        let message = json!({
            "type": "chat",
            "data": chat
        });
        self.broadcast(&message.to_string())
    }
} 