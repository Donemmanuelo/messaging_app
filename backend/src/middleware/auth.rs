use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::services::AppState;
use crate::services::jwt::verify_jwt_token;

// User ID extractor for request extensions
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
}

// JWT authentication middleware
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract the token from the Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if the header has the Bearer prefix
    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Extract the token without the Bearer prefix
    let token = &auth_header[7..];

    // Verify the token
    let claims = verify_jwt_token(token, &state.config.jwt_secret)?;

    // Parse the user ID from the claims
    let user_id = claims.sub.parse::<Uuid>()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add the user ID to the request extensions
    request.extensions_mut().insert(AuthUser { user_id });

    // Continue with the request
    Ok(next.run(request).await)
}

// Middleware for WebSocket authentication
pub async fn ws_auth_middleware(
    token: &str,
    state: &AppState,
) -> Result<Uuid, StatusCode> {
    // Verify the token
    let claims = verify_jwt_token(token, &state.config.jwt_secret)?;

    // Parse the user ID from the claims
    let user_id = claims.sub.parse::<Uuid>()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(user_id)
}