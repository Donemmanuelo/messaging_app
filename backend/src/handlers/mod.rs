use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::AppState;

pub mod auth;
pub mod users;
pub mod media;
pub mod groups;
pub mod messages;
pub mod chat;
pub mod group_chat;
pub mod message_actions;
pub mod message_reactions;
pub mod ws;

pub use auth::*;
pub use users::*;
pub use media::*;
pub use groups::*;
pub use messages::*;
pub use chat::*;
pub use group_chat::*;
pub use message_actions::*;
pub use message_reactions::*;
pub use ws::*;

// use crate::{config::Config, database::Database};