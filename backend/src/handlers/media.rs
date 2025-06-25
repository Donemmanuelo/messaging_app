use crate::{
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::multipart::Multipart;
use serde::Serialize;
use std::fs::create_dir_all;
use std::io::Write;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;
use crate::middleware::AuthUser;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const ALLOWED_IMAGE_TYPES: [&str; 3] = ["image/jpeg", "image/png", "image/gif"];
const ALLOWED_VIDEO_TYPES: [&str; 2] = ["video/mp4", "video/webm"];
const ALLOWED_AUDIO_TYPES: [&str; 2] = ["audio/mpeg", "audio/wav"];

#[derive(Serialize)]
pub struct UploadResponse {
    pub url: String,
}

pub async fn upload_media(State(state): State<Arc<AppState>>, mut multipart: Multipart) -> impl IntoResponse {
    let upload_dir = "media_uploads";
    create_dir_all(upload_dir).unwrap();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().map(|s| s.to_string()).unwrap_or("file".to_string());
        let data = field.bytes().await.unwrap();
        let file_path = format!("{}/{}", upload_dir, file_name);
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(&data).unwrap();
        let url = format!("/media/{}", file_name);
        return (StatusCode::OK, Json(UploadResponse { url })).into_response();
    }
    (StatusCode::BAD_REQUEST, "No file uploaded").into_response()
}

pub async fn delete_media(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(media_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Get media info
    let media = sqlx::query!(
        r#"
        SELECT public_id, user_id FROM media
        WHERE id = $1
        "#,
        media_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch media info: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
        error!("Media not found: {}", media_id);
        StatusCode::NOT_FOUND
    })?;

    // Check ownership
    if media.user_id != auth_user.id {
        return Err(StatusCode::FORBIDDEN);
    }

    // Delete from Cloudinary
    // cloudinary::api::delete::Delete::new()
    //     .public_id(&media.public_id)
    //     .execute()
    //     .await
    //     .map_err(|e| {
    //         error!("Failed to delete from Cloudinary: {}", e);
    //         StatusCode::INTERNAL_SERVER_ERROR
    //     })?;

    // Delete from database
    sqlx::query!(
        r#"
        DELETE FROM media
        WHERE id = $1
        "#,
        media_id
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        error!("Failed to delete media from database: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Media deleted successfully: {}", media_id);
    Ok(StatusCode::OK)
}
