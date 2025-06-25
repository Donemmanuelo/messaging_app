use std::sync::Arc;
use futures::StreamExt;

async fn handle_direct_message(
    state: &Arc<AppState>,
    message: crate::models::message::Message,
) -> Result<crate::models::message::Message, AppError> {
    // Save message to database
    let saved_message = sqlx::query_as!(
        crate::models::message::Message,
        r#"
        INSERT INTO messages (id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        RETURNING id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at
        "#,
        message.id,
        message.chat_id,
        message.sender_id,
        message.content,
        message.media_url,
        message.media_type,
        message.reply_to_id
    )
    .fetch_one(&state.pool)
    .await?;

    // Broadcast message to all connected clients
    state
        .ws_tx
        .send(serde_json::to_string(&WebSocketMessage::DirectMessage(
            saved_message.clone(),
        ))?)?;

    Ok(saved_message)
}

if let Message::Text(text) = msg {
    if let Ok(parsed) = serde_json::from_str::<YourMessageStruct>(&text) {
        // Use parsed struct fields here
    }
} 