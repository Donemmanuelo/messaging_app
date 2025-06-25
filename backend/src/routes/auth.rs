use crate::handlers::auth::{login, register};
use axum::{routing::post, Router};
use std::sync::Arc;
use crate::AppState;

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/register", post(register))
        .route("/api/login", post(login))
}
