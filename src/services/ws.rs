let message = serde_json::to_string(&json!({
    "type": "message",
    "data": message
})).map_err(|e| format!("Serialization error: {}", e))?;
self.broadcast(&message)

let message = serde_json::to_string(&json!({
    "type": "chat",
    "data": chat
})).map_err(|e| format!("Serialization error: {}", e))?;
self.broadcast(&message) 