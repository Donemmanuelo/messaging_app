use axum::{Router, routing::{post, get}};
use std::sync::Arc;
use crate::AppState;
use crate::handlers::call_log::{log_call, get_call_history};

pub fn call_log_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/call_logs", post(log_call))
        .route("/api/call_logs/:user_id", get(get_call_history))
} 