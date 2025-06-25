use crate::handlers::group::{create_group, fetch_groups, upload_group_avatar};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::AppState;

pub fn group_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/groups", post(create_group))
        .route("/api/groups", get(fetch_groups))
        .route("/api/groups/:group_id/avatar", post(upload_group_avatar))
}
