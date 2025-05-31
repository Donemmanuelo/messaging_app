use crate::models::*;
use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: usize,
}

pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub websocket_tx: broadcast::Sender<WebSocketMessage>,
    pub online_users: Arc<RwLock<HashMap<Uuid, tokio::sync::mpsc::UnboundedSender<String>>>>,
}

impl AppState {
    pub async fn new(database_url: &str) -> Result<Self> {
        let db = PgPool::connect(database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&db).await?;
        
        let (websocket_tx, _) = broadcast::channel(1000);
        let online_users = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            db,
            jwt_secret: "your-secret-key".to_string(),
            websocket_tx,
            online_users,
        })
    }

    pub fn generate_token(&self, user_id: Uuid) -> Result<String> {
        let claims = Claims {
            sub: user_id.to_string(),
            exp: (Utc::now().timestamp() + 86400) as usize, // 24 hours
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<TokenData<Claims>> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data)
    }

    pub async fn create_user(&self, req: RegisterRequest) -> Result<User> {
        let password_hash = hash(req.password, DEFAULT_COST)?;
        let user_id = Uuid::new_v4();

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, username, email, password_hash, display_name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            RETURNING *
            "#,
            user_id,
            req.username,
            req.email,
            password_hash,
            req.display_name
        )
        .fetch_one(&self.db)
        .await?;

        Ok(user)
    }

    pub async fn authenticate_user(&self, email: &str, password: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.db)
        .await?;

        if let Some(user) = user {
            if verify(password, &user.password_hash)? {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    pub async fn get_user_chats(&self, user_id: Uuid) -> Result<Vec<ChatResponse>> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT
                c.id as chat_id,
                c.chat_type,
                c.name as chat_name,
                c.avatar_url as chat_avatar,
                c.updated_at,
                m.id as message_id,
                m.content as message_content,
                m.created_at as message_created_at,
                sender.id as sender_id,
                sender.username as sender_username,
                sender.display_name as sender_display_name,
                sender.avatar_url as sender_avatar
            FROM chats c
            INNER JOIN chat_participants cp ON c.id = cp.chat_id
            LEFT JOIN messages m ON c.id = m.chat_id 
                AND m.created_at = (
                    SELECT MAX(created_at) 
                    FROM messages 
                    WHERE chat_id = c.id
                )
            LEFT JOIN users sender ON m.sender_id = sender.id
            WHERE cp.user_id = $1
            ORDER BY c.updated_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await?;

        let mut chats = Vec::new();
        for row in rows {
            let participants = self.get_chat_participants(row.chat_id).await?;
            
            let last_message = if let Some(message_id) = row.message_id {
                Some(MessageResponse {
                    id: message_id,
                    chat_id: row.chat_id,
                    sender: UserResponse {
                        id: row.sender_id.unwrap(),
                        username: row.sender_username.unwrap(),
                        email: String::new(), // Don't expose email in chat responses
                        display_name: row.sender_display_name,
                        avatar_url: row.sender_avatar,
                        status: None,
                        is_online: false,
                        last_seen: None,
                    },
                    content: row.message_content,
                    message_type: "text".to_string(),
                    media_url: None,
                    reply_to: None,
                    status: "sent".to_string(),
                    created_at: row.message_created_at.unwrap(),
                })
            } else {
                None
            };

            chats.push(ChatResponse {
                id: row.chat_id,
                chat_type: row.chat_type,
                name: row.chat_name,
                avatar_url: row.chat_avatar,
                participants,
                last_message,
                unread_count: 0, // TODO: Implement unread count
                updated_at: row.updated_at,
            });
        }

        Ok(chats)
    }

    pub async fn get_chat_participants(&self, chat_id: Uuid) -> Result<Vec<UserResponse>> {
        let participants = sqlx::query_as!(
            User,
            r#"
            SELECT u.* FROM users u
            INNER JOIN chat_participants cp ON u.id = cp.user_id
            WHERE cp.chat_id = $1
            "#,
            chat_id
        )
        .fetch_all(&self.db)
        .await?;

        Ok(participants.into_iter().map(|u| UserResponse {
            id: u.id,
            username: u.username,
            email: u.email,
            display_name: u.display_name,
            avatar_url: u.avatar_url,
            status: u.status,
            is_online: u.is_online,
            last_seen: u.last_seen,
        }).collect())
    }

    pub async fn get_chat_messages(&self, chat_id: Uuid) -> Result<Vec<MessageResponse>> {
        let messages = sqlx::query!(
            r#"
            SELECT 
                m.*,
                sender.id as sender_id,
                sender.username as sender_username,
                sender.display_name as sender_display_name,
                sender.avatar_url as sender_avatar
            FROM messages m
            INNER JOIN users sender ON m.sender_id = sender.id
            WHERE m.chat_id = $1
            ORDER BY m.created_at ASC
            "#,
            chat_id
        )
        .fetch_all(&self.db)
        .await?;

        Ok(messages.into_iter().map(|m| MessageResponse {
            id: m.id,
            chat_id: m.chat_id,
            sender: UserResponse {
                id: m.sender_id,
                username: m.sender_username,
                email: String::new(),
                display_name: m.sender_display_name,
                avatar_url: m.sender_avatar,
                status: None,
                is_online: false,
                last_seen: None,
            },
            content: m.content,
            message_type: m.message_type,
            media_url: m.media_url,
            reply_to: None, // TODO: Implement reply_to
            status: m.status,
            created_at: m.created_at,
        }).collect())
    }

    pub async fn send_message(&self, chat_id: Uuid, sender_id: Uuid, req: SendMessageRequest) -> Result<MessageResponse> {
        let message_id = Uuid::new_v4();
        
        let message = sqlx::query!(
            r#"
            INSERT INTO messages (id, chat_id, sender_id, content, message_type, media_url, reply_to, status, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'sent', NOW())
            RETURNING *
            "#,
            message_id,
            chat_id,
            sender_id,
            req.content,
            req.message_type,
            req.media_url,
            req.reply_to
        )
        .fetch_one(&self.db)
        .await?;

        // Update chat's updated_at timestamp
        sqlx::query!(
            "UPDATE chats SET updated_at = NOW() WHERE id = $1",
            chat_id
        )
        .execute(&self.db)
        .await?;

        // Get sender info
        let sender = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            sender_id
        )
        .fetch_one(&self.db)
        .await?;

        let message_response = MessageResponse {
            id: message.id,
            chat_id: message.chat_id,
            sender: UserResponse {
                id: sender.id,
                username: sender.username,
                email: String::new(),
                display_name: sender.display_name,
                avatar_url: sender.avatar_url,
                status: sender.status,
                is_online: sender.is_online,
                last_seen: sender.last_seen,
            },
            content: message.content,
            message_type: message.message_type,
            media_url: message.media_url,
            reply_to: None,
            status: message.status,
            created_at: message.created_at,
        };

        // Broadcast message via WebSocket
        let ws_message = WebSocketMessage {
            message_type: "new_message".to_string(),
            chat_id: Some(chat_id),
            sender_id,
            content: req.content,
            timestamp: Utc::now(),
            is_typing: None,
        };

        let _ = self.websocket_tx.send(ws_message);

        Ok(message_response)
    }

    pub async fn create_chat(&self, creator_id: Uuid, req: CreateChatRequest) -> Result<ChatResponse> {
        let chat_id = Uuid::new_v4();
        
        let mut tx = self.db.begin().await?;

        // Create chat
        let chat = sqlx::query!(
            r#"
            INSERT INTO chats (id, chat_type, name, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING *
            "#,
            chat_id,
            req.chat_type,
            req.name,
            creator_id
        )
        .fetch_one(&mut *tx)
        .await?;

        // Add creator as participant
        sqlx::query!(
            r#"
            INSERT INTO chat_participants (chat_id, user_id, joined_at, is_admin)
            VALUES ($1, $2, NOW(), true)
            "#,
            chat_id,
            creator_id
        )
        .execute(&mut *tx)
        .await?;

        // Add other participants
        for participant_id in req.participant_ids {
            sqlx::query!(
                r#"
                INSERT INTO chat_participants (chat_id, user_id, joined_at, is_admin)
                VALUES ($1, $2, NOW(), false)
                "#,
                chat_id,
                participant_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        let participants = self.get_chat_participants(chat_id).await?;

        Ok(ChatResponse {
            id: chat.id,
            chat_type: chat.chat_type,
            name: chat.name,
            avatar_url: chat.avatar_url,
            participants,
            last_message: None,
            unread_count: 0,
            updated_at: chat.updated_at,
        })
    }
}