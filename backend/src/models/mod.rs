use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use sqlx::FromRow;
use sqlx::Row;
use uuid::Uuid;

pub mod group;
pub mod message;
pub mod user;
pub mod status_view;

pub use user::{User, UserResponse, CreateUserRequest, LoginRequest, UpdateProfileRequest};

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
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Contact {
    pub id: Uuid,
    pub user_id: Uuid,
    pub contact_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChatRequest {
    pub name: Option<String>,
    pub is_group: bool,
    pub initial_members: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub media_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
pub struct Chat {
    pub id: Uuid,
    pub name: Option<String>,
    pub is_group: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // pub description: Option<String>,
    // pub avatar_url: Option<String>,
    // pub participants: Option<Vec<User>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChatParticipant {
    pub chat_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MessageRead {
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub read_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MessageReaction {
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Media {
    pub id: Uuid,
    pub user_id: Uuid,
    pub type_: String,
    pub url: String,
    pub public_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ForwardMessageRequest {
    pub message_id: Uuid,
    pub chat_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddReactionRequest {
    pub emoji: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveReactionRequest {
    pub emoji: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageReactionResponse {
    pub emoji: String,
    pub count: i64,
    pub users: Vec<User>,
}

#[derive(Debug, Deserialize)]
pub struct SearchMessagesRequest {
    pub query: String,
    pub chat_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub sender: UserResponse,
    pub chat: Chat,
    pub media_url: Option<String>,
    pub reactions: Option<serde_json::Value>,
    pub read_by: Option<serde_json::Value>,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for SearchResult {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(SearchResult {
            id: row.try_get("id")?,
            content: row.try_get("content")?,
            created_at: row.try_get("created_at")?,
            sender: serde_json::from_value(row.try_get::<serde_json::Value, _>("sender")?).unwrap_or_default(),
            chat: serde_json::from_value(row.try_get::<serde_json::Value, _>("chat")?).unwrap_or_default(),
            media_url: row.try_get("media_url").ok(),
            reactions: row.try_get("reactions").ok(),
            read_by: row.try_get("read_by").ok(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaUploadResponse {
    pub id: Uuid,
    pub url: String,
    pub type_: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Document,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}
