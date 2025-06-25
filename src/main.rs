use axum::http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

let database_url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
let redis_url = env::var("REDIS_URL").map_err(|_| "REDIS_URL must be set")?;
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
let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
let addr = format!("0.0.0.0:{}", port);
let server = axum::Server::bind(&addr.parse()?);
let shutdown_signal = async {
    let ctrl_c = async {
        signal::ctrl_c().await.map_err(|_| "Failed to listen for ctrl+c")?;
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .map_err(|_| "Failed to install SIGTERM handler")?
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