mod config;
mod models;
mod services;
mod routes;
mod utils;

use actix_web::{web, App, HttpServer};
use config::{app::AppConfig, database::DatabaseConfig};
use services::db::Database;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration
    let app_config = AppConfig::default();
    let db_config = DatabaseConfig::default();

    // Initialize database
    let db = Arc::new(Database::new(&db_config.url).await.unwrap());

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(routes::auth::config)
            .configure(routes::messages::config)
            .configure(routes::websocket::config)
    })
    .bind((app_config.host, app_config.port))?
    .run()
    .await
} 