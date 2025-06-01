use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug)]
pub struct RateLimiter {
    limits: HashMap<Uuid, Vec<Instant>>,
    window: Duration,
    max_requests: usize,
}

impl RateLimiter {
    pub fn new(window: Duration, max_requests: usize) -> Self {
        Self {
            limits: HashMap::new(),
            window,
            max_requests,
        }
    }

    pub fn check_rate_limit(&mut self, user_id: Uuid) -> bool {
        let now = Instant::now();
        let window_start = now - self.window;

        // Get or create the user's request timestamps
        let timestamps = self.limits.entry(user_id).or_insert_with(Vec::new);

        // Remove old timestamps
        timestamps.retain(|&t| t > window_start);

        // Check if under the limit
        if timestamps.len() < self.max_requests {
            timestamps.push(now);
            true
        } else {
            false
        }
    }

    pub fn cleanup(&mut self) {
        let now = Instant::now();
        let window_start = now - self.window;

        // Remove expired entries
        self.limits.retain(|_, timestamps| {
            timestamps.retain(|&t| t > window_start);
            !timestamps.is_empty()
        });
    }
}

#[derive(Debug)]
pub struct ConnectionPool {
    connections: HashMap<Uuid, usize>,
    max_connections: usize,
}

impl ConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: HashMap::new(),
            max_connections,
        }
    }

    pub fn can_connect(&self, user_id: Uuid) -> bool {
        self.connections.get(&user_id).map_or(true, |&count| count < self.max_connections)
    }

    pub fn add_connection(&mut self, user_id: Uuid) {
        *self.connections.entry(user_id).or_insert(0) += 1;
    }

    pub fn remove_connection(&mut self, user_id: Uuid) {
        if let Some(count) = self.connections.get_mut(&user_id) {
            if *count > 0 {
                *count -= 1;
            }
            if *count == 0 {
                self.connections.remove(&user_id);
            }
        }
    }
}

#[derive(Debug)]
pub struct WebSocketManager {
    rate_limiter: Arc<RwLock<RateLimiter>>,
    connection_pool: Arc<RwLock<ConnectionPool>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new(
                Duration::from_secs(60), // 1 minute window
                100, // 100 requests per minute
            ))),
            connection_pool: Arc::new(RwLock::new(ConnectionPool::new(3))), // 3 connections per user
        }
    }

    pub async fn check_rate_limit(&self, user_id: Uuid) -> bool {
        self.rate_limiter.write().await.check_rate_limit(user_id)
    }

    pub async fn can_connect(&self, user_id: Uuid) -> bool {
        self.connection_pool.read().await.can_connect(user_id)
    }

    pub async fn add_connection(&self, user_id: Uuid) {
        self.connection_pool.write().await.add_connection(user_id);
    }

    pub async fn remove_connection(&self, user_id: Uuid) {
        self.connection_pool.write().await.remove_connection(user_id);
    }

    pub async fn cleanup(&self) {
        self.rate_limiter.write().await.cleanup();
    }
} 