use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::AppState;

pub mod auth;
pub mod chat;
pub mod group;
pub mod group_chat;
pub mod groups;
pub mod media;
pub mod message;
pub mod message_actions;
pub mod message_reactions;
pub mod messages;
pub mod users;
pub mod ws;
pub mod status;
pub mod push;
pub mod call_log;
pub mod status_view;

pub use auth::*;
pub use chat::*;
pub use group::*;
pub use group_chat::*;
pub use groups::*;
pub use media::*;
pub use message::*;
pub use message_actions::*;
pub use message_reactions::*;
pub use messages::*;
pub use users::*;
pub use ws::*;

// use crate::{config::Config, database::Database};
