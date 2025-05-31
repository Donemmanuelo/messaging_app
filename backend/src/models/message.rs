use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub sender_id: Uuid,
    pub content: Option<String>,
    pub message_type: MessageType,
    pub media_url: Option<String>,
    pub reply_to_id: Option<Uuid>,
    pub forwarded_from_id: Option<Uuid>,
    pub status: MessageStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "message_type", rename_all = "lowercase")]
pub enum MessageType {
    Text,
    Image,
    Video,
    Audio,
    Document,
    Location,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "message_status", rename_all = "lowercase")]
pub enum MessageStatus {
    Sent,
    Delivered,
    Read,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: Option<String>,
    pub message_type: MessageType,
    pub media_url: Option<String>,
    pub reply_to_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub sender: crate::models::UserResponse,
    pub content: Option<String>,
    pub message_type: MessageType,
    pub media_url: Option<String>,
    pub reply_to: Option<Box<MessageResponse>>,
    pub status: MessageStatus,
    pub created_at: DateTime<Utc>,
}