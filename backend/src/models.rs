use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: Option<String>,
    pub is_online: bool,
    pub last_seen: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Chat {
    pub id: Uuid,
    pub chat_type: String, // 'direct' or 'group'
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub sender_id: Uuid,
    pub content: Option<String>,
    pub message_type: String, // 'text', 'image', 'video', 'audio', 'document'
    pub media_url: Option<String>,
    pub reply_to: Option<Uuid>,
    pub status: String, // 'sent', 'delivered', 'read'
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatParticipant {
    pub chat_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub is_admin: bool,
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: Option<String>,
    pub is_online: bool,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateChatRequest {
    pub chat_type: String,
    pub name: Option<String>,
    pub participant_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: Option<String>,
    pub message_type: String,
    pub media_url: Option<String>,
    pub reply_to: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub id: Uuid,
    pub chat_type: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub participants: Vec<UserResponse>,
    pub last_message: Option<MessageResponse>,
    pub unread_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub sender: UserResponse,
    pub content: Option<String>,
    pub message_type: String,
    pub media_url: Option<String>,
    pub reply_to: Option<Box<MessageResponse>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub chat_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub content: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub is_typing: Option<bool>,
}