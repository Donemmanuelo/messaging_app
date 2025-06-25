// Comment out or remove the unresolved import of message_reads from handlers
// use crate::handlers::message_reads::{get_reads, mark_read};
use axum::{
    routing::{get, post},
    Router,
};

pub fn message_reads_routes() -> Router {
    Router::new()
        // .route("/api/message_reads", post(mark_read))
        // .route("/api/message_reads/:message_id", get(get_reads))
}
