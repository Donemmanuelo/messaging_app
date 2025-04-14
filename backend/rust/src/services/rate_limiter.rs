use std::sync::Mutex;
use std::time::{Duration, Instant};
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorTooManyRequests;
use actix_web::{Error, FromRequest};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    capacity: f64,
    refill_rate: f64,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
            capacity,
            refill_rate,
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let time_passed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = time_passed * self.refill_rate;
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }
}

pub struct RateLimiter {
    buckets: Mutex<std::collections::HashMap<String, TokenBucket>>,
    default_capacity: f64,
    default_refill_rate: f64,
}

impl RateLimiter {
    pub fn new(default_capacity: f64, default_refill_rate: f64) -> Self {
        Self {
            buckets: Mutex::new(std::collections::HashMap::new()),
            default_capacity,
            default_refill_rate,
        }
    }

    pub fn check_rate_limit(&self, key: &str) -> Result<(), Error> {
        let mut buckets = self.buckets.lock().unwrap();
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(self.default_capacity, self.default_refill_rate));

        if bucket.try_consume(1.0) {
            Ok(())
        } else {
            Err(ErrorTooManyRequests("Rate limit exceeded"))
        }
    }
}

pub struct RateLimited {
    pub key: String,
}

impl FromRequest for RateLimited {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Error>>>>;

    fn from_request(req: &ServiceRequest) -> Self::Future {
        let limiter = req.app_data::<RateLimiter>().unwrap().clone();
        let key = req
            .headers()
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        Box::pin(async move {
            limiter.check_rate_limit(&key)?;
            Ok(RateLimited { key })
        })
    }
} 