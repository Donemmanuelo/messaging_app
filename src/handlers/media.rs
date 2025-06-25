create_dir_all(upload_dir).map_err(|e| AppError::InternalServerError(format!("Failed to create upload dir: {}", e)))?;
while let Some(field) = multipart.next_field().await.map_err(|e| AppError::InternalServerError(format!("Failed to get next field: {}", e)))? {
    let data = field.bytes().await.map_err(|e| AppError::InternalServerError(format!("Failed to read field bytes: {}", e)))?;
    let mut file = std::fs::File::create(&file_path).map_err(|e| AppError::InternalServerError(format!("Failed to create file: {}", e)))?;
    file.write_all(&data).map_err(|e| AppError::InternalServerError(format!("Failed to write file: {}", e)))?;
} 