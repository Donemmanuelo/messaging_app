use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use redis::{AsyncCommands, Commands};
use serde::{Deserialize, Serialize};
use std::fs::create_dir_all;
use std::io::Write;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{Contact, User, UserResponse},
    AppState,
};

pub async fn get_user(
    State(_state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let cache_key = format!("user:{}", user_id);

    let redis = &_state.redis;
    let cached: Option<String> = redis.clone().get(&cache_key)?;
    if let Some(cached) = cached {
        if let Ok(user) = serde_json::from_str::<UserResponse>(&cached) {
            return Ok(Json(user));
        }
    }

    let user = sqlx::query_as!(
        User,
        r#"SELECT id, username, email, password_hash, display_name, avatar_url, status, last_seen, is_online, created_at, updated_at, public_key, private_key_encrypted, phone_number, otp_code, otp_expires_at FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_one(&_state.pool)
    .await?;

    let user_response = UserResponse::from(user);

    if let Ok(json) = serde_json::to_string(&user_response) {
        let _: redis::RedisResult<()> = redis.clone().set_ex(&cache_key, &json, 3600); // Cache for 1 hour
    }

    Ok(Json(user_response))
}

pub async fn update_user(
    State(_state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(update): Json<UserResponse>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"UPDATE users SET display_name = $1, avatar_url = $2, status = $3, updated_at = NOW() WHERE id = $4 RETURNING id, username, email, password_hash, display_name, avatar_url, status, last_seen, is_online, created_at, updated_at, public_key, private_key_encrypted, phone_number, otp_code, otp_expires_at"#,
        update.display_name,
        update.avatar_url,
        update.status,
        user_id
    )
    .fetch_one(&_state.pool)
    .await?;

    let user_response = UserResponse::from(user);
    let cache_key = format!("user:{}", user_id);

    let redis = &_state.redis;
    if let Ok(json) = serde_json::to_string(&user_response) {
        let _: redis::RedisResult<()> = redis.clone().set_ex(&cache_key, &json, 3600); // Cache for 1 hour
    }

    Ok(Json(user_response))
}

pub async fn get_users(
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let cache_key = "users:all";

    let redis = &_state.redis;
    let cached: Option<String> = redis.clone().get(&cache_key)?;
    if let Some(cached) = cached {
        if let Ok(users) = serde_json::from_str::<Vec<UserResponse>>(&cached) {
            return Ok(Json(users));
        }
    }

    let users = sqlx::query_as!(
        User,
        r#"SELECT id, username, email, password_hash, display_name, avatar_url, status, last_seen, is_online, created_at, updated_at, public_key, private_key_encrypted, phone_number, otp_code, otp_expires_at FROM users ORDER BY created_at DESC"#,
    )
    .fetch_all(&_state.pool)
    .await?;

    let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

    if let Ok(json) = serde_json::to_string(&user_responses) {
        let _: redis::RedisResult<()> = redis.clone().set_ex(&cache_key, &json, 300); // Cache for 5 minutes
    }

    Ok(Json(user_responses))
}

pub async fn get_contacts(
    State(_state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let cache_key = format!("user:{}:contacts", user_id);

    let redis = &_state.redis;
    let cached: Option<String> = redis.clone().get(&cache_key)?;
    if let Some(cached) = cached {
        if let Ok(contacts) = serde_json::from_str::<Vec<Contact>>(&cached) {
            return Ok(Json(contacts));
        }
    }

    let contacts = sqlx::query_as!(
        Contact,
        r#"SELECT id, user_id, contact_id, created_at FROM contacts WHERE user_id = $1 ORDER BY created_at DESC"#,
        user_id
    )
    .fetch_all(&_state.pool)
    .await?;

    if let Ok(json) = serde_json::to_string(&contacts) {
        let _: redis::RedisResult<()> = redis.clone().set_ex(&cache_key, &json, 300); // Cache for 5 minutes
    }

    Ok(Json(contacts))
}

pub async fn add_contact(
    State(_state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(contact_id): Json<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let contact = sqlx::query_as!(
        Contact,
        r#"INSERT INTO contacts (user_id, contact_id) VALUES ($1, $2) RETURNING id, user_id, contact_id, created_at"#,
        user_id,
        contact_id
    )
    .fetch_one(&_state.pool)
    .await?;

    let cache_key = format!("user:{}:contacts", user_id);
    let redis = &_state.redis;
    let _: redis::RedisResult<()> = redis.clone().del(&cache_key);

    Ok(Json(contact))
}

pub async fn remove_contact(
    State(_state): State<Arc<AppState>>,
    Path((user_id, contact_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        r#"
        DELETE FROM contacts
        WHERE user_id = $1 AND contact_id = $2
        "#,
        user_id,
        contact_id
    )
    .execute(&_state.pool)
    .await?;

    let cache_key = format!("user:{}:contacts", user_id);
    let redis = &_state.redis;
    let _: redis::RedisResult<()> = redis.clone().del(&cache_key);

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct UserWithStatus {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub online: bool,
}

pub async fn get_users_with_status(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let users = sqlx::query!("SELECT id, username, avatar_url FROM users")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::from(e))?;
    let con = &state.redis;
    let mut result = vec![];
    for user in users {
        let online: bool = con.clone()
            .sismember("online_users", user.id.to_string())
            .unwrap_or(false);
        result.push(UserWithStatus {
            id: user.id,
            username: user.username,
            avatar_url: user.avatar_url,
            online,
        });
    }
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct UpdateProfileInput {
    pub username: Option<String>,
    pub email: Option<String>,
}

pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(input): Json<UpdateProfileInput>,
) -> impl IntoResponse {
    if let Some(username) = input.username {
        sqlx::query!(
            "UPDATE users SET username = $1 WHERE id = $2",
            username,
            user_id
        )
        .execute(&state.pool)
        .await
        .unwrap();
    }
    if let Some(email) = input.email {
        sqlx::query!("UPDATE users SET email = $1 WHERE id = $2", email, user_id)
            .execute(&state.pool)
            .await
            .unwrap();
    }
    (StatusCode::OK, "Profile updated")
}

#[derive(Serialize)]
pub struct AvatarUrlResponse {
    pub avatar_url: String,
}

pub async fn upload_user_avatar(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let upload_dir = "user_avatars";
    create_dir_all(upload_dir).unwrap();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = format!("user_{}_{}", user_id, field.file_name().unwrap_or("avatar"));
        let file_path = format!("{}/{}", upload_dir, file_name);
        let data = field.bytes().await.unwrap();
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(&data).unwrap();
        let url = format!("/user_avatars/{}", file_name);
        sqlx::query!(
            "UPDATE users SET avatar_url = $1 WHERE id = $2",
            url,
            user_id
        )
        .execute(&state.pool)
        .await
        .unwrap();
        return (StatusCode::OK, Json(AvatarUrlResponse { avatar_url: url })).into_response();
    }
    (StatusCode::BAD_REQUEST, "No file uploaded").into_response()
}

#[derive(Debug, Deserialize)]
pub struct PublicKeyRequest {
    pub public_key: String,
}

/// Endpoint to upload/update a user's public key for E2EE
pub async fn upload_public_key(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<PublicKeyRequest>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        r#"UPDATE users SET public_key = $1 WHERE id = $2"#,
        req.public_key,
        user_id
    )
    .execute(&state.pool)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Endpoint to fetch a user's public key for E2EE
pub async fn get_public_key(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let rec = sqlx::query!(r#"SELECT public_key FROM users WHERE id = $1"#, user_id)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(rec.public_key))
}
