use actix_web::{web, HttpResponse, Error};
use actix_web_actors::ws;
use serde_json::json;

use crate::{
    services::auth::AuthService,
    utils::error::ApiError,
    websocket::Session,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ws")
            .route("/connect", web::get().to(websocket_route)),
    );
}

async fn websocket_route(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    auth: AuthService,
) -> Result<HttpResponse, Error> {
    // Extract token from query parameters
    let token = req.query_string()
        .split('&')
        .find(|param| param.starts_with("token="))
        .and_then(|param| param.split('=').nth(1))
        .ok_or_else(|| ApiError::new(401, "Missing token"))?;

    // Verify token and get user ID
    let claims = auth.verify_token(token)?;
    let user_id = claims.sub;

    // Create new WebSocket session
    let session = Session::new(user_id, auth.clone());
    
    // Start WebSocket connection
    ws::start(session, &req, stream)
} 