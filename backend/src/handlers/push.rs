use axum::{extract::{State, Json}, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::{AppState, middleware::AuthUser, error::AppError};

#[derive(Deserialize, Serialize)]
pub struct PushSubscription {
    pub endpoint: String,
    pub keys: serde_json::Value,
}

// POST /api/push/subscribe
pub async fn subscribe_push(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(sub): Json<PushSubscription>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "INSERT INTO push_subscriptions (user_id, endpoint, keys) VALUES ($1, $2, $3) ON CONFLICT (user_id, endpoint) DO UPDATE SET keys = $3",
        auth_user.id, sub.endpoint, sub.keys
    ).execute(&state.pool).await?;
    Ok(StatusCode::NO_CONTENT)
} 