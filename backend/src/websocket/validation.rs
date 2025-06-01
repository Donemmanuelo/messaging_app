use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::AppError;
use crate::models::message::Message;

pub const MAX_MESSAGE_LENGTH: usize = 4096; // 4KB
pub const MAX_EMOJI_LENGTH: usize = 8; // Maximum length for emoji reactions
pub const MAX_STATUS_LENGTH: usize = 128; // Maximum length for user status

#[derive(Debug, Serialize, Deserialize)]
pub enum WebSocketMessage {
    DirectMessage(Message),
    GroupMessage {
        group_id: Uuid,
        message: Message,
    },
    Typing {
        user_id: Uuid,
        chat_id: Uuid,
    },
    GroupTyping {
        group_id: Uuid,
        user_id: Uuid,
    },
    Read {
        user_id: Uuid,
        chat_id: Uuid,
        message_id: Uuid,
    },
    GroupRead {
        group_id: Uuid,
        user_id: Uuid,
        message_id: Uuid,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketError {
    pub code: String,
    pub message: String,
}

impl WebSocketError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<WebSocketError>,
}

impl<T> WebSocketResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: &str, message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(WebSocketError::new(code, message)),
        }
    }
}

impl WebSocketMessage {
    pub fn validate(&self) -> Result<(), AppError> {
        match self {
            WebSocketMessage::DirectMessage(message) => {
                if message.content.is_empty() {
                    return Err(AppError::BadRequest("Message content cannot be empty".into()));
                }
                if message.content.len() > MAX_MESSAGE_LENGTH {
                    return Err(AppError::BadRequest(format!(
                        "Message content exceeds maximum length of {} characters",
                        MAX_MESSAGE_LENGTH
                    )));
                }
                if let Some(url) = &message.media_url {
                    if url.len() > MAX_MESSAGE_LENGTH {
                        return Err(AppError::BadRequest(format!(
                            "Media URL exceeds maximum length of {} characters",
                            MAX_MESSAGE_LENGTH
                        )));
                    }
                }
            }
            WebSocketMessage::GroupMessage { message, .. } => {
                if message.content.is_empty() {
                    return Err(AppError::BadRequest("Message content cannot be empty".into()));
                }
                if message.content.len() > MAX_MESSAGE_LENGTH {
                    return Err(AppError::BadRequest(format!(
                        "Message content exceeds maximum length of {} characters",
                        MAX_MESSAGE_LENGTH
                    )));
                }
                if let Some(url) = &message.media_url {
                    if url.len() > MAX_MESSAGE_LENGTH {
                        return Err(AppError::BadRequest(format!(
                            "Media URL exceeds maximum length of {} characters",
                            MAX_MESSAGE_LENGTH
                        )));
                    }
                }
            }
            WebSocketMessage::MessageReaction { emoji, .. } => {
                if emoji.is_empty() {
                    return Err(AppError::BadRequest("Emoji cannot be empty".into()));
                }
                if emoji.len() > MAX_EMOJI_LENGTH {
                    return Err(AppError::BadRequest(format!(
                        "Emoji exceeds maximum length of {} characters",
                        MAX_EMOJI_LENGTH
                    )));
                }
            }
            WebSocketMessage::UserStatus { status, .. } => {
                if status.len() > MAX_STATUS_LENGTH {
                    return Err(AppError::BadRequest(format!(
                        "Status exceeds maximum length of {} characters",
                        MAX_STATUS_LENGTH
                    )));
                }
            }
            _ => {}
        }
        Ok(())
    }
} 