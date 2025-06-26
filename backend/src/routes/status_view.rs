use axum::{Router, routing::{post, get}};
use std::sync::Arc;
use crate::AppState;
use crate::handlers::status_view::{mark_status_seen, get_status_views};

pub fn status_view_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/status_views", post(mark_status_seen))
        .route("/api/status_views/:user_id", get(get_status_views))
} 