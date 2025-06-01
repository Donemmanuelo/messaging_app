use axum::{
    extract::{
        ws::{Message, WebSocketUpgrade},
        State,
    },
    http::{HeaderValue, Method},
    response::IntoResponse,
    routing::{get, post, put, delete},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use dotenv::dotenv;
use crate::{
    auth::AuthUser,
    handlers::{
        auth::{login, register},
        chat::{
            create_chat, delete_message, get_chat, get_chats, get_messages,
            send_message,
        },
        group_chat::{
            add_member, create_group, get_group, get_groups, get_members,
            remove_member, update_member_role,
        },
        media::{delete_media, upload_media},
        message_actions::{
            forward_message, get_read_receipts, mark_as_read,
        },
        message_reactions::{
            add_reaction, get_reactions, remove_reaction, search_messages,
        },
    },
    websocket::{handle_websocket, WebSocketState},
};
use axum::Server;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling;
use std::env;
use redis::Client;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use crate::middleware::rate_limit::{RateLimiter, rate_limit_middleware};
use crate::middleware::security::{security_headers, validate_request, add_security_headers};
use crate::middleware::circuit_breaker::{CircuitBreaker, graceful_shutdown};
use crate::handlers::health::{health_check, readiness_check, liveness_check};
use tokio::signal;
use std::time::Duration;

mod auth;
mod error;
mod handlers;
mod models;
mod websocket;
mod middleware;
mod config;

#[derive(Clone)]
struct AppState {
    pool: PgPoolOptions,
    ws_state: Arc<WebSocketState>,
    rate_limiter: Arc<RateLimiter>,
    circuit_breaker: Arc<CircuitBreaker>,
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
    dotenv().ok();
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
    let rate_limiter = Arc::new(RateLimiter::new(redis_url));

    // Circuit breaker setup
    let circuit_breaker = Arc::new(CircuitBreaker::new(
        5, // threshold
        Duration::from_secs(30), // reset timeout
    ));

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
            .parse::<HeaderValue>()
            .unwrap())
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_credentials(true);

    // Build our application with a route
    let app = Router::new()
        // Health check endpoints
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/live", get(liveness_check))
        // API routes
        .route("/api/messages", post(handlers::messages::send_message))
        .route("/api/messages/:id", get(handlers::messages::get_message))
        .route("/api/messages/:id", put(handlers::messages::update_message))
        .route("/api/messages/:id", delete(handlers::messages::delete_message))
        .route("/api/messages/group", post(handlers::messages::send_group_message))
        .route("/api/media", post(handlers::media::upload_media))
        .route("/api/media/:id", delete(handlers::media::delete_media))
        // Add state
        .with_state(pool)
        .with_state(rate_limiter)
        .with_state(circuit_breaker)
        // Add middleware
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::compression::CompressionLayer::new())
        .layer(security_headers())
        .layer(axum::middleware::from_fn(validate_request))
        .layer(axum::middleware::from_fn(add_security_headers));

    // Run it
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    info!("Server running on {}", addr);
    
    let server = axum::Server::bind(&addr.parse()?);
    
    // Handle graceful shutdown
    let shutdown_signal = async {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl+c");
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

    graceful_shutdown(shutdown_signal, server.serve(app.into_make_service())).await;

    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    handle_websocket(ws, state.ws_state, auth_user).await
}
