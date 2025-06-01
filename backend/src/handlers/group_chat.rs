use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    models::{CreateGroupRequest, UpdateGroupRequest, AddGroupMembersRequest, RemoveGroupMembersRequest, UpdateGroupRoleRequest, GroupChat, GroupMember},
    auth::Claims,
    auth::AuthUser,
    error::AppError,
};

pub async fn get_groups(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let groups = sqlx::query_as!(
        Chat,
        r#"
        SELECT c.*, 
            (SELECT json_agg(json_build_object(
                'id', u.id,
                'username', u.username,
                'email', u.email
            ))
            FROM users u
            JOIN chat_participants cp ON cp.user_id = u.id
            WHERE cp.chat_id = c.id) as participants
        FROM chats c
        JOIN chat_participants cp ON cp.chat_id = c.id
        WHERE cp.user_id = $1 AND c.is_group = true
        ORDER BY c.updated_at DESC
        "#,
        auth_user.id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(groups))
}

pub async fn get_group(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(group_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let group = sqlx::query_as!(
        Chat,
        r#"
        SELECT c.*, 
            (SELECT json_agg(json_build_object(
                'id', u.id,
                'username', u.username,
                'email', u.email
            ))
            FROM users u
            JOIN chat_participants cp ON cp.user_id = u.id
            WHERE cp.chat_id = c.id) as participants
        FROM chats c
        WHERE c.id = $1 AND c.is_group = true
        "#,
        group_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Group not found".into()))?;

    // Check if user is a participant
    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2)",
        group_id,
        auth_user.id
    )
    .fetch_one(&pool)
    .await?
    .exists;

    if !is_participant {
        return Err(AppError::Forbidden("Not a participant in this group".into()));
    }

    Ok(Json(group))
}

pub async fn create_group(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Json(req): Json<CreateGroupRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Start transaction
    let mut tx = pool.begin().await?;

    // Create group
    let group = sqlx::query_as!(
        Chat,
        r#"
        INSERT INTO chats (name, is_group)
        VALUES ($1, true)
        RETURNING *
        "#,
        req.name
    )
    .fetch_one(&mut tx)
    .await?;

    // Add participants
    for user_id in req.participant_ids {
        sqlx::query!(
            "INSERT INTO chat_participants (chat_id, user_id) VALUES ($1, $2)",
            group.id,
            user_id
        )
        .execute(&mut tx)
        .await?;
    }

    // Add creator as participant
    sqlx::query!(
        "INSERT INTO chat_participants (chat_id, user_id) VALUES ($1, $2)",
        group.id,
        auth_user.id
    )
    .execute(&mut tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(group)))
}

pub async fn get_members(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(group_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    // Check if user is a participant
    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2)",
        group_id,
        auth_user.id
    )
    .fetch_one(&pool)
    .await?
    .exists;

    if !is_participant {
        return Err(AppError::Forbidden("Not a participant in this group".into()));
    }

    // Get members
    let members = sqlx::query!(
        r#"
        SELECT 
            u.id,
            u.username,
            u.email,
            cp.created_at as joined_at
        FROM users u
        JOIN chat_participants cp ON cp.user_id = u.id
        WHERE cp.chat_id = $1
        ORDER BY cp.created_at ASC
        "#,
        group_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(members))
}

pub async fn add_member(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(group_id): Path<i32>,
    Json(req): Json<AddMemberRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check if user is a participant
    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2)",
        group_id,
        auth_user.id
    )
    .fetch_one(&pool)
    .await?
    .exists;

    if !is_participant {
        return Err(AppError::Forbidden("Not a participant in this group".into()));
    }

    // Add member
    sqlx::query!(
        "INSERT INTO chat_participants (chat_id, user_id) VALUES ($1, $2)",
        group_id,
        req.user_id
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_member(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path((group_id, user_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, AppError> {
    // Check if user is a participant
    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2)",
        group_id,
        auth_user.id
    )
    .fetch_one(&pool)
    .await?
    .exists;

    if !is_participant {
        return Err(AppError::Forbidden("Not a participant in this group".into()));
    }

    // Remove member
    sqlx::query!(
        "DELETE FROM chat_participants WHERE chat_id = $1 AND user_id = $2",
        group_id,
        user_id
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_member_role(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path((group_id, user_id)): Path<(i32, i32)>,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check if user is a participant
    let is_participant = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2)",
        group_id,
        auth_user.id
    )
    .fetch_one(&pool)
    .await?
    .exists;

    if !is_participant {
        return Err(AppError::Forbidden("Not a participant in this group".into()));
    }

    // Update member role
    sqlx::query!(
        "UPDATE chat_participants SET role = $1 WHERE chat_id = $2 AND user_id = $3",
        req.role as _,
        group_id,
        user_id
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    user_id: i32,
}

pub async fn update_group(
    State(pool): State<PgPool>,
    claims: Claims,
    Path(group_id): Path<i32>,
    Json(req): Json<UpdateGroupRequest>,
) -> Result<Json<GroupChat>, StatusCode> {
    // Check if user is admin
    let is_admin = sqlx::query!(
        r#"
        SELECT role FROM chat_participants
        WHERE chat_id = $1 AND user_id = $2 AND role = 'admin'
        "#,
        group_id,
        claims.sub
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .is_some();

    if !is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let group = sqlx::query_as!(
        GroupChat,
        r#"
        UPDATE chats
        SET name = COALESCE($1, name),
            description = COALESCE($2, description),
            avatar_url = COALESCE($3, avatar_url),
            updated_at = NOW()
        WHERE id = $4 AND chat_type = 'group'
        RETURNING id, name, description, avatar_url, created_by as "created_by!", created_at, updated_at
        "#,
        req.name,
        req.description,
        req.avatar_url,
        group_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(group))
}

pub async fn add_members(
    State(pool): State<PgPool>,
    claims: Claims,
    Path(group_id): Path<i32>,
    Json(req): Json<AddGroupMembersRequest>,
) -> Result<StatusCode, StatusCode> {
    // Check if user is admin
    let is_admin = sqlx::query!(
        r#"
        SELECT role FROM chat_participants
        WHERE chat_id = $1 AND user_id = $2 AND role = 'admin'
        "#,
        group_id,
        claims.sub
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .is_some();

    if !is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    for member_id in req.member_ids {
        sqlx::query!(
            r#"
            INSERT INTO chat_participants (id, chat_id, user_id, role, joined_at)
            VALUES ($1, $2, $3, 'member', NOW())
            ON CONFLICT (chat_id, user_id) DO NOTHING
            "#,
            Uuid::new_v4(),
            group_id,
            member_id
        )
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(StatusCode::OK)
}

pub async fn remove_members(
    State(pool): State<PgPool>,
    claims: Claims,
    Path(group_id): Path<i32>,
    Json(req): Json<RemoveGroupMembersRequest>,
) -> Result<StatusCode, StatusCode> {
    // Check if user is admin
    let is_admin = sqlx::query!(
        r#"
        SELECT role FROM chat_participants
        WHERE chat_id = $1 AND user_id = $2 AND role = 'admin'
        "#,
        group_id,
        claims.sub
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .is_some();

    if !is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    for member_id in req.member_ids {
        sqlx::query!(
            r#"
            DELETE FROM chat_participants
            WHERE chat_id = $1 AND user_id = $2
            "#,
            group_id,
            member_id
        )
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(StatusCode::OK)
}

pub async fn update_member_role(
    State(pool): State<PgPool>,
    claims: Claims,
    Path(group_id): Path<i32>,
    Json(req): Json<UpdateGroupRoleRequest>,
) -> Result<StatusCode, StatusCode> {
    // Check if user is admin
    let is_admin = sqlx::query!(
        r#"
        SELECT role FROM chat_participants
        WHERE chat_id = $1 AND user_id = $2 AND role = 'admin'
        "#,
        group_id,
        claims.sub
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .is_some();

    if !is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    sqlx::query!(
        r#"
        UPDATE chat_participants
        SET role = $1
        WHERE chat_id = $2 AND user_id = $3
        "#,
        req.role,
        group_id,
        req.user_id
    )
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn get_group_members(
    State(pool): State<PgPool>,
    claims: Claims,
    Path(group_id): Path<i32>,
) -> Result<Json<Vec<GroupMember>>, StatusCode> {
    // Check if user is member
    let is_member = sqlx::query!(
        r#"
        SELECT 1 FROM chat_participants
        WHERE chat_id = $1 AND user_id = $2
        "#,
        group_id,
        claims.sub
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .is_some();

    if !is_member {
        return Err(StatusCode::FORBIDDEN);
    }

    let members = sqlx::query_as!(
        GroupMember,
        r#"
        SELECT id, group_id as "group_id!", user_id as "user_id!", role, joined_at
        FROM chat_participants
        WHERE chat_id = $1
        "#,
        group_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(members))
} 