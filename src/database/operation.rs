use diesel::prelude::*;
use crate::database::db_client::Pool;
use crate::database::models::user::User;
use crate::database::models::message::Message;
use crate::database::models::chat::Chat;
use chrono::Utc;
use log::{info, error};

pub async fn create_user(user: User, pool: &Pool) -> Result<User, diesel::result::Error> {
    use crate::schema::users;

    let conn = pool.get().expect("Failed to get connection");
    info!("Creating user: {}", user.username);
    match diesel::insert_into(users::table)
        .values(&user)
        .get_result(&conn) {
        Ok(new_user) => {
            info!("User created successfully: {}", new_user.username);
            Ok(new_user)
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            Err(e)
        }
    }
}

pub async fn get_user_by_username(username: &str, pool: &Pool) -> Result<Option<User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let conn = pool.get().expect("Failed to get connection");
    info!("Fetching user by username: {}", username);
    match users.filter(username.eq(username)).first(&conn).optional() {
        Ok(Some(user)) => {
            info!("User found: {}", user.username);
            Ok(Some(user))
        }
        Ok(None) => {
            info!("User not found: {}", username);
            Ok(None)
        }
        Err(e) => {
            error!("Failed to fetch user: {}", e);
            Err(e)
        }
    }
}

pub async fn create_message(message: Message, pool: &Pool) -> Result<Message, diesel::result::Error> {
    use crate::schema::messages;

    let conn = pool.get().expect("Failed to get connection");
    info!("Creating message from user {} to user {}", message.from_user_id, message.to_user_id);
    match diesel::insert_into(messages::table)
        .values(&message)
        .get_result(&conn) {
        Ok(new_message) => {
            info!("Message created successfully: {}", new_message.content);
            Ok(new_message)
        }
        Err(e) => {
            error!("Failed to create message: {}", e);
            Err(e)
        }
    }
}

pub async fn get_messages_by_chat(chat_id: i32, pool: &Pool) -> Result<Vec<Message>, diesel::result::Error> {
    use crate::schema::messages::dsl::*;

    let conn = pool.get().expect("Failed to get connection");
    info!("Fetching messages for chat ID: {}", chat_id);
    match messages.filter(chat_id.eq(&chat_id)).load(&conn) {
        Ok(messages) => {
            info!("Fetched {} messages for chat ID: {}", messages.len(), chat_id);
            Ok(messages)
        }
        Err(e) => {
            error!("Failed to fetch messages: {}", e);
            Err(e)
        }
    }
}

pub async fn create_chat(chat: Chat, pool: &Pool) -> Result<Chat, diesel::result::Error> {
    use crate::schema::chats;

    let conn = pool.get().expect("Failed to get connection");
    info!("Creating chat between user {} and user {}", chat.user1_id, chat.user2_id);
    match diesel::insert_into(chats::table)
        .values(&chat)
        .get_result(&conn) {
        Ok(new_chat) => {
            info!("Chat created successfully: {}", new_chat.id.unwrap());
            Ok(new_chat)
        }
        Err(e) => {
            error!("Failed to create chat: {}", e);
            Err(e)
        }
    }
}