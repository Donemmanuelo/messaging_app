let dsn = env::var("SENTRY_DSN").map_err(|_| "SENTRY_DSN must be set")?;
let environment = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
let options = ClientOptions {
    dsn: Some(dsn.parse().map_err(|_| "Invalid SENTRY_DSN")?),
    environment: Some(environment.into()),
    release: Some(env!("CARGO_PKG_VERSION").into()),
    traces_sample_rate: 1.0,
    ..Default::default()
}; 