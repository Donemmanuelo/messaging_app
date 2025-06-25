use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    auth::{create_token, AuthUser},
    error::AppError,
    models::user::User,
    models::{CreateUserRequest, UserResponse},
    AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub user: UserResponse,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check if user already exists
    let existing_user = sqlx::query("SELECT 1 FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.pool)
        .await?;

    if existing_user.is_some() {
        return Err(AppError::BadRequest("User already exists".to_string()));
    }

    // Enforce password policy
    let password = req.password.as_str();
    if password.len() < 12
        || !password.chars().any(|c| c.is_uppercase())
        || !password.chars().any(|c| c.is_lowercase())
        || !password.chars().any(|c| c.is_ascii_digit())
        || !password.chars().any(|c| !c.is_alphanumeric())
    {
        return Err(AppError::BadRequest(
            "Password must be at least 12 characters and include uppercase, lowercase, number, and special character.".to_string(),
        ));
    }

    // Hash password
    let hashed_password = hash(req.password.as_bytes(), DEFAULT_COST)?;

    // Create user
    // This block of code is responsible for creating a new user in the database during the registration process.
    // It executes an SQL INSERT statement to add a new row to the "users" table with the provided username, email,
    // hashed password, and display name. Some fields like avatar_url, status, last_seen, and private_key_encrypted
    // are set to None (NULL in the database) for now. The function then returns the newly created user record
    // (including all its fields) as a User struct by using the RETURNING clause, and awaits the result from the database.
    let user = sqlx::query_as!(
        User,
        r#"
            INSERT INTO users (
                id, username, email, password_hash, display_name, avatar_url, status, last_seen, is_online, public_key, private_key_encrypted, created_at, updated_at
            )
            VALUES (
                gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW()
            )
            RETURNING
                id, username, email, password_hash, display_name, avatar_url, status, last_seen, is_online as "is_online!", public_key, private_key_encrypted, created_at, updated_at
        "#,
        req.username,
        req.email,
        &hashed_password,
        req.display_name,
        None::<String>, // avatar_url
        None::<String>, // status
        None::<chrono::DateTime<chrono::Utc>>, // last_seen
        false, // is_online
        None::<String>, // public_key
        None::<String>, // private_key_encrypted
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert user: {:?}", e);
        AppError::InternalServerError("Failed to create user".to_string())
    })?;
    let token = create_token(user.id)?;

    Ok((
        StatusCode::CREATED,
        Json(TokenResponse {
            token,
            user: user.into(),
        }),
    ))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Get user by email
    let user = sqlx::query_as!(
        User,
        r#"
            SELECT 
                id, 
                username, 
                email, 
                password_hash, 
                display_name, 
                avatar_url, 
                status, 
                last_seen, 
                is_online as "is_online!", 
                public_key, 
                private_key_encrypted, 
                created_at, 
                updated_at
            FROM users 
            WHERE email = $1
        "#,
        req.email
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("Invalid credentials".to_string()))?;

    // Verify password
    if !verify(req.password, &user.password_hash)? {
        return Err(AppError::BadRequest("Invalid credentials".to_string()));
    }

    // Generate token
    let token = create_token(user.id)?;

    Ok((
        StatusCode::OK,
        Json(TokenResponse {
            token,
            user: user.into(),
        }),
    ))
}

pub async fn logout() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    // Get user
    let user = sqlx::query_as!(
        User,
        r#"
            SELECT 
                id, 
                username, 
                email, 
                password_hash, 
                display_name, 
                avatar_url, 
                status, 
                last_seen, 
                is_online as "is_online!", 
                public_key, 
                private_key_encrypted, 
                created_at, 
                updated_at
            FROM users 
            WHERE id = $1
        "#,
        auth_user.id
    )
    .fetch_one(&state.pool)
    .await?;

    // Generate new token
    let token = create_token(user.id)?;

    Ok((
        StatusCode::OK,
        Json(TokenResponse {
            token,
            user: user.into(),
        }),
    ))
}

