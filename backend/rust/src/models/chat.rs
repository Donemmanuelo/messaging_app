use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Chat {
    pub id: String,
    pub name: String,
    pub is_group: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCreate {
    pub name: String,
    pub is_group: bool,
    pub participant_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatUpdate {
    pub name: Option<String>,
    pub participant_ids: Option<Vec<String>>,
} 