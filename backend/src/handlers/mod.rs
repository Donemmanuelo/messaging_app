pub mod auth;
pub mod chats;
pub mod users;

use crate::{config::Config, database::Database};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis_client: redis::Client,
    pub config: Config,
}