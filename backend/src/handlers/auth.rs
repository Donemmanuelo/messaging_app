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
    let existing_user = sqlx::query!(
        "SELECT 1 FROM users WHERE email = $1",
        req.email
    )
    .fetch_optional(&state.pool)
    .await?;

    if existing_user.is_some() {
        return Err(AppError::BadRequest("User already exists".to_string()));
    }

    // Hash password
    let hashed_password = hash(req.password.as_bytes(), DEFAULT_COST)?;

    // Create user
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (email, password, display_name) VALUES ($1, $2, $3) RETURNING *",
        req.email,
        hashed_password,
        req.display_name
    )
    .fetch_one(&state.pool)
    .await?;

    // Generate token
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
        "SELECT * FROM users WHERE email = $1",
        req.email
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("Invalid credentials".to_string()))?;

    // Verify password
    if !verify(req.password, &user.password)? {
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
        "SELECT * FROM users WHERE id = $1",
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