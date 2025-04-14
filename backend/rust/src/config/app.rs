use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_expiry: i64,
    pub refresh_token_expiry: i64,
    pub verification_token_expiry: i64,
    pub password_reset_token_expiry: i64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            jwt_secret: "your-secret-key".to_string(),
            jwt_expiry: 3600, // 1 hour
            refresh_token_expiry: 2592000, // 30 days
            verification_token_expiry: 604800, // 7 days
            password_reset_token_expiry: 3600, // 1 hour
        }
    }
} 