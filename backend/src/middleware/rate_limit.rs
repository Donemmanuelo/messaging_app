use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use redis::{Client, Commands};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::limit::RateLimitLayer;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::sync::Semaphore;
use std::time::Instant;

const RATE_LIMIT_WINDOW: u64 = 60; // 1 minute window
const MAX_REQUESTS_PER_WINDOW: u32 = 100; // 100 requests per minute

#[derive(Clone)]
pub struct RateLimiter {
    redis: Client,
    semaphore: Arc<Semaphore>,
}

impl RateLimiter {
    pub fn new(redis_url: String) -> Self {
        Self {
            redis: Client::open(redis_url).expect("Failed to create Redis client"),
            semaphore: Arc::new(Semaphore::new(MAX_REQUESTS_PER_WINDOW as usize)),
        }
    }

    pub async fn check_rate_limit(&self, key: &str) -> Result<(), StatusCode> {
        let mut conn = self.redis.get_connection()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let current = Instant::now();
        let window_key = format!("rate_limit:{}:{}", key, current.as_secs() / RATE_LIMIT_WINDOW);

        let count: u32 = conn.incr(&window_key, 1)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if count == 1 {
            conn.expire(&window_key, RATE_LIMIT_WINDOW as usize)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        if count > MAX_REQUESTS_PER_WINDOW {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        Ok(())
    }
}

pub async fn rate_limit_middleware<B>(
    State(rate_limiter): State<Arc<RateLimiter>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let key = req
        .headers()
        .get("x-forwarded-for")
        .or_else(|| req.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    rate_limiter.check_rate_limit(key).await?;

    Ok(next.run(req).await)
}

pub fn create_rate_limit_layer() -> RateLimitLayer {
    RateLimitLayer::new(NonZeroU32::new(MAX_REQUESTS_PER_WINDOW).unwrap())
        .timeout(Duration::from_secs(RATE_LIMIT_WINDOW))
} 