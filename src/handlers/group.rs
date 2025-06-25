use std::fs;
use std::io::Write;
use uuid::Uuid;

pub async fn handle_group_upload(multipart: &mut multipart::Multipart<impl AsyncRead + Unpin + Send>) -> Result<(), AppError> {
    let upload_dir = "uploads/group";
    let file_path = format!("{}/avatar.jpg", upload_dir);

    fs::create_dir_all(upload_dir).map_err(|e| AppError::InternalServerError(format!("Failed to create upload dir: {}", e)))?;

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::InternalServerError(format!("Failed to get next field: {}", e)))? {
        let filename = field.file_name().unwrap_or("avatar");
        let data = field.bytes().await.map_err(|e| AppError::InternalServerError(format!("Failed to read field bytes: {}", e)))?;
        let mut file = std::fs::File::create(&file_path).map_err(|e| AppError::InternalServerError(format!("Failed to create file: {}", e)))?;
        file.write_all(&data).map_err(|e| AppError::InternalServerError(format!("Failed to write file: {}", e)))?;
    }

    let _ = some_result.map_err(|e| AppError::InternalServerError(format!("Some error: {}", e)))?;

    Ok(())
} 