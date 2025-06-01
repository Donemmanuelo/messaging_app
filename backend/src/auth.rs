use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use redis::{Client as RedisClient, Commands};

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
}

impl AuthUser {
    pub fn from_token(token: &str, redis: &RedisClient) -> Result<Self, AppError> {
        // First check if the token is blacklisted in Redis
        let mut conn = redis.get_connection()?;
        let blacklisted: bool = conn.get(format!("blacklist:{}", token))?;
        if blacklisted {
            return Err(AppError::Unauthorized("Token has been revoked".into()));
        }

        // Verify the token
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_bytes()),
            &Validation::default(),
        )?;

        // Parse the user ID from the token
        let user_id = Uuid::parse_str(&token_data.claims.sub)?;

        Ok(AuthUser { id: user_id })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Missing Authorization header".to_string(),
                )
                    .into_response()
            })?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid Authorization header format".to_string(),
                )
                    .into_response()
            })?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Invalid token".to_string(),
            )
                .into_response()
        })?;

        let user_id = Uuid::parse_str(&token_data.claims.sub).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Invalid user ID in token".to_string(),
            )
                .into_response()
        })?;

        Ok(AuthUser { id: user_id })
    }
}

pub fn create_token(user_id: Uuid) -> Result<String, AppError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_bytes()),
    )?;

    Ok(token)
} 