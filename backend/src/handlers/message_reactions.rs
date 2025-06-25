use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use uuid::Uuid;
use std::sync::Arc;

use crate::{
    error::AppError,
    models::{
        AddReactionRequest, RemoveReactionRequest,
        SearchMessagesRequest, SearchResult,
    },
    AppState,
    middleware::AuthUser,
};
use axum::response::IntoResponse;
use serde_json::Value;

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct MessageReactionResponse {
    pub emoji: String,
    pub count: i64,
    pub users: Value, // Accept raw JSON value
}

pub async fn add_reaction(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(message_id): Path<Uuid>,
    Json(req): Json<AddReactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut tx = state.pool.begin().await?;

    // Check if user has access to the message
    let message = sqlx::query!("SELECT chat_id FROM messages WHERE id = $1", message_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("Message not found".into()))?;

    let chat_access = sqlx::query!(
        "SELECT 1 as exists FROM chat_participants WHERE chat_id = $1 AND user_id = $2",
        message.chat_id,
        auth_user.id
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
        auth_user.id,
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
            $2::text as "emoji!",
            COUNT(*) as "count!",
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
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(message_id): Path<Uuid>,
    Json(req): Json<RemoveReactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "DELETE FROM message_reactions
         WHERE message_id = $1 AND user_id = $2 AND emoji = $3",
        message_id,
        auth_user.id,
        req.emoji
    )
    .execute(&state.pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_reactions(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(message_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Check if user has access to the message
    let message = sqlx::query!("SELECT chat_id FROM messages WHERE id = $1", message_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound("Message not found".into()))?;

    let chat_access = sqlx::query!(
        "SELECT 1 as exists FROM chat_participants WHERE chat_id = $1 AND user_id = $2",
        message.chat_id,
        auth_user.id
    )
    .fetch_optional(&state.pool)
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
            COUNT(*) as "count!",
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
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(reactions))
}

pub async fn search_messages(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Query(req): Query<SearchMessagesRequest>,
) -> Result<impl IntoResponse, AppError> {
    let  query = String::from(
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
        
    "#);
    // Add any additional WHERE clauses and parameters as needed
    let mut query_builder = sqlx::query_as::<_, SearchResult>(&query);
    query_builder = query_builder.bind(&req.query);
    if let Some(chat_id) = req.chat_id {
        query_builder = query_builder.bind(chat_id);
    }
    query_builder = query_builder.bind(&auth_user.id);
    if let Some(limit) = req.limit {
        query_builder = query_builder.bind(limit);
    }
    if let Some(offset) = req.offset {
        query_builder = query_builder.bind(offset);
    }
    let results = query_builder.fetch_all(&state.pool).await?;
    Ok(Json(results))
}
