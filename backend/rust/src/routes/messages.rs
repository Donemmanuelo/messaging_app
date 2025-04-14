use actix_web::{web, HttpResponse};
use serde_json::json;
use validator::Validate;

use crate::{
    models::{chat::{Chat, ChatCreate, ChatUpdate}, message::{Message, MessageCreate, MessageUpdate}},
    services::auth::AuthService,
    utils::{error::ApiError, validation::{validate_request, PaginationParams}},
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/chats")
                    .route("", web::get().to(get_chats))
                    .route("", web::post().to(create_chat))
                    .route("/{chat_id}", web::get().to(get_chat))
                    .route("/{chat_id}", web::put().to(update_chat))
                    .route("/{chat_id}", web::delete().to(delete_chat))
                    .route("/{chat_id}/messages", web::get().to(get_messages))
                    .route("/{chat_id}/messages", web::post().to(create_message))
                    .route("/{chat_id}/messages/{message_id}", web::put().to(update_message))
                    .route("/{chat_id}/messages/{message_id}", web::delete().to(delete_message)),
            ),
    );
}

async fn get_chats(
    auth: AuthService,
    pagination: web::Query<PaginationParams>,
) -> Result<HttpResponse, ApiError> {
    validate_request(&pagination)?;
    let chats = auth.get_user_chats(pagination.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({
        "data": chats
    })))
}

async fn create_chat(
    auth: AuthService,
    chat_data: web::Json<ChatCreate>,
) -> Result<HttpResponse, ApiError> {
    validate_request(&chat_data)?;
    let chat = auth.create_chat(chat_data.into_inner()).await?;
    Ok(HttpResponse::Created().json(json!({
        "data": chat
    })))
}

async fn get_chat(
    auth: AuthService,
    chat_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let chat = auth.get_chat(chat_id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({
        "data": chat
    })))
}

async fn update_chat(
    auth: AuthService,
    chat_id: web::Path<String>,
    chat_data: web::Json<ChatUpdate>,
) -> Result<HttpResponse, ApiError> {
    validate_request(&chat_data)?;
    let chat = auth.update_chat(chat_id.into_inner(), chat_data.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({
        "data": chat
    })))
}

async fn delete_chat(
    auth: AuthService,
    chat_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    auth.delete_chat(chat_id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Chat deleted successfully"
    })))
}

async fn get_messages(
    auth: AuthService,
    chat_id: web::Path<String>,
    pagination: web::Query<PaginationParams>,
) -> Result<HttpResponse, ApiError> {
    validate_request(&pagination)?;
    let messages = auth.get_chat_messages(chat_id.into_inner(), pagination.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({
        "data": messages
    })))
}

async fn create_message(
    auth: AuthService,
    chat_id: web::Path<String>,
    message_data: web::Json<MessageCreate>,
) -> Result<HttpResponse, ApiError> {
    validate_request(&message_data)?;
    let message = auth.create_message(chat_id.into_inner(), message_data.into_inner()).await?;
    Ok(HttpResponse::Created().json(json!({
        "data": message
    })))
}

async fn update_message(
    auth: AuthService,
    path: web::Path<(String, String)>,
    message_data: web::Json<MessageUpdate>,
) -> Result<HttpResponse, ApiError> {
    validate_request(&message_data)?;
    let (chat_id, message_id) = path.into_inner();
    let message = auth.update_message(chat_id, message_id, message_data.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({
        "data": message
    })))
}

async fn delete_message(
    auth: AuthService,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (chat_id, message_id) = path.into_inner();
    auth.delete_message(chat_id, message_id).await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Message deleted successfully"
    })))
} 