use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Duration, Utc};

use crate::{
    handlers::AppState,
    models::{CreateUserRequest, LoginRequest, AuthResponse, User, UserResponse},
    services::jwt::{Claims, create_jwt_token},
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Check if user already exists
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 OR username = $2"
    )
    .bind(&payload.email)
    .bind(&payload.username)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing_user.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    // Hash password
    let password_hash = hash(payload.password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create user
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, email, password_hash, display_name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.display_name)
    .bind(now)
    .bind(now)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate JWT tokens
    let access_token = create_jwt_token(user.id, &state.config.jwt_secret, Duration::hours(24))?;
    let refresh_token = create_jwt_token(user.id, &state.config.jwt_secret, Duration::days(30))?;

    let response = AuthResponse {
        user: user.into(),
        access_token,
        refresh_token,
    };

    Ok(Json(json!(response)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Find user
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify password
    if !verify(&payload.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Update user online status
    sqlx::query(
        "UPDATE users SET is_online = true, last_seen = $1 WHERE id = $2"
    )
    .bind(Utc::now())
    .bind(user.id)
    .execute(&state.db.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate JWT tokens
    let access_token = create_jwt_token(user.id, &state.config.jwt_secret, Duration::hours(24))?;
    let refresh_token = create_jwt_token(user.id, &state.config.jwt_secret, Duration::days(30))?;

    let response = AuthResponse {
        user: user.into(),
        access_token,
        refresh_token,
    };

    Ok(Json(json!(response)))
}

pub async fn logout(
    State(state): State<AppState>,
    // Add JWT middleware to extract user_id
) -> Result<Json<Value>, StatusCode> {
    // In a real implementation, you'd extract user_id from JWT middleware
    // For now, this is a placeholder
    Ok(Json(json!({"message": "Logged out successfully"})))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // Implementation for refreshing JWT tokens
    Ok(Json(json!({"message": "Token refreshed"})))
}