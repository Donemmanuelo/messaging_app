let original_message = sqlx::query_as!(
    ChatMessage,
    r#"
    SELECT id, chat_id, sender_id, content, media_url, media_type, reply_to_id, created_at, updated_at
    FROM messages
    WHERE id = $1
    "#,
    req.message_id
)
.fetch_one(&pool)
.await?;

if !is_participant.unwrap_or(false) {
    return Err(AppError::Forbidden("Not a participant in this chat".into()));
} 

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChatMessage {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub sender_id: Uuid,
    pub content: Option<String>,
    pub media_url: Option<String>,
    pub media_type: Option<String>,
    pub reply_to_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
} 

pub async fn get_read_receipts(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path((chat_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ReadReceipts>, AppError> {
    // ... existing code ...
    if !is_participant.unwrap_or(false) {
        return Err(AppError::Forbidden("Not a participant in this chat".into()));
    }
    // ... existing code ...
} 