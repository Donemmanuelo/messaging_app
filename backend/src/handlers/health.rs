use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::PgPool;
use redis::Client;
use std::sync::Arc;
use crate::AppState;

#[derive(Debug, serde::Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    database: DatabaseStatus,
    redis: RedisStatus,
    uptime: u64,
}

#[derive(Debug, serde::Serialize)]
struct DatabaseStatus {
    status: String,
    latency_ms: u64,
}

#[derive(Debug, serde::Serialize)]
struct RedisStatus {
    status: String,
    latency_ms: u64,
}

pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let start_time = std::time::Instant::now();
    
    // Check database health
    let db_start = std::time::Instant::now();
    let db_status = match sqlx::query!("SELECT 1")
        .fetch_one(&state.pool)
        .await
    {
        Ok(_) => DatabaseStatus {
            status: "healthy".to_string(),
            latency_ms: db_start.elapsed().as_millis() as u64,
        },
        Err(e) => DatabaseStatus {
            status: format!("unhealthy: {}", e),
            latency_ms: db_start.elapsed().as_millis() as u64,
        },
    };

    // Check Redis health
    let redis_start = std::time::Instant::now();
    let redis_status = match redis::Client::open(state.redis_url.clone())
        .and_then(|client| client.get_connection())
    {
        Ok(_) => RedisStatus {
            status: "healthy".to_string(),
            latency_ms: redis_start.elapsed().as_millis() as u64,
        },
        Err(e) => RedisStatus {
            status: format!("unhealthy: {}", e),
            latency_ms: redis_start.elapsed().as_millis() as u64,
        },
    };

    let response = HealthResponse {
        status: if db_status.status == "healthy" && redis_status.status == "healthy" {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: db_status,
        redis: redis_status,
        uptime: start_time.elapsed().as_secs(),
    };

    let status = if response.status == "healthy" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(response))
}

pub async fn readiness_check(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Check if the application is ready to accept traffic
    let db_ready = sqlx::query!("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .is_ok();

    let redis_ready = redis::Client::open(state.redis_url.clone())
        .and_then(|client| client.get_connection())
        .is_ok();

    if db_ready && redis_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

pub async fn liveness_check() -> impl IntoResponse {
    // Simple check if the application is running
    StatusCode::OK
} 