use std::{
    sync::atomic::{AtomicU32, Ordering},
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    failures: Arc<AtomicU32>,
    last_failure: Arc<RwLock<Instant>>,
    threshold: u32,
    reset_timeout: Duration,
    state: Arc<RwLock<CircuitState>>,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            failures: Arc::new(AtomicU32::new(0)),
            last_failure: Arc::new(RwLock::new(Instant::now())),
            threshold,
            reset_timeout,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
        }
    }

    pub async fn execute<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let state = *self.state.read().await;
        
        match state {
            CircuitState::Open => {
                if self.should_reset().await {
                    self.reset().await;
                } else {
                    return Err(f().unwrap_err());
                }
            }
            CircuitState::HalfOpen => {
                if let Ok(result) = f() {
                    self.on_success().await;
                    return Ok(result);
                } else {
                    self.on_failure().await;
                    return Err(f().unwrap_err());
                }
            }
            CircuitState::Closed => {
                if let Ok(result) = f() {
                    return Ok(result);
                } else {
                    self.on_failure().await;
                    return Err(f().unwrap_err());
                }
            }
        }

        f()
    }

    async fn should_reset(&self) -> bool {
        let last_failure = *self.last_failure.read().await;
        last_failure.elapsed() >= self.reset_timeout
    }

    async fn reset(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::HalfOpen;
        self.failures.store(0, Ordering::SeqCst);
    }

    async fn on_success(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Closed;
        self.failures.store(0, Ordering::SeqCst);
    }

    async fn on_failure(&self) {
        let failures = self.failures.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure.write().await = Instant::now();

        if failures >= self.threshold {
            let mut state = self.state.write().await;
            *state = CircuitState::Open;
        }
    }
}

pub async fn graceful_shutdown(
    signal: impl std::future::Future<Output = ()>,
    server: axum::Server<axum::Router>,
) {
    let graceful = server.with_graceful_shutdown(signal);
    
    if let Err(e) = graceful.await {
        eprintln!("Server error: {}", e);
    }
} 