use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::Claims,
    error::AppError,
    models::{
        AddReactionRequest, MessageReaction, MessageReactionResponse,
        RemoveReactionRequest, SearchMessagesRequest, SearchResult,
    },
};

pub async fn add_reaction(
    State(pool): State<PgPool>,
    claims: Claims,
    Path(message_id): Path<Uuid>,
    Json(req): Json<AddReactionRequest>,
) -> Result<Json<MessageReactionResponse>, AppError> {
    let mut tx = pool.begin().await?;

    // Check if user has access to the message
    let message = sqlx::query!(
        "SELECT chat_id FROM messages WHERE id = $1",
        message_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound("Message not found".into()))?;

    let chat_access = sqlx::query!(
        "SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2",
        message.chat_id,
        claims.user_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    if chat_access.is_none() {
        return Err(AppError::Forbidden("No access to this chat".into()));
    }

    // Add reaction
    sqlx::query!(
        "INSERT INTO message_reactions (message_id, user_id, emoji)
         VALUES ($1, $2, $3)
         ON CONFLICT (message_id, user_id, emoji) DO NOTHING",
        message_id,
        claims.user_id,
        req.emoji
    )
    .execute(&mut *tx)
    .await?;

    // Get reaction count and users
    let reaction = sqlx::query_as!(
        MessageReactionResponse,
        r#"
        WITH reaction_users AS (
            SELECT u.*
            FROM message_reactions mr
            JOIN users u ON u.id = mr.user_id
            WHERE mr.message_id = $1 AND mr.emoji = $2
        )
        SELECT 
            $2 as emoji,
            COUNT(*) as count,
            COALESCE(json_agg(json_build_object(
                'id', u.id,
                'username', u.username,
                'email', u.email,
                'created_at', u.created_at
            )), '[]') as users
        FROM reaction_users u
        "#,
        message_id,
        req.emoji
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(reaction))
}

pub async fn remove_reaction(
    State(pool): State<PgPool>,
    claims: Claims,
    Path(message_id): Path<Uuid>,
    Json(req): Json<RemoveReactionRequest>,
) -> Result<StatusCode, AppError> {
    sqlx::query!(
        "DELETE FROM message_reactions
         WHERE message_id = $1 AND user_id = $2 AND emoji = $3",
        message_id,
        claims.user_id,
        req.emoji
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_reactions(
    State(pool): State<PgPool>,
    claims: Claims,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Vec<MessageReactionResponse>>, AppError> {
    // Check if user has access to the message
    let message = sqlx::query!(
        "SELECT chat_id FROM messages WHERE id = $1",
        message_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound("Message not found".into()))?;

    let chat_access = sqlx::query!(
        "SELECT 1 FROM chat_participants WHERE chat_id = $1 AND user_id = $2",
        message.chat_id,
        claims.user_id
    )
    .fetch_optional(&pool)
    .await?;

    if chat_access.is_none() {
        return Err(AppError::Forbidden("No access to this chat".into()));
    }

    let reactions = sqlx::query_as!(
        MessageReactionResponse,
        r#"
        WITH reaction_users AS (
            SELECT 
                mr.emoji,
                u.*
            FROM message_reactions mr
            JOIN users u ON u.id = mr.user_id
            WHERE mr.message_id = $1
        )
        SELECT 
            emoji,
            COUNT(*) as count,
            COALESCE(json_agg(json_build_object(
                'id', u.id,
                'username', u.username,
                'email', u.email,
                'created_at', u.created_at
            )), '[]') as users
        FROM reaction_users u
        GROUP BY emoji
        "#,
        message_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(reactions))
}

pub async fn search_messages(
    State(pool): State<PgPool>,
    claims: Claims,
    Query(req): Query<SearchMessagesRequest>,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    let mut query = String::from(
        r#"
        WITH message_reactions AS (
            SELECT 
                message_id,
                emoji,
                COUNT(*) as count,
                json_agg(json_build_object(
                    'id', u.id,
                    'username', u.username,
                    'email', u.email,
                    'created_at', u.created_at
                )) as users
            FROM message_reactions mr
            JOIN users u ON u.id = mr.user_id
            GROUP BY message_id, emoji
        ),
        message_reads AS (
            SELECT 
                message_id,
                json_agg(json_build_object(
                    'id', u.id,
                    'username', u.username,
                    'email', u.email,
                    'created_at', u.created_at
                )) as users
            FROM message_reads mr
            JOIN users u ON u.id = mr.user_id
            GROUP BY message_id
        )
        SELECT 
            m.*,
            json_build_object(
                'id', s.id,
                'username', s.username,
                'email', s.email,
                'created_at', s.created_at
            ) as sender,
            json_build_object(
                'id', c.id,
                'name', c.name,
                'is_group', c.is_group,
                'created_at', c.created_at
            ) as chat,
            COALESCE(json_agg(DISTINCT jsonb_build_object(
                'emoji', mr.emoji,
                'count', mr.count,
                'users', mr.users
            )) FILTER (WHERE mr.emoji IS NOT NULL), '[]') as reactions,
            COALESCE(mr2.users, '[]') as read_by
        FROM messages m
        JOIN users s ON s.id = m.sender_id
        JOIN chats c ON c.id = m.chat_id
        LEFT JOIN message_reactions mr ON mr.message_id = m.id
        LEFT JOIN message_reads mr2 ON mr2.message_id = m.id
        WHERE m.deleted_at IS NULL
        "#,
    );

    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn sqlx::types::Type>> = Vec::new();
    let mut param_count = 1;

    // Add search condition
    conditions.push(format!("m.search_vector @@ to_tsquery('english', ${})", param_count));
    params.push(Box::new(req.query));
    param_count += 1;

    // Add chat filter if specified
    if let Some(chat_id) = req.chat_id {
        conditions.push(format!("m.chat_id = ${}", param_count));
        params.push(Box::new(chat_id));
        param_count += 1;
    }

    // Add date range if specified
    if let Some(from_date) = req.from_date {
        conditions.push(format!("m.created_at >= ${}", param_count));
        params.push(Box::new(from_date));
        param_count += 1;
    }
    if let Some(to_date) = req.to_date {
        conditions.push(format!("m.created_at <= ${}", param_count));
        params.push(Box::new(to_date));
        param_count += 1;
    }

    // Add access control
    conditions.push(format!(
        "EXISTS (SELECT 1 FROM chat_participants cp WHERE cp.chat_id = m.chat_id AND cp.user_id = ${})",
        param_count
    ));
    params.push(Box::new(claims.user_id));
    param_count += 1;

    // Add conditions to query
    if !conditions.is_empty() {
        query.push_str(" AND ");
        query.push_str(&conditions.join(" AND "));
    }

    // Add grouping and ordering
    query.push_str(
        r#"
        GROUP BY m.id, s.id, c.id, mr2.users
        ORDER BY m.created_at DESC
        "#,
    );

    // Add pagination
    if let Some(limit) = req.limit {
        query.push_str(&format!(" LIMIT ${}", param_count));
        params.push(Box::new(limit));
        param_count += 1;
    }
    if let Some(offset) = req.offset {
        query.push_str(&format!(" OFFSET ${}", param_count));
        params.push(Box::new(offset));
    }

    let results = sqlx::query_as::<_, SearchResult>(&query)
        .bind_all(params)
        .fetch_all(&pool)
        .await?;

    Ok(Json(results))
} 