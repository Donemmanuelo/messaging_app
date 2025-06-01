use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,  // Can be either a user_id or group_id
    pub content: String,
    pub media_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_edited: bool,
    pub is_deleted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMessageRequest {
    pub content: String,
    pub media_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMessageRequest {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub content: String,
    pub media_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub sender_name: String,
    pub sender_avatar: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupMessageResponse {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub group_id: Uuid,
    pub content: String,
    pub media_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub sender_name: String,
    pub sender_avatar: Option<String>,
    pub group_name: String,
    pub group_avatar: Option<String>,
}