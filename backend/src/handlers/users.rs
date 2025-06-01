use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;
use redis::{AsyncCommands, Commands};

use crate::{
    AppState,
    error::AppError,
    models::{
        Contact,
        UserResponse,
        User,
    },
};

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    let cache_key = format!("user:{}", user_id);
    
    if let Ok(Some(cached)) = state.redis.get::<_, Option<String>>(&cache_key) {
        if let Ok(user) = serde_json::from_str::<UserResponse>(&cached) {
            return Ok(Json(user));
        }
    }

    let user = sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&state.pool)
    .await?;

    let user_response = UserResponse::from(user);
    
    if let Ok(json) = serde_json::to_string(&user_response) {
        let _ = state.redis.set_ex(&cache_key, &json, 3600); // Cache for 1 hour
    }

    Ok(Json(user_response))
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(update): Json<UserResponse>,
) -> Result<Json<UserResponse>, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET display_name = $1, avatar_url = $2, status = $3, updated_at = NOW()
        WHERE id = $4
        RETURNING *
        "#,
        update.display_name,
        update.avatar_url,
        update.status,
        user_id
    )
    .fetch_one(&state.pool)
    .await?;

    let user_response = UserResponse::from(user);
    let cache_key = format!("user:{}", user_id);
    
    if let Ok(json) = serde_json::to_string(&user_response) {
        let _ = state.redis.set_ex(&cache_key, &json, 3600); // Cache for 1 hour
    }

    Ok(Json(user_response))
}

pub async fn get_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let cache_key = "users:all";
    
    if let Ok(Some(cached)) = state.redis.get::<_, Option<String>>(&cache_key) {
        if let Ok(users) = serde_json::from_str::<Vec<UserResponse>>(&cached) {
            return Ok(Json(users));
        }
    }

    let users = sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    
    if let Ok(json) = serde_json::to_string(&user_responses) {
        let _ = state.redis.set_ex(&cache_key, &json, 300); // Cache for 5 minutes
    }

    Ok(Json(user_responses))
}

pub async fn get_contacts(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<Contact>>, AppError> {
    let cache_key = format!("user:{}:contacts", user_id);
    
    if let Ok(Some(cached)) = state.redis.get::<_, Option<String>>(&cache_key) {
        if let Ok(contacts) = serde_json::from_str::<Vec<Contact>>(&cached) {
            return Ok(Json(contacts));
        }
    }

    let contacts = sqlx::query_as!(
        Contact,
        r#"
        SELECT * FROM contacts
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.pool)
    .await?;
    
    if let Ok(json) = serde_json::to_string(&contacts) {
        let _ = state.redis.set_ex(&cache_key, &json, 300); // Cache for 5 minutes
    }

    Ok(Json(contacts))
}

pub async fn add_contact(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(contact_id): Json<Uuid>,
) -> Result<Json<Contact>, AppError> {
    let contact = sqlx::query_as!(
        Contact,
        r#"
        INSERT INTO contacts (user_id, contact_id)
        VALUES ($1, $2)
        RETURNING *
        "#,
        user_id,
        contact_id
    )
    .fetch_one(&state.pool)
    .await?;

    let cache_key = format!("user:{}:contacts", user_id);
    let _ = state.redis.del(&cache_key);

    Ok(Json(contact))
}

pub async fn remove_contact(
    State(state): State<Arc<AppState>>,
    Path((user_id, contact_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    sqlx::query!(
        r#"
        DELETE FROM contacts
        WHERE user_id = $1 AND contact_id = $2
        "#,
        user_id,
        contact_id
    )
    .execute(&state.pool)
    .await?;

    let cache_key = format!("user:{}:contacts", user_id);
    let _ = state.redis.del(&cache_key);

    Ok(StatusCode::NO_CONTENT)
}