use axum::{routing::{post, get, delete}, Router};
use std::sync::Arc;
use crate::AppState;
use crate::handlers::status::{post_status, get_statuses, delete_status};

pub fn status_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/status", post(post_status))
        .route("/api/status", get(get_statuses))
        .route("/api/status/:id", delete(delete_status))
} 