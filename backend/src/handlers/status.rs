use axum::{extract::{State, Path}, Json, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{Utc, Duration};
use uuid::Uuid;
use crate::{AppState, middleware::AuthUser, error::AppError};

#[derive(Deserialize)]
pub struct StatusInput {
    pub media_url: Option<String>,
    pub text: Option<String>,
}

#[derive(Serialize)]
pub struct Status {
    pub id: Uuid,
    pub user_id: Uuid,
    pub media_url: Option<String>,
    pub text: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub expires_at: chrono::DateTime<Utc>,
}

// POST /api/status
pub async fn post_status(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Json(input): Json<StatusInput>,
) -> Result<impl IntoResponse, AppError> {
    let expires_at = Utc::now() + Duration::hours(24);
    let rec = sqlx::query!(
        "INSERT INTO statuses (user_id, media_url, text, created_at, expires_at) VALUES ($1, $2, $3, NOW(), $4) RETURNING id, user_id, media_url, text, created_at, expires_at",
        auth_user.id, input.media_url, input.text, expires_at.naive_utc()
    ).fetch_one(&state.pool).await?;
    let status = Status {
        id: rec.id,
        user_id: rec.user_id.expect("user_id should not be null"),
        media_url: rec.media_url,
        text: rec.text,
        created_at: chrono::DateTime::from_utc(rec.created_at.expect("created_at should not be null"), Utc),
        expires_at: chrono::DateTime::from_utc(rec.expires_at.expect("expires_at should not be null"), Utc),
    };
    Ok((StatusCode::CREATED, Json(status)))
}

// GET /api/status
pub async fn get_statuses(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();
    let recs = sqlx::query!(
        "SELECT id, user_id, media_url, text, created_at, expires_at FROM statuses WHERE expires_at > $1",
        now.naive_utc()
    ).fetch_all(&state.pool).await?;
    let statuses: Vec<Status> = recs.into_iter().map(|rec| Status {
        id: rec.id,
        user_id: rec.user_id.expect("user_id should not be null"),
        media_url: rec.media_url,
        text: rec.text,
        created_at: chrono::DateTime::from_utc(rec.created_at.expect("created_at should not be null"), Utc),
        expires_at: chrono::DateTime::from_utc(rec.expires_at.expect("expires_at should not be null"), Utc),
    }).collect();
    Ok(Json(statuses))
}

// DELETE /api/status/:id
pub async fn delete_status(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let res = sqlx::query!(
        "DELETE FROM statuses WHERE id = $1 AND user_id = $2",
        id, auth_user.id
    ).execute(&state.pool).await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Status not found or not owned by user".into()));
    }
    Ok(StatusCode::NO_CONTENT)
} 