let msg = sqlx::query_as!(Message,
    "INSERT INTO messages (chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at",
    input.chat_id, input.sender_id, input.content, input.media_url, None::<String>, None::<Uuid>, Utc::now(), Utc::now()
)
.fetch_one(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

let messages = sqlx::query_as!(
    Message,
    "SELECT id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at FROM messages WHERE chat_id = $1 ORDER BY created_at ASC",
    chat_id
)
.fetch_all(&pool)
.await
.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

Json(messages) 