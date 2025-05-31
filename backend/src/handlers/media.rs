use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use std::path::Path;

use crate::services::AppState;
use crate::middleware::auth::AuthUser;

pub async fn upload_media(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Create uploads directory if it doesn't exist
    let upload_dir = Path::new("uploads");
    if !upload_dir.exists() {
        fs::create_dir_all(upload_dir).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    
    let mut file_paths = Vec::new();
    
    // Process each part of the multipart form
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let file_name = field.file_name().map(|s| s.to_string()).unwrap_or_else(|| {
            let uuid = Uuid::new_v4();
            format!("{}.bin", uuid)
        });
        
        // Determine content type
        let content_type = field.content_type().unwrap_or("application/octet-stream");
        let extension = match content_type {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "video/mp4" => "mp4",
            "audio/mpeg" => "mp3",
            "audio/ogg" => "ogg",
            _ => "bin",
        };
        
        // Create a unique filename
        let uuid = Uuid::new_v4();
        let file_path = format!("uploads/{}.{}", uuid, extension);
        
        // Read the file data
        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
        
        // Write the file to disk
        let mut file = fs::File::create(&file_path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        file.write_all(&data).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        // In a production environment, you would upload to S3 or Cloudinary instead
        // For now, we'll just return the local path
        file_paths.push(file_path);
    }
    
    // Return the file paths
    Ok(Json(json!({
        "success": true,
        "file_paths": file_paths
    })))
}