// use crate::handlers::health::{health_check, liveness_check, readiness_check};
// use crate::middleware::circuit_breaker::{graceful_shutdown, CircuitBreaker};
// use crate::middleware::rate_limit::{rate_limit_middleware, RateLimiter};
// use crate::middleware::security::{add_security_headers, security_headers, validate_request};
use crate::{
    auth::AuthUser,
    handlers::{
        auth::{login, register},
        chat::{create_chat, delete_message, get_chat, get_chats, get_messages, send_message},
        group_chat::{
            add_member, create_group, get_group, get_groups, get_members, remove_member,
            // update_member_role,
        },
        media::{delete_media, upload_media},
        message_actions::{forward_message, get_read_receipts, mark_as_read},
        message_reactions::{add_reaction, get_reactions, remove_reaction, search_messages},
        // users::users_routes,
    },
    // websocket::{handle_websocket, WebSocketState},
};
use axum::{
    extract::{
        ws::{Message, WebSocketUpgrade},
        State,
        connect_info::IntoMakeServiceWithConnectInfo,
    },
    http::{HeaderValue, Method},
    response::{IntoResponse, Html},
    routing::{delete, get, post, put},
    Router,
};
use redis::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, Level};
use tracing_appender::rolling;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::sync::broadcast;
use messaging_app_backend::create_app;

mod api_docs;
mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod websocket;
mod services;

use db::get_pg_pool;
use routes::auth::auth_routes;
use routes::group::group_routes;
use routes::media::media_routes;
use routes::status::status_routes;
// use routes::message_reactions::message_reactions_routes;
use routes::users::users_routes;
use routes::push::push_routes;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    ws_tx: broadcast::Sender<String>,
    // rate_limiter: Arc<RateLimiter>,
    // circuit_breaker: Arc<CircuitBreaker>,
    redis: redis::Client,
}

async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .init();

    // Load environment variables
    info!("Starting messaging app backend...");

    // Database setup
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Redis setup
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = Client::open(redis_url)?;
    // let rate_limiter = Arc::new(RateLimiter::new(redis_url));

    // WebSocket broadcast channel setup
    let (ws_tx, _) = broadcast::channel(100);
    let pop = pool.clone();
    let red_cli = redis_client.clone();
    let state = Arc::new(AppState {
        pool,
        ws_tx,
        redis: redis_client,
    });

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true);

    // Build the app using modular routers
    let app = Router::new()
        .route("/", get(|| async { Html("<h1>Messaging App Backend is running</h1>") }))
        .nest("/api/auth", auth_routes())
        .nest("/api/groups", group_routes())
        .nest("/api/media", media_routes())
        .nest("/api/status", status_routes())
        .nest("/api/users", users_routes())
        .nest("/api/push", push_routes())
        // Add more .nest() for other modular routers as needed
        .layer(cors)
        .with_state(state.clone());

    // Axum server startup
    let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    info!("Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// async fn ws_handler(
//     ws: WebSocketUpgrade,
//     State(state): State<AppState>,
//     auth_user: AuthUser,
// ) -> impl IntoResponse {
//     handle_websocket(ws, state.ws_state, auth_user).await
// }
