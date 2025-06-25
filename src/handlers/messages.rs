#[derive(sqlx::FromRow)]
struct MessageInsertResult {
    id: Uuid,
    chat_id: Uuid,
    sender_id: Uuid,
    content: Option<String>,
    media_url: Option<String>,
    media_type: Option<String>,
    reply_to_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct GroupMessageInsertResult {
    id: Uuid,
    chat_id: Uuid,
    sender_id: Uuid,
    content: Option<String>,
    media_url: Option<String>,
    media_type: Option<String>,
    reply_to_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

        content: message.content.as_deref().unwrap_or("").to_string(),

        content: message.content.as_deref().unwrap_or("").to_string(),

        content: m.content.as_deref().unwrap_or("").to_string(),

        content: updated_message.content.as_deref().unwrap_or("").to_string(), 