use crate::handlers::users::{
    get_public_key, get_users_with_status, update_profile, upload_public_key, upload_user_avatar,
};
use axum::{
    routing::{get, patch, post},
    Router,
};
use std::sync::Arc;
use crate::AppState;

pub fn users_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/users", get(get_users_with_status))
        .route("/api/users/:user_id", patch(update_profile))
        .route("/api/users/:user_id/avatar", post(upload_user_avatar))
        .route("/api/users/:user_id/public_key", post(upload_public_key))
        .route("/api/users/:user_id/public_key", get(get_public_key))
}
