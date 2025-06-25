use crate::handlers::media::upload_media;
use axum::{routing::post, Router};
use std::sync::Arc;
use crate::AppState;

pub fn media_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/media/upload", post(upload_media))
}
