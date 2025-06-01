use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use config::{Config as ConfigSource, ConfigError, File};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub server_address: String,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub cloudinary_cloud_name: String,
    pub cloudinary_api_key: String,
    pub cloudinary_api_secret: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        Ok(AppConfig {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost/whatsapp_clone".to_string()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            server_address: env::var("SERVER_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0:8000".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key".to_string()),
            jwt_expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()?,
            cloudinary_cloud_name: env::var("CLOUDINARY_CLOUD_NAME")
                .unwrap_or_else(|_| "your-cloud-name".to_string()),
            cloudinary_api_key: env::var("CLOUDINARY_API_KEY")
                .unwrap_or_else(|_| "your-api-key".to_string()),
            cloudinary_api_secret: env::var("CLOUDINARY_API_SECRET")
                .unwrap_or_else(|_| "your-api-secret".to_string()),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub frontend_url: String,
    pub port: u16,
    pub environment: Environment,
    pub cloudinary_cloud_name: String,
    pub cloudinary_api_key: String,
    pub cloudinary_api_secret: String,
    pub jwt_secret: String,
    pub rate_limit_requests: u32,
    pub rate_limit_window: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Production,
    Testing,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            redis_url: env::var("REDIS_URL")?,
            frontend_url: env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            environment: match env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()).as_str() {
                "production" => Environment::Production,
                "testing" => Environment::Testing,
                _ => Environment::Development,
            },
            cloudinary_cloud_name: env::var("CLOUDINARY_CLOUD_NAME")?,
            cloudinary_api_key: env::var("CLOUDINARY_API_KEY")?,
            cloudinary_api_secret: env::var("CLOUDINARY_API_SECRET")?,
            jwt_secret: env::var("JWT_SECRET")?,
            rate_limit_requests: env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
            rate_limit_window: env::var("RATE_LIMIT_WINDOW")
                .unwrap_or_else(|_| "60".to_string())
                .parse()?,
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }

    pub fn is_testing(&self) -> bool {
        self.environment == Environment::Testing
    }
}