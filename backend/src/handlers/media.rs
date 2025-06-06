use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::multipart::Multipart;
use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    AppState,
    error::AppError,
    models::{MediaUploadResponse, MediaType},
    auth::Claims,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use cloudinary::api::upload::Upload;
use cloudinary::api::delete::Delete;
use cloudinary::Cloudinary;
use tracing::{error, info};

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const ALLOWED_IMAGE_TYPES: [&str; 3] = ["image/jpeg", "image/png", "image/gif"];
const ALLOWED_VIDEO_TYPES: [&str; 2] = ["video/mp4", "video/webm"];
const ALLOWED_AUDIO_TYPES: [&str; 2] = ["audio/mpeg", "audio/wav"];

pub async fn upload_media(
    State(pool): State<PgPool>,
    claims: Claims,
    mut multipart: Multipart,
) -> Result<Json<MediaUploadResponse>, AppError> {
    let cloudinary = Cloudinary::new(
        std::env::var("CLOUDINARY_CLOUD_NAME").unwrap_or_default(),
        std::env::var("CLOUDINARY_API_KEY").unwrap_or_default(),
        std::env::var("CLOUDINARY_API_SECRET").unwrap_or_default(),
    );

    let mut file = None;
    let mut media_type = None;
    let mut content_type = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to parse multipart form: {}", e);
        AppError::BadRequest("Failed to parse multipart form".into())
    })? {
        let name = field.name().unwrap_or_default();
        match name {
            "file" => {
                let data = field.bytes().await.map_err(|_| AppError::BadRequest("Failed to read file data".into()))?;
                content_type = field.content_type().map(|s| s.to_string());
                file = Some(data);
            }
            "type" => {
                let type_str = field.text().await.map_err(|_| AppError::BadRequest("Failed to read media type".into()))?;
                media_type = Some(type_str);
            }
            _ => {}
        }
    }

    let file = file.ok_or_else(|| {
        error!("No file provided in the request");
        AppError::BadRequest("No file provided".into())
    })?;
    let media_type = media_type.ok_or_else(|| {
        error!("No media type provided in the request");
        AppError::BadRequest("No media type provided".into())
    })?;
    let content_type = content_type.ok_or_else(|| {
        error!("No content type provided in the request");
        AppError::BadRequest("No content type provided".into())
    })?;

    // Validate file size
    if file.len() > MAX_FILE_SIZE {
        error!("File size exceeds maximum limit: {} bytes", file.len());
        return Err(AppError::BadRequest(format!(
            "File size exceeds maximum limit of {}MB",
            MAX_FILE_SIZE / (1024 * 1024)
        )));
    }

    // Validate content type
    let is_valid_type = match media_type.as_str() {
        "image" => ALLOWED_IMAGE_TYPES.contains(&content_type.as_str()),
        "video" => ALLOWED_VIDEO_TYPES.contains(&content_type.as_str()),
        "audio" => ALLOWED_AUDIO_TYPES.contains(&content_type.as_str()),
        _ => false,
    };

    if !is_valid_type {
        error!("Invalid content type {} for media type {}", content_type, media_type);
        return Err(AppError::BadRequest(format!(
            "Invalid content type {} for media type {}",
            content_type, media_type
        )));
    }

    // Upload to Cloudinary
    let resource_type = match media_type.as_str() {
        "image" => "image",
        "video" => "video",
        "audio" => "raw",
        _ => "auto",
    };

    let upload_result = cloudinary.upload()
        .resource_type(resource_type)
        .file(&file)
        .execute()
        .await
        .map_err(|e| {
            error!("Failed to upload to Cloudinary: {}", e);
            AppError::InternalServerError(format!("Failed to upload to Cloudinary: {}", e))
        })?;

    // Store media info in database
    let media_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO media (id, user_id, type, url, public_id, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        "#,
        media_id,
        claims.sub,
        media_type,
        upload_result.secure_url,
        upload_result.public_id
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("Failed to store media info: {}", e);
        AppError::InternalServerError(format!("Failed to store media info: {}", e))
    })?;

    info!("Media uploaded successfully: {}", media_id);
    Ok(Json(MediaUploadResponse {
        id: media_id,
        url: upload_result.secure_url,
        type_: media_type,
    }))
}

pub async fn delete_media(
    State(pool): State<PgPool>,
    claims: Claims,
    Path(media_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Get media info
    let media = sqlx::query!(
        r#"
        SELECT public_id, user_id FROM media
        WHERE id = $1
        "#,
        media_id
    )
    .fetch_optional(&pool)
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
    if media.user_id != claims.sub {
        return Err(StatusCode::FORBIDDEN);
    }

    // Delete from Cloudinary
    let cloudinary = Cloudinary::new(
        std::env::var("CLOUDINARY_CLOUD_NAME").unwrap_or_default(),
        std::env::var("CLOUDINARY_API_KEY").unwrap_or_default(),
        std::env::var("CLOUDINARY_API_SECRET").unwrap_or_default(),
    );

    cloudinary.delete()
        .public_id(&media.public_id)
        .execute()
        .await
        .map_err(|e| {
            error!("Failed to delete from Cloudinary: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Delete from database
    sqlx::query!(
        r#"
        DELETE FROM media
        WHERE id = $1
        "#,
        media_id
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("Failed to delete media from database: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Media deleted successfully: {}", media_id);
    Ok(StatusCode::OK)
}