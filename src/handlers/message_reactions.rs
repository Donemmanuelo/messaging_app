#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageReactionResponse {
    pub emoji: String,
    pub count: i64,
}

let reaction = sqlx::query_as!(
    MessageReactionResponse,
    r#"
    WITH reaction_users AS (
        SELECT user_id, emoji, created_at FROM message_reactions WHERE message_id = $1 AND emoji = $2
    )
    SELECT emoji, COUNT(*) as count
    FROM reaction_users
    GROUP BY emoji
    "#,
    message_id,
    req.emoji
)
.fetch_one(&pool)
.await?;

let reactions = sqlx::query_as!(
    MessageReactionResponse,
    r#"
    WITH reaction_users AS (
        SELECT user_id, emoji, created_at FROM message_reactions WHERE message_id = $1
    )
    SELECT emoji, COUNT(*) as count
    FROM reaction_users
    GROUP BY emoji
    "#,
    message_id
)
.fetch_all(&pool)
.await?; 