use serde::Deserialize;
use std::env;
use log::{info, error};

#[derive(Deserialize)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub fcm_api_key: String,
    pub apns_cert_path: String,
    pub apns_key_path: String,
    pub apns_key_id: String,
    pub apns_team_id: String,
}

impl Config {
    pub fn new() -> Self {
        info!("Loading configuration");
        dotenv::dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let fcm_api_key = env::var("FCM_API_KEY").expect("FCM_API_KEY must be set");
        let apns_cert_path = env::var("APNS_CERT_PATH").expect("APNS_CERT_PATH must be set");
        let apns_key_path = env::var("APNS_KEY_PATH").expect("APNS_KEY_PATH must be set");
        let apns_key_id = env::var("APNS_KEY_ID").expect("APNS_KEY_ID must be set");
        let apns_team_id = env::var("APNS_TEAM_ID").expect("APNS_TEAM_ID must be set");

        info!("Configuration loaded successfully");
        Config {
            database_url,
            jwt_secret,
            fcm_api_key,
            apns_cert_path,
            apns_key_path,
            apns_key_id,
            apns_team_id,
        }
    }
}