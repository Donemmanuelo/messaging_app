use axum::{
    extract::{ws::WebSocketUpgrade, State, Query},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Router,
};
use axum::middleware::from_fn_with_state;
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

mod config;
mod handlers;
mod middleware;
mod models;
mod services;
mod websocket;

use config::AppConfig;
use middleware::auth::{auth_middleware, ws_auth_middleware};
use services::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = AppConfig::from_env()?;
    let app_state = AppState::new(&config.database_url).await?;

    let app = Router::new()
        .route("/", get(|| async { "WhatsApp Clone API" }))
        .route("/ws", get(websocket_handler))
        .nest("/api", api_routes())
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(app_state));

    let listener = tokio::net::TcpListener::bind(&config.server_address).await?;
    tracing::info!("Server running on {}", config.server_address);

    axum::serve(listener, app).await?;
    Ok(())
}

fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/auth", auth_routes())
        .nest("/users", user_routes().route_layer(from_fn_with_state(Arc::<AppState>::default(), auth_middleware)))
        .nest("/chats", chat_routes().route_layer(from_fn_with_state(Arc::<AppState>::default(), auth_middleware)))
}

fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(handlers::auth::register))
        .route("/login", post(handlers::auth::login))
        .route("/refresh", post(handlers::auth::refresh_token))
}

fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/profile", post(handlers::users::update_profile))
        .route("/search", get(handlers::users::search_users))
}

fn chat_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handlers::chats::get_chats))
        .route("/", post(handlers::chats::create_chat))
        .route("/:chat_id/messages", get(handlers::chats::get_messages))
        .route("/:chat_id/messages", post(handlers::chats::send_message))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, StatusCode> {
    // Extract token from query parameters
    let token = params.get("token").ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Authenticate the token and get the user ID
    let user_id = ws_auth_middleware(token, &state).await?;
    
    // Create a handler that includes the authenticated user_id
    Ok(ws.on_upgrade(move |socket| handle_authenticated_websocket(socket, state, user_id)))
}

// WebSocket handler that includes the authenticated user ID
async fn handle_authenticated_websocket(
    socket: axum::extract::ws::WebSocket,
    state: Arc<AppState>,
    user_id: Uuid,
) {
    // Store the authenticated user ID in the socket context
    tracing::info!("WebSocket connection authenticated for user: {}", user_id);
    
    // Call the internal websocket connection handler with the authenticated user
    websocket::websocket_connection(socket, state).await
}
