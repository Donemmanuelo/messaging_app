use crate::handlers::group::{create_group, fetch_groups, upload_group_avatar};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::AppState;
use axum::{Json, extract::{Path, State}};
use serde::Deserialize;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

pub fn group_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/groups", post(create_group))
        .route("/api/groups", get(fetch_groups))
        .route("/api/groups/:group_id/avatar", post(upload_group_avatar))
}

#[derive(Deserialize)]
pub struct StartGroupCall {
    pub call_type: String, // "audio" or "video"
}

pub async fn start_group_call(
    Path(group_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<StartGroupCall>,
) -> impl IntoResponse {
    // Optionally: log the call, notify group, etc.
    // For now, just return success
    (StatusCode::OK, Json(json!({"status": "started", "group_id": group_id, "call_type": payload.call_type})))
}
