let msg_json = serde_json::to_string(&message).map_err(|e| AppError::InternalServerError(format!("Serialization error: {}", e)))?;
if let Err(_) = state.ws_tx.send(msg_json) {
    // handle error
} 