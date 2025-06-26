use axum::{routing::post, Router};
use std::sync::Arc;
use crate::AppState;
use crate::handlers::push::subscribe_push;

pub fn push_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/push/subscribe", post(subscribe_push))
} 