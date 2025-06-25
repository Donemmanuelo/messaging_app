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
// use axum::Server;
use axum::{
    extract::{
        ws::{Message, WebSocketUpgrade},
        State,
    },
    http::{HeaderValue, Method},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
// use dotenv::dotenv;
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

use db::get_pg_pool;
use routes::auth::auth_routes;
use routes::group::group_routes;
use routes::media::media_routes;
use routes::message::message_routes;
// use routes::message_reactions::message_reactions_routes;
use routes::users::users_routes;

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
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_ansi(true)
        .pretty()
        .init();

    // Load environment variables
    // dotenv().ok();
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

    let state = Arc::new(AppState {
        pool,
        ws_tx,
        redis: redis_client,
    });

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
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

    // Build our application with a route
    let app: Router<Arc<AppState>> = Router::new()
        // .merge(swagger_router()) // Remove OpenAPI/Swagger for now
        // Health check endpoints
        .route("/health", get(health))
        // .route("/ready", get(readiness_check))
        // .route("/live", get(liveness_check))
        // API routes are now only nested
        .nest("/api", message_routes())
        .nest("/api", group_routes())
        .nest("/api", media_routes())
        .nest("/api", users_routes())
        .nest("/auth", auth_routes())
        // .nest("/api", message_reactions_routes())
        // .route("/ws", get(ws_handler))
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::compression::CompressionLayer::new())
        // .layer(security_headers())
        // .layer(axum::middleware::from_fn(validate_request))
        // .layer(axum::middleware::from_fn(add_security_headers))
        .with_state(state.clone());

    // Run it
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    info!("Server running on {}", addr);

    // Handle graceful shutdown
    let shutdown_signal = async {
        let ctrl_c = async {
            signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        info!("Shutting down gracefully...");
    };

    // Instead, use a placeholder or comment for server startup for now
    // let server = axum::Server::bind(&addr.parse()?);
    // graceful_shutdown(shutdown_signal, server.serve(app.into_make_service())).await;

    Ok(())
}

// async fn ws_handler(
//     ws: WebSocketUpgrade,
//     State(state): State<AppState>,
//     auth_user: AuthUser,
// ) -> impl IntoResponse {
//     handle_websocket(ws, state.ws_state, auth_user).await
// }
