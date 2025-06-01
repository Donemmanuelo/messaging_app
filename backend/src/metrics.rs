use lazy_static::lazy_static;
use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, IntCounter, IntGauge, Registry,
    opts,
};
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // Request metrics
    pub static ref HTTP_REQUESTS_TOTAL: IntCounter = IntCounter::new(
        "http_requests_total",
        "Total number of HTTP requests"
    ).unwrap();

    pub static ref HTTP_REQUEST_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds"
        )
    ).unwrap();

    // Database metrics
    pub static ref DB_CONNECTIONS: IntGauge = IntGauge::new(
        "db_connections",
        "Number of active database connections"
    ).unwrap();

    pub static ref DB_QUERY_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "db_query_duration_seconds",
            "Database query duration in seconds"
        )
    ).unwrap();

    // Redis metrics
    pub static ref REDIS_CONNECTIONS: IntGauge = IntGauge::new(
        "redis_connections",
        "Number of active Redis connections"
    ).unwrap();

    pub static ref REDIS_OPERATION_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "redis_operation_duration_seconds",
            "Redis operation duration in seconds"
        )
    ).unwrap();

    // Message metrics
    pub static ref MESSAGES_SENT: IntCounter = IntCounter::new(
        "messages_sent_total",
        "Total number of messages sent"
    ).unwrap();

    pub static ref MESSAGES_DELIVERED: IntCounter = IntCounter::new(
        "messages_delivered_total",
        "Total number of messages delivered"
    ).unwrap();

    // Error metrics
    pub static ref ERROR_COUNTER: IntCounter = IntCounter::new(
        "error_total",
        "Total number of errors"
    ).unwrap();
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(HTTP_REQUEST_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(DB_CONNECTIONS.clone())).unwrap();
    REGISTRY.register(Box::new(DB_QUERY_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(REDIS_CONNECTIONS.clone())).unwrap();
    REGISTRY.register(Box::new(REDIS_OPERATION_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(MESSAGES_SENT.clone())).unwrap();
    REGISTRY.register(Box::new(MESSAGES_DELIVERED.clone())).unwrap();
    REGISTRY.register(Box::new(ERROR_COUNTER.clone())).unwrap();
}

pub async fn metrics_handler() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&REGISTRY.gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
} 