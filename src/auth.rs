let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| AppError::InternalServerError("JWT_SECRET must be set".to_string()))?;
let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());

let expiration = chrono::Utc::now()
    .checked_add_signed(chrono::Duration::hours(24))
    .ok_or(AppError::InternalServerError("valid timestamp error".to_string()))?
    .timestamp() as usize;

let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| AppError::InternalServerError("JWT_SECRET must be set".to_string()))?;
let encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes()); 