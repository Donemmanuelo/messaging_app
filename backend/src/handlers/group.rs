use crate::models::group::Group;
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fs::create_dir_all;
use std::io::Write;
use uuid;
use uuid::Uuid;
use std::sync::Arc;
use crate::AppState;

#[derive(Deserialize)]
pub struct CreateGroupInput {
    pub name: String,
    pub created_by: uuid::Uuid,
}

#[derive(Serialize)]
pub struct AvatarUrlResponse {
    pub avatar_url: String,
}

pub async fn create_group(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateGroupInput>,
) -> impl IntoResponse {
    match sqlx::query_as!(
        Group,
        "INSERT INTO groups (name, created_by, created_at, description, avatar_url, updated_at, is_private, max_members) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id, name, description, avatar_url, created_by, created_at, updated_at, is_private, max_members",
        input.name,
        input.created_by,
        Utc::now(),
        None::<String>,
        None::<String>,
        Utc::now(),
        None::<bool>,
        None::<i32>
    )
    .fetch_one(&state.pool)
    .await
    {
        Ok(group) => (StatusCode::CREATED, Json(group)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create group: {}", e),
        )
        .into_response(),
    }
}

pub async fn fetch_groups(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match sqlx::query_as!(Group, "SELECT * FROM groups ORDER BY created_at DESC")
        .fetch_all(&state.pool)
        .await
    {
        Ok(groups) => Json(groups).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch groups: {}", e),
        )
            .into_response(),
    }
}


pub async fn upload_group_avatar(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<Uuid>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let upload_dir = "group_avatars";
    create_dir_all(upload_dir).unwrap();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = format!("group_{}_{}", group_id, field.file_name().unwrap_or("avatar"));
        let file_path = format!("{}/{}", upload_dir, file_name);
        let data = field.bytes().await.unwrap();
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(&data).unwrap();
        let url = format!("/group_avatars/{}", file_name);
        sqlx::query!(
            "UPDATE groups SET avatar_url = $1 WHERE id = $2",
            url,
            group_id
        )
        .execute(&state.pool)
        .await
        .unwrap();
        return (StatusCode::OK, Json(AvatarUrlResponse { avatar_url: url })).into_response();
    }
    (StatusCode::BAD_REQUEST, "No file uploaded").into_response()
}
