use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct StatusView {
    pub id: i32,
    pub status_id: Uuid,
    pub user_id: Uuid,
    pub viewed_at: NaiveDateTime,
} 