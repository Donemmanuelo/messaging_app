use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CallLog {
    pub id: i32,
    pub caller_id: Uuid,
    pub callee_id: Uuid,
    pub call_type: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: String,
    pub group_id: Option<Uuid>,
} 