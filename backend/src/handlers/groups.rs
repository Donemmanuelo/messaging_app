use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    AppState,
    error::AppError,
    models::group::{
        Group, GroupMember, GroupResponse, GroupMemberResponse,
        CreateGroupRequest, UpdateGroupRequest,
    },
    auth::Claims,
};
use std::sync::Arc;

pub async fn create_group(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(req): Json<CreateGroupRequest>,
) -> Result<Json<GroupResponse>, AppError> {
    let group_id = Uuid::new_v4();
    
    // Start a transaction
    let mut tx = state.pool.begin().await?;

    // Create the group
    let group = sqlx::query_as!(
        Group,
        r#"
        INSERT INTO groups (id, name, description, avatar_url, created_by, created_at, updated_at, is_private, max_members)
        VALUES ($1, $2, $3, $4, $5, NOW(), NOW(), $6, $7)
        RETURNING *
        "#,
        group_id,
        req.name,
        req.description,
        req.avatar_url,
        claims.sub,
        req.is_private,
        req.max_members.unwrap_or(100)
    )
    .fetch_one(&mut *tx)
    .await?;

    // Add the creator as owner
    sqlx::query!(
        r#"
        INSERT INTO group_members (id, group_id, user_id, role, joined_at)
        VALUES ($1, $2, $3, 'owner', NOW())
        "#,
        Uuid::new_v4(),
        group_id,
        claims.sub
    )
    .execute(&mut *tx)
    .await?;

    // Add initial members
    for member_id in req.initial_members {
        if member_id != claims.sub {
            sqlx::query!(
                r#"
                INSERT INTO group_members (id, group_id, user_id, role, joined_at)
                VALUES ($1, $2, $3, 'member', NOW())
                "#,
                Uuid::new_v4(),
                group_id,
                member_id
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    // Commit the transaction
    tx.commit().await?;

    // Get member count
    let member_count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM group_members
        WHERE group_id = $1
        "#,
        group_id
    )
    .fetch_one(&state.pool)
    .await?
    .count;

    Ok(Json(GroupResponse {
        id: group.id,
        name: group.name,
        description: group.description,
        avatar_url: group.avatar_url,
        created_by: group.created_by,
        created_at: group.created_at,
        updated_at: group.updated_at,
        is_private: group.is_private,
        max_members: group.max_members,
        member_count,
        role: crate::models::group::GroupRole::Owner,
    }))
}

pub async fn get_group(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(group_id): Path<Uuid>,
) -> Result<Json<GroupResponse>, AppError> {
    // Check if user is a member
    let member = sqlx::query_as!(
        GroupMember,
        r#"
        SELECT * FROM group_members
        WHERE group_id = $1 AND user_id = $2
        "#,
        group_id,
        claims.sub
    )
    .fetch_optional(&state.pool)
    .await?;

    if member.is_none() {
        return Err(AppError::NotFound("Group not found or access denied".into()));
    }

    let group = sqlx::query_as!(
        Group,
        r#"
        SELECT * FROM groups
        WHERE id = $1
        "#,
        group_id
    )
    .fetch_one(&state.pool)
    .await?;

    let member_count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM group_members
        WHERE group_id = $1
        "#,
        group_id
    )
    .fetch_one(&state.pool)
    .await?
    .count;

    Ok(Json(GroupResponse {
        id: group.id,
        name: group.name,
        description: group.description,
        avatar_url: group.avatar_url,
        created_by: group.created_by,
        created_at: group.created_at,
        updated_at: group.updated_at,
        is_private: group.is_private,
        max_members: group.max_members,
        member_count,
        role: member.unwrap().role,
    }))
}

pub async fn update_group(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(group_id): Path<Uuid>,
    Json(req): Json<UpdateGroupRequest>,
) -> Result<Json<GroupResponse>, AppError> {
    // Check if user is an admin or owner
    let member = sqlx::query_as!(
        GroupMember,
        r#"
        SELECT * FROM group_members
        WHERE group_id = $1 AND user_id = $2 AND role IN ('owner', 'admin')
        "#,
        group_id,
        claims.sub
    )
    .fetch_optional(&state.pool)
    .await?;

    if member.is_none() {
        return Err(AppError::Forbidden("Only admins can update group settings".into()));
    }

    // Update the group
    let group = sqlx::query_as!(
        Group,
        r#"
        UPDATE groups
        SET 
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            avatar_url = COALESCE($3, avatar_url),
            is_private = COALESCE($4, is_private),
            max_members = COALESCE($5, max_members),
            updated_at = NOW()
        WHERE id = $6
        RETURNING *
        "#,
        req.name,
        req.description,
        req.avatar_url,
        req.is_private,
        req.max_members,
        group_id
    )
    .fetch_one(&state.pool)
    .await?;

    let member_count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM group_members
        WHERE group_id = $1
        "#,
        group_id
    )
    .fetch_one(&state.pool)
    .await?
    .count;

    Ok(Json(GroupResponse {
        id: group.id,
        name: group.name,
        description: group.description,
        avatar_url: group.avatar_url,
        created_by: group.created_by,
        created_at: group.created_at,
        updated_at: group.updated_at,
        is_private: group.is_private,
        max_members: group.max_members,
        member_count,
        role: member.unwrap().role,
    }))
}

pub async fn get_group_members(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Vec<GroupMemberResponse>>, AppError> {
    // Check if user is a member
    let is_member = sqlx::query!(
        r#"
        SELECT 1 FROM group_members
        WHERE group_id = $1 AND user_id = $2
        "#,
        group_id,
        claims.sub
    )
    .fetch_optional(&state.pool)
    .await?
    .is_some();

    if !is_member {
        return Err(AppError::NotFound("Group not found or access denied".into()));
    }

    let members = sqlx::query!(
        r#"
        SELECT 
            gm.id,
            gm.user_id,
            u.display_name,
            u.avatar_url,
            gm.role,
            gm.joined_at,
            gm.last_read_at
        FROM group_members gm
        JOIN users u ON u.id = gm.user_id
        WHERE gm.group_id = $1
        ORDER BY 
            CASE gm.role
                WHEN 'owner' THEN 1
                WHEN 'admin' THEN 2
                ELSE 3
            END,
            gm.joined_at
        "#,
        group_id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(members.into_iter().map(|m| GroupMemberResponse {
        id: m.id,
        user_id: m.user_id,
        display_name: m.display_name,
        avatar_url: m.avatar_url,
        role: m.role.into(),
        joined_at: m.joined_at,
        last_read_at: m.last_read_at,
    }).collect()))
}

pub async fn add_group_member(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path((group_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<GroupMemberResponse>, AppError> {
    // Check if user is an admin or owner
    let member = sqlx::query_as!(
        GroupMember,
        r#"
        SELECT * FROM group_members
        WHERE group_id = $1 AND user_id = $2 AND role IN ('owner', 'admin')
        "#,
        group_id,
        claims.sub
    )
    .fetch_optional(&state.pool)
    .await?;

    if member.is_none() {
        return Err(AppError::Forbidden("Only admins can add members".into()));
    }

    // Check if group is full
    let member_count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM group_members
        WHERE group_id = $1
        "#,
        group_id
    )
    .fetch_one(&state.pool)
    .await?
    .count;

    let max_members = sqlx::query!(
        r#"
        SELECT max_members FROM groups
        WHERE id = $1
        "#,
        group_id
    )
    .fetch_one(&state.pool)
    .await?
    .max_members;

    if member_count >= max_members as i64 {
        return Err(AppError::BadRequest("Group is full".into()));
    }

    // Add the member
    let new_member = sqlx::query_as!(
        GroupMember,
        r#"
        INSERT INTO group_members (id, group_id, user_id, role, joined_at)
        VALUES ($1, $2, $3, 'member', NOW())
        RETURNING *
        "#,
        Uuid::new_v4(),
        group_id,
        user_id
    )
    .fetch_one(&state.pool)
    .await?;

    // Get user info
    let user = sqlx::query!(
        r#"
        SELECT display_name, avatar_url
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(GroupMemberResponse {
        id: new_member.id,
        user_id: new_member.user_id,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        role: new_member.role,
        joined_at: new_member.joined_at,
        last_read_at: new_member.last_read_at,
    }))
}

pub async fn remove_group_member(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path((group_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    // Check if user is an admin or owner
    let member = sqlx::query_as!(
        GroupMember,
        r#"
        SELECT * FROM group_members
        WHERE group_id = $1 AND user_id = $2 AND role IN ('owner', 'admin')
        "#,
        group_id,
        claims.sub
    )
    .fetch_optional(&state.pool)
    .await?;

    if member.is_none() {
        return Err(AppError::Forbidden("Only admins can remove members".into()));
    }

    // Check if trying to remove the owner
    let target_member = sqlx::query_as!(
        GroupMember,
        r#"
        SELECT * FROM group_members
        WHERE group_id = $1 AND user_id = $2
        "#,
        group_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?;

    if let Some(target) = target_member {
        if target.role == crate::models::group::GroupRole::Owner {
            return Err(AppError::Forbidden("Cannot remove group owner".into()));
        }
    }

    // Remove the member
    sqlx::query!(
        r#"
        DELETE FROM group_members
        WHERE group_id = $1 AND user_id = $2
        "#,
        group_id,
        user_id
    )
    .execute(&state.pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
} 