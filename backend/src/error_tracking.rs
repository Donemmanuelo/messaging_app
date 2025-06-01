use sentry::{ClientOptions, init};
use std::env;

pub fn init_error_tracking() {
    let dsn = env::var("SENTRY_DSN").expect("SENTRY_DSN must be set");
    let environment = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    
    let options = ClientOptions {
        dsn: Some(dsn.parse().unwrap()),
        environment: Some(environment.into()),
        release: Some(env!("CARGO_PKG_VERSION").into()),
        traces_sample_rate: 1.0,
        ..Default::default()
    };

    let _guard = init(options);
    log::info!("Sentry error tracking initialized");
}

pub fn capture_error(error: &anyhow::Error) {
    sentry::capture_error(error);
}

pub fn capture_message(message: &str, level: sentry::Level) {
    sentry::capture_message(message, level);
} 