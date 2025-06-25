use crate::AppState;
use axum::{
    async_trait,
    extract::FromRequestParts,
    extract::{Request, State},
    http::request::Parts,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::Layer;
use tower::Service;
use uuid::Uuid;

// Comment out or remove the unresolved import of verify_token from services::jwt
// use crate::services::jwt::verify_token;

// User ID extractor for request extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

#[derive(Debug, Clone)]
pub struct AuthLayer;

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        AuthMiddleware::new(service)
    }
}

#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
}

impl<S> AuthMiddleware<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Layer<S> for AuthMiddleware<S> {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware::new(inner)
    }
}

impl<S> Service<Request> for AuthMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static + Clone,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let state = req.extensions().get::<Arc<AppState>>().cloned();
        let auth_header = req.headers().get(header::AUTHORIZATION).cloned();

        let mut inner = self.inner.clone();
        Box::pin(async move {
            if let (Some(state), Some(auth_header)) = (state, auth_header) {
                if let Ok(token) = auth_header.to_str() {
                    if let Some(token) = token.strip_prefix("Bearer ") {
                        let decoding_key = DecodingKey::from_secret(
                            std::env::var("JWT_SECRET")
                                .expect("JWT_SECRET must be set")
                                .as_bytes(),
                        );
                        if let Ok(claims) = decode::<Claims>(
                            token,
                            &decoding_key,
                            &Validation::new(Algorithm::HS256),
                        ) {
                            let auth_user = AuthUser {
                                id: Uuid::parse_str(&claims.claims.sub).unwrap_or(Uuid::nil()),
                                email: "".to_string(), // We don't need the email in the middleware
                            };
                            req.extensions_mut().insert(auth_user);
                        }
                    }
                }
            }
            inner.call(req).await
        })
    }
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or((StatusCode::UNAUTHORIZED, "Missing or invalid token"))
    }
}

pub fn auth_middleware() -> AuthMiddleware<()> {
    AuthMiddleware::new(())
}

// Middleware for WebSocket authentication
pub async fn ws_auth_middleware(
    token: String,
    state: Arc<AppState>,
) -> Result<AuthUser, (StatusCode, &'static str)> {
    let decoding_key = DecodingKey::from_secret(
        std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set")
            .as_bytes(),
    );
    if let Ok(claims) = decode::<Claims>(&token, &decoding_key, &Validation::new(Algorithm::HS256))
    {
        Ok(AuthUser {
            id: Uuid::parse_str(&claims.claims.sub).unwrap_or(Uuid::nil()),
            email: "".to_string(), // We don't need the email for WebSocket auth
        })
    } else {
        Err((StatusCode::UNAUTHORIZED, "Invalid token"))
    }
}
