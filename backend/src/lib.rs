use axum::{
    routing::{get, post, put, delete},
    Router,
};
use redis::Client as RedisClient;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

mod auth;
mod error;
mod handlers;
mod models;
mod websocket;
pub mod middleware;
pub mod services;
pub mod config;
pub mod database;

pub use auth::AuthUser;
pub use error::AppError;
use websocket::WebSocketManager;
use websocket::handler::ws_handler;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub redis: RedisClient,
    pub ws_tx: broadcast::Sender<String>,
    pub ws_manager: Arc<WebSocketManager>,
}

pub fn create_app(pool: PgPool, redis: RedisClient) -> Router<Arc<AppState>> {
    let (ws_tx, _) = broadcast::channel(100);
    let state = Arc::new(AppState {
        pool,
        redis,
        ws_tx,
        ws_manager: Arc::new(WebSocketManager::new()),
    });

    Router::new()
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/logout", post(handlers::auth::logout))
        .route("/auth/refresh", post(handlers::auth::refresh_token))
        .route("/users", get(handlers::users::get_users))
        .route("/users/:id", get(handlers::users::get_user))
        .route("/users/:id", put(handlers::users::update_user))
        .route("/users/:id/contacts", get(handlers::users::get_contacts))
        .route("/users/:id/contacts/:contact_id", post(handlers::users::add_contact))
        .route("/users/:id/contacts/:contact_id", delete(handlers::users::remove_contact))
        .route("/ws", get(ws_handler))
        .route("/media", post(handlers::media::upload_media))
        .route("/media/:id", delete(handlers::media::delete_media))
        .route("/groups", post(handlers::groups::create_group))
        .route("/groups/:id", get(handlers::groups::get_group))
        .route("/groups/:id", put(handlers::groups::update_group))
        .route("/groups/:id/members", get(handlers::groups::get_group_members))
        .route("/groups/:id/members/:user_id", post(handlers::groups::add_group_member))
        .route("/groups/:id/members/:user_id", delete(handlers::groups::remove_group_member))
        .route("/messages/:receiver_id", post(handlers::messages::send_message))
        .route("/messages/:receiver_id", get(handlers::messages::get_messages))
        .route("/messages/:id", put(handlers::messages::update_message))
        .route("/messages/:id", delete(handlers::messages::delete_message))
        .route("/groups/:id/messages", post(handlers::messages::send_group_message))
        .route("/groups/:id/messages", get(handlers::messages::get_group_messages))
        .layer(CorsLayer::permissive())
        .with_state(state)
} 