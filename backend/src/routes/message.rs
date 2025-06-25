use crate::handlers::message::{fetch_messages, send_message};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::AppState;

pub fn message_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/messages", post(send_message))
        .route("/api/messages/:chat_id", get(fetch_messages))
}
