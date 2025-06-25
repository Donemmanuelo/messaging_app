// If redis is wrapped in Arc<Mutex<...>>, lock before use:
// let mut redis = _state.redis.lock().unwrap();
// If not, and only immutable methods are needed, use as is without &mut or mutable borrow. 

// In AppState struct:
pub struct AppState {
    // ... existing code ...
    pub redis: tokio::sync::Mutex<redis::Client>,
    // ... existing code ...
}

// Before each redis call:
let mut redis = _state.redis.lock().await;
// Use redis.get(), redis.set_ex(), redis.del() as needed
// ... existing code ... 

create_dir_all(upload_dir).map_err(|e| AppError::InternalServerError(format!("Failed to create upload dir: {}", e)))?;
while let Some(field) = multipart.next_field().await.map_err(|e| AppError::InternalServerError(format!("Failed to get next field: {}", e)))? {
    let data = field.bytes().await.map_err(|e| AppError::InternalServerError(format!("Failed to read field bytes: {}", e)))?;
    let mut file = std::fs::File::create(&file_path).map_err(|e| AppError::InternalServerError(format!("Failed to create file: {}", e)))?;
    file.write_all(&data).map_err(|e| AppError::InternalServerError(format!("Failed to write file: {}", e)))?;
} 