use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use crate::models::*;
use crate::schema::*;
use chrono::Utc;
use crate::services::auth::{hash_password, verify_password};
use sqlx::{postgres::PgPoolOptions, PgPool, Error as SqlxError};
use uuid::Uuid;
use std::sync::Arc;
use crate::{
    models::{
        user::{User, UserCreate, UserUpdate},
        chat::{Chat, ChatCreate, ChatUpdate},
        message::{Message, MessageCreate, MessageUpdate},
    },
    utils::error::ApiError,
};

pub struct DbService {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DbService {
    pub fn new(database_url: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create database pool");
        
        DbService { pool }
    }

    // User operations
    pub fn create_user(&self, new_user: &NewUser) -> Result<User, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();
        let hashed_password = hash_password(&new_user.password);
        let user = NewUser {
            username: new_user.username.clone(),
            email: new_user.email.clone(),
            password: hashed_password,
        };
        diesel::insert_into(users::table)
            .values(&user)
            .get_result(&mut conn)
    }

    pub fn get_user(&self, user_id: i32) -> Result<User, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();
        users::table.find(user_id).first(&mut conn)
    }

    pub fn get_user_by_email(&self, email: &str) -> Result<User, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();
        users::table
            .filter(users::email.eq(email))
            .first(&mut conn)
    }

    pub fn verify_user_credentials(&self, email: &str, password: &str) -> Result<User, diesel::result::Error> {
        let user = self.get_user_by_email(email)?;
        if verify_password(password, &user.password) {
            Ok(user)
        } else {
            Err(diesel::result::Error::NotFound)
        }
    }

    // Chat operations
    pub fn create_chat(&self, new_chat: &NewChat) -> Result<Chat, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();
        diesel::insert_into(chats::table)
            .values(new_chat)
            .get_result(&mut conn)
    }

    pub fn get_chat(&self, chat_id: i32) -> Result<Chat, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();
        chats::table.find(chat_id).first(&mut conn)
    }

    pub fn get_user_chats(&self, user_id: i32) -> Result<Vec<Chat>, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();
        chat_participants::table
            .filter(chat_participants::user_id.eq(user_id))
            .inner_join(chats::table)
            .select(chats::all_columns)
            .load(&mut conn)
    }

    // Message operations
    pub fn create_message(&self, new_message: &NewMessage) -> Result<Message, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();
        diesel::insert_into(messages::table)
            .values(new_message)
            .get_result(&mut conn)
    }

    pub fn get_chat_messages(&self, chat_id: i32) -> Result<Vec<Message>, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();
        messages::table
            .filter(messages::chat_id.eq(chat_id))
            .order(messages::timestamp.desc())
            .load(&mut conn)
    }

    // Chat participant operations
    pub fn add_participant(&self, chat_id: i32, user_id: i32) -> Result<(), diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();
        let participant = ChatParticipant { chat_id, user_id };
        diesel::insert_into(chat_participants::table)
            .values(&participant)
            .execute(&mut conn)?;
        Ok(())
    }

    pub fn get_chat_participants(&self, chat_id: i32) -> Result<Vec<User>, diesel::result::Error> {
        let mut conn = self.pool.get().unwrap();
        chat_participants::table
            .filter(chat_participants::chat_id.eq(chat_id))
            .inner_join(users::table)
            .select(users::all_columns)
            .load(&mut conn)
    }
}

#[derive(Clone)]
pub struct Database {
    pool: Arc<PgPool>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, SqlxError> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .connect(database_url)
            .await?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    // User operations
    pub async fn create_user(&self, user: UserCreate) -> Result<User, ApiError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, username, password_hash, is_verified, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            id,
            user.email,
            user.username,
            user.password,
            false,
            now,
            now
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| ApiError::new(500, format!("Database error: {}", e)))
        .map(|row| User {
            id: row.id,
            email: row.email,
            username: row.username,
            password_hash: row.password_hash,
            is_verified: row.is_verified,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, ApiError> {
        sqlx::query!(
            r#"
            SELECT * FROM users WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| ApiError::new(500, format!("Database error: {}", e)))
        .map(|row| row.map(|r| User {
            id: r.id,
            email: r.email,
            username: r.username,
            password_hash: r.password_hash,
            is_verified: r.is_verified,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    pub async fn update_user(&self, id: &str, user: UserUpdate) -> Result<User, ApiError> {
        let now = Utc::now();
        
        sqlx::query!(
            r#"
            UPDATE users
            SET username = COALESCE($1, username),
                email = COALESCE($2, email),
                password_hash = COALESCE($3, password_hash),
                updated_at = $4
            WHERE id = $5
            RETURNING *
            "#,
            user.username,
            user.email,
            user.password,
            now,
            id
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| ApiError::new(500, format!("Database error: {}", e)))
        .map(|row| User {
            id: row.id,
            email: row.email,
            username: row.username,
            password_hash: row.password_hash,
            is_verified: row.is_verified,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    // Chat operations
    pub async fn create_chat(&self, chat: ChatCreate) -> Result<Chat, ApiError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        sqlx::query!(
            r#"
            INSERT INTO chats (id, name, is_group, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            id,
            chat.name,
            chat.is_group,
            now,
            now
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| ApiError::new(500, format!("Database error: {}", e)))
        .map(|row| Chat {
            id: row.id,
            name: row.name,
            is_group: row.is_group,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn get_chat(&self, id: &str) -> Result<Option<Chat>, ApiError> {
        sqlx::query!(
            r#"
            SELECT * FROM chats WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| ApiError::new(500, format!("Database error: {}", e)))
        .map(|row| row.map(|r| Chat {
            id: r.id,
            name: r.name,
            is_group: r.is_group,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    // Message operations
    pub async fn create_message(&self, message: MessageCreate) -> Result<Message, ApiError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        sqlx::query!(
            r#"
            INSERT INTO messages (id, chat_id, sender_id, content, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            id,
            message.chat_id,
            message.sender_id,
            message.content,
            now,
            now
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| ApiError::new(500, format!("Database error: {}", e)))
        .map(|row| Message {
            id: row.id,
            chat_id: row.chat_id,
            sender_id: row.sender_id,
            content: row.content,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn get_messages(
        &self,
        chat_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>, ApiError> {
        sqlx::query!(
            r#"
            SELECT * FROM messages
            WHERE chat_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            chat_id,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| ApiError::new(500, format!("Database error: {}", e)))
        .map(|rows| rows.into_iter().map(|row| Message {
            id: row.id,
            chat_id: row.chat_id,
            sender_id: row.sender_id,
            content: row.content,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }).collect())
    }
} 