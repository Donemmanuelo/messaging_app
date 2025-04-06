mod auth;
mod database;
mod websocket;
mod notifications;
mod security;
mod routes;
mod config;

use routes::auth_routes;
use routes::message_routes;
use routes::user_routes;
use actix_web::{web, App, HttpServer, middleware::Logger};
use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = config::Config::new();
    info!("Starting the server on http://127.0.0.1:8080");

    // Initialize FCM and APNs services
    let fcm_service = notifications::fcm::FcmService::new(&config.fcm_api_key);
    let apns_service = notifications::apns::ApnsService::new(
        &config.apns_cert_path,
        &config.apns_key_path,
        &config.apns_key_id,
        &config.apns_team_id,
    );

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(fcm_service.clone()))
            .app_data(web::Data::new(apns_service.clone()))
            .configure(auth_routes)
            .configure(message_routes)
            .configure(user_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}