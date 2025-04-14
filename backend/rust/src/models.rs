use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use diesel::{Queryable, Insertable};
use diesel::prelude::*;

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub profile_picture: Option<String>,
    pub status: Option<String>,
    pub last_seen: Option<DateTime<Utc>>,
    pub is_online: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct Chat {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "chats"]
pub struct NewChat {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct Message {
    pub id: i32,
    pub chat_id: i32,
    pub sender_id: i32,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub status: MessageStatus,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "messages"]
pub struct NewMessage {
    pub chat_id: i32,
    pub sender_id: i32,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, DbEnum)]
pub enum MessageStatus {
    Sent,
    Delivered,
    Read,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatParticipant {
    pub chat_id: i32,
    pub user_id: i32,
}

// DTOs for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDTO {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub profile_picture: Option<String>,
    pub status: Option<String>,
    pub is_online: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatDTO {
    pub id: i32,
    pub name: String,
    pub participants: Vec<UserDTO>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub last_message: Option<MessageDTO>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageDTO {
    pub id: i32,
    pub chat_id: i32,
    pub sender_id: i32,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub status: MessageStatus,
}

// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
pub struct WsMessage {
    pub message_type: String,
    pub data: serde_json::Value,
} 