use axum::{
    http::{Request, Response, HeaderValue, header},
    middleware::Next,
};
use tower_http::security::SecurityHeadersLayer;
use std::time::Duration;

pub fn security_headers() -> SecurityHeadersLayer {
    SecurityHeadersLayer::new()
        .content_security_policy("default-src 'self'; img-src 'self' https: data:; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline';")
        .x_content_type_options("nosniff")
        .x_frame_options("DENY")
        .x_xss_protection("1; mode=block")
        .strict_transport_security(Duration::from_secs(31536000))
        .referrer_policy("strict-origin-when-cross-origin")
}

pub async fn validate_request<B>(req: Request<B>, next: Next<B>) -> Result<Response, axum::http::StatusCode> {
    // Validate content type for POST/PUT requests
    if matches!(req.method(), &axum::http::Method::POST | &axum::http::Method::PUT) {
        if let Some(content_type) = req.headers().get(header::CONTENT_TYPE) {
            if !content_type.to_str().unwrap_or("").starts_with("application/json") {
                return Err(axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE);
            }
        }
    }

    // Validate request size
    if let Some(content_length) = req.headers().get(header::CONTENT_LENGTH) {
        if let Ok(length) = content_length.to_str().unwrap_or("0").parse::<usize>() {
            if length > 10 * 1024 * 1024 { // 10MB limit
                return Err(axum::http::StatusCode::PAYLOAD_TOO_LARGE);
            }
        }
    }

    Ok(next.run(req).await)
}

pub async fn add_security_headers<B>(req: Request<B>, next: Next<B>) -> Response {
    let mut response = next.run(req).await;
    
    let headers = response.headers_mut();
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(header::X_XSS_PROTECTION, HeaderValue::from_static("1; mode=block"));
    headers.insert(header::REFERRER_POLICY, HeaderValue::from_static("strict-origin-when-cross-origin"));
    
    if let Some(origin) = req.headers().get(header::ORIGIN) {
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone());
    }
    
    response
} 