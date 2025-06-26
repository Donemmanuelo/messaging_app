use serde_json::json;
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
        self.tx.send(message.to_string()).map(|_| ())
    }

    pub fn broadcast_message(
        &self,
        message: &crate::models::message::Message,
    ) -> Result<(), broadcast::error::SendError<String>> {
        let message = serde_json::to_string(&json!({
            "type": "message",
            "data": message
        })).unwrap();
        self.broadcast(&message)
    }

    pub fn broadcast_chat(
        &self,
        chat: &crate::models::Chat,
    ) -> Result<(), broadcast::error::SendError<String>> {
        let message = serde_json::to_string(&json!({
            "type": "chat",
            "data": chat
        })).unwrap();
        self.broadcast(&message)
    }
}
