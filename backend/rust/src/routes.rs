use actix_web::{web, HttpResponse, Responder, error::ResponseError};
use actix_web_actors::ws;
use uuid::Uuid;
use crate::websocket::Session;
use crate::AppState;
use crate::models::{NewUser, User, NewChat, Chat, NewMessage, Message, UserDTO, ChatDTO, MessageDTO};
use crate::services::db::DbService;
use crate::services::auth::{
    LoginRequest, LoginResponse, create_access_token, create_refresh_token,
    create_verification_token, create_password_reset_token,
    AuthenticatedUser, AuthError, RefreshRequest, PasswordResetRequest, NewPasswordRequest,
    TokenType, validate_password_strength, ResendVerificationRequest,
    ChangePasswordRequest, SessionManager, SessionActivity,
};
use crate::services::rate_limiter::{RateLimiter, RateLimited};
use crate::services::email::EmailService;
use serde_json::json;
use std::sync::Arc;
use validator::Validate;
use actix_web::http::header;

// Custom error type for API responses
#[derive(Debug)]
pub struct ApiError {
    message: String,
    status: actix_web::http::StatusCode,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        self.status
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status)
            .json(json!({
                "error": self.message
            }))
    }
}

// Authentication routes
pub async fn login(
    login_data: web::Json<LoginRequest>,
    db: web::Data<Arc<DbService>>,
    session_manager: web::Data<Arc<SessionManager>>,
    req: HttpRequest,
    _: RateLimited,
) -> Result<HttpResponse, AuthError> {
    login_data.validate()?;

    let user = db.verify_user_credentials(&login_data.email, &login_data.password)
        .map_err(|_| AuthError::InvalidCredentials)?;

    let device_id = req.headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let ip_address = req.connection_info()
        .remote_addr()
        .unwrap_or("unknown")
        .to_string();

    let session_id = session_manager.create_session(
        user.id,
        device_id.clone(),
        ip_address.clone(),
        Some(format!("Browser: {}", device_id)),
    )?;

    let access_token = create_access_token(user.id)?;
    let refresh_token = create_refresh_token(user.id)?;

    session_manager.update_session_activity(
        &session_id,
        "login",
        Some(format!("Successful login from {}", ip_address)),
    )?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token,
        user_id: user.id,
    }))
}

pub async fn refresh_token(
    refresh_data: web::Json<RefreshRequest>,
    db: web::Data<Arc<DbService>>,
    _: RateLimited,
) -> Result<HttpResponse, AuthError> {
    let token_data = jsonwebtoken::decode::<Claims>(
        &refresh_data.refresh_token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    ).map_err(|_| AuthError::InvalidRefreshToken)?;

    if !token_data.claims.refresh {
        return Err(AuthError::InvalidRefreshToken);
    }

    let user = db.get_user(token_data.claims.sub)
        .map_err(|_| AuthError::InvalidCredentials)?;

    let access_token = create_access_token(user.id)?;
    let refresh_token = create_refresh_token(user.id)?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token,
        user_id: user.id,
    }))
}

pub async fn request_password_reset(
    reset_request: web::Json<PasswordResetRequest>,
    db: web::Data<Arc<DbService>>,
    email_service: web::Data<Arc<EmailService>>,
    _: RateLimited,
) -> Result<HttpResponse, AuthError> {
    reset_request.validate()?;

    let user = db.get_user_by_email(&reset_request.email)
        .map_err(|_| AuthError::InvalidCredentials)?;

    let reset_token = create_password_reset_token(user.id)?;
    email_service.send_password_reset_email(&user.email, &reset_token)?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Password reset email sent"
    })))
}

pub async fn reset_password(
    reset_data: web::Json<NewPasswordRequest>,
    db: web::Data<Arc<DbService>>,
    _: RateLimited,
) -> Result<HttpResponse, AuthError> {
    reset_data.validate()?;
    validate_password_strength(&reset_data.new_password)?;

    let token_data = jsonwebtoken::decode::<Claims>(
        &reset_data.token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    ).map_err(|_| AuthError::PasswordReset)?;

    if token_data.claims.token_type != TokenType::PasswordReset {
        return Err(AuthError::PasswordReset);
    }

    let hashed_password = hash_password(&reset_data.new_password)?;
    db.update_user_password(token_data.claims.sub, &hashed_password)
        .map_err(|_| AuthError::PasswordReset)?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Password reset successful"
    })))
}

pub async fn verify_email(
    token: web::Path<String>,
    db: web::Data<Arc<DbService>>,
) -> Result<HttpResponse, AuthError> {
    let token_data = jsonwebtoken::decode::<Claims>(
        &token.into_inner(),
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    ).map_err(|_| AuthError::EmailVerification)?;

    if token_data.claims.token_type != TokenType::Verification {
        return Err(AuthError::EmailVerification);
    }

    db.verify_user_email(token_data.claims.sub)
        .map_err(|_| AuthError::EmailVerification)?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Email verified successfully"
    })))
}

pub async fn register(
    user_data: web::Json<NewUser>,
    db: web::Data<Arc<DbService>>,
    email_service: web::Data<Arc<EmailService>>,
    _: RateLimited,
) -> Result<HttpResponse, AuthError> {
    user_data.validate()?;
    validate_password_strength(&user_data.password)?;

    // Check if user already exists
    if let Ok(_) = db.get_user_by_email(&user_data.email) {
        return Err(AuthError::InvalidCredentials);
    }

    let user = db.create_user(&user_data.into_inner())
        .map_err(|_| AuthError::InvalidCredentials)?;

    let verification_token = create_verification_token(user.id)?;
    email_service.send_verification_email(&user.email, &verification_token)?;

    let access_token = create_access_token(user.id)?;
    let refresh_token = create_refresh_token(user.id)?;

    Ok(HttpResponse::Created().json(LoginResponse {
        access_token,
        refresh_token,
        user_id: user.id,
    }))
}

// Update existing routes to use authentication
pub async fn ws_index(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let session = Session {
        id: Uuid::new_v4().to_string(),
        user_id: auth.user_id,
        hb: std::time::Instant::now(),
        state: state.get_ref().clone(),
    };

    let resp = ws::start(session, &req, stream)?;
    Ok(resp)
}

// User routes
pub async fn create_user(
    user: web::Json<NewUser>,
    db: web::Data<Arc<DbService>>,
) -> Result<HttpResponse, ApiError> {
    let user = db.create_user(&user.into_inner())
        .map_err(|e| ApiError {
            message: format!("Failed to create user: {}", e),
            status: actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(HttpResponse::Created().json(UserDTO::from(user)))
}

pub async fn get_user(
    user_id: web::Path<i32>,
    db: web::Data<Arc<DbService>>,
) -> Result<HttpResponse, ApiError> {
    let user = db.get_user(user_id.into_inner())
        .map_err(|e| ApiError {
            message: format!("Failed to get user: {}", e),
            status: actix_web::http::StatusCode::NOT_FOUND,
        })?;

    Ok(HttpResponse::Ok().json(UserDTO::from(user)))
}

// Chat routes
pub async fn create_chat(
    chat_data: web::Json<NewChat>,
    db: web::Data<Arc<DbService>>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AuthError> {
    if !auth.email_verified {
        return Err(AuthError::EmailNotVerified);
    }

    let chat = db.create_chat(&chat_data.into_inner())
        .map_err(|_| AuthError::InvalidCredentials)?;

    Ok(HttpResponse::Created().json(ChatDTO::from(chat)))
}

pub async fn get_chat(
    chat_id: web::Path<i32>,
    db: web::Data<Arc<DbService>>,
) -> Result<HttpResponse, ApiError> {
    let chat = db.get_chat(chat_id.into_inner())
        .map_err(|e| ApiError {
            message: format!("Failed to get chat: {}", e),
            status: actix_web::http::StatusCode::NOT_FOUND,
        })?;

    let participants = db.get_chat_participants(chat.id)
        .map_err(|e| ApiError {
            message: format!("Failed to get chat participants: {}", e),
            status: actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    let mut chat_dto = ChatDTO::from(chat);
    chat_dto.participants = participants.into_iter().map(UserDTO::from).collect();
    Ok(HttpResponse::Ok().json(chat_dto))
}

pub async fn get_user_chats(
    user_id: web::Path<i32>,
    db: web::Data<Arc<DbService>>,
) -> Result<HttpResponse, ApiError> {
    let chats = db.get_user_chats(user_id.into_inner())
        .map_err(|e| ApiError {
            message: format!("Failed to get user chats: {}", e),
            status: actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    let mut chat_dtos = Vec::new();
    for chat in chats {
        let participants = db.get_chat_participants(chat.id)
            .map_err(|e| ApiError {
                message: format!("Failed to get chat participants: {}", e),
                status: actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let mut chat_dto = ChatDTO::from(chat);
        chat_dto.participants = participants.into_iter().map(UserDTO::from).collect();
        chat_dtos.push(chat_dto);
    }

    Ok(HttpResponse::Ok().json(json!({ "chats": chat_dtos })))
}

// Message routes
pub async fn send_message(
    message_data: web::Json<NewMessage>,
    db: web::Data<Arc<DbService>>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AuthError> {
    if !auth.email_verified {
        return Err(AuthError::EmailNotVerified);
    }

    let message = db.create_message(&message_data.into_inner())
        .map_err(|_| AuthError::InvalidCredentials)?;

    Ok(HttpResponse::Created().json(MessageDTO::from(message)))
}

pub async fn get_chat_messages(
    chat_id: web::Path<i32>,
    db: web::Data<Arc<DbService>>,
) -> Result<HttpResponse, ApiError> {
    let messages = db.get_chat_messages(chat_id.into_inner())
        .map_err(|e| ApiError {
            message: format!("Failed to get chat messages: {}", e),
            status: actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    let message_dtos: Vec<MessageDTO> = messages.into_iter().map(MessageDTO::from).collect();
    Ok(HttpResponse::Ok().json(json!({ "messages": message_dtos })))
}

pub async fn resend_verification_email(
    request: web::Json<ResendVerificationRequest>,
    db: web::Data<Arc<DbService>>,
    email_service: web::Data<Arc<EmailService>>,
    _: RateLimited,
) -> Result<HttpResponse, AuthError> {
    request.validate()?;

    let user = db.get_user_by_email(&request.email)
        .map_err(|_| AuthError::UserNotFound)?;

    if user.email_verified {
        return Err(AuthError::EmailAlreadyVerified);
    }

    let verification_token = create_verification_token(user.id)?;
    email_service.send_verification_email(&user.email, &verification_token)?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Verification email sent"
    })))
}

pub async fn get_user_profile(
    db: web::Data<Arc<DbService>>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AuthError> {
    let user = db.get_user(auth.user_id)
        .map_err(|_| AuthError::UserNotFound)?;

    Ok(HttpResponse::Ok().json(json!({
        "id": user.id,
        "username": user.username,
        "email": user.email,
        "email_verified": user.email_verified,
        "created_at": user.created_at,
        "updated_at": user.updated_at
    })))
}

pub async fn change_password(
    password_data: web::Json<ChangePasswordRequest>,
    db: web::Data<Arc<DbService>>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AuthError> {
    password_data.validate()?;
    validate_password_strength(&password_data.new_password)?;

    let user = db.get_user(auth.user_id)
        .map_err(|_| AuthError::UserNotFound)?;

    if !verify(&password_data.current_password, &user.password_hash)? {
        return Err(AuthError::IncorrectPassword);
    }

    let hashed_password = hash_password(&password_data.new_password)?;
    db.update_user_password(auth.user_id, &hashed_password)
        .map_err(|_| AuthError::InvalidCredentials)?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Password changed successfully"
    })))
}

pub async fn get_sessions(
    db: web::Data<Arc<DbService>>,
    session_manager: web::Data<Arc<SessionManager>>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AuthError> {
    let sessions = session_manager.get_user_sessions(auth.user_id);
    Ok(HttpResponse::Ok().json(json!({ "sessions": sessions })))
}

pub async fn logout(
    session_manager: web::Data<Arc<SessionManager>>,
    req: HttpRequest,
) -> Result<HttpResponse, AuthError> {
    if let Some(session_id) = req.headers().get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
    {
        session_manager.remove_session(session_id);
    }

    Ok(HttpResponse::Ok().json(json!({
        "message": "Logged out successfully"
    })))
}

pub async fn revoke_session(
    session_id: web::Path<String>,
    session_manager: web::Data<Arc<SessionManager>>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AuthError> {
    session_manager.revoke_session(&session_id.into_inner())?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Session revoked successfully"
    })))
}

pub async fn revoke_all_sessions(
    session_manager: web::Data<Arc<SessionManager>>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AuthError> {
    session_manager.revoke_all_sessions(auth.user_id);
    Ok(HttpResponse::Ok().json(json!({
        "message": "All sessions revoked successfully"
    })))
}

pub async fn cleanup_sessions(
    session_manager: web::Data<Arc<SessionManager>>,
) -> Result<HttpResponse, AuthError> {
    session_manager.cleanup_expired_sessions();
    Ok(HttpResponse::Ok().json(json!({
        "message": "Expired sessions cleaned up successfully"
    })))
}

pub async fn get_session_activities(
    session_id: web::Path<String>,
    session_manager: web::Data<Arc<SessionManager>>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AuthError> {
    let activities = session_manager.get_session_activities(&session_id.into_inner())?;
    Ok(HttpResponse::Ok().json(json!({ "activities": activities })))
}

pub async fn get_session_count(
    session_manager: web::Data<Arc<SessionManager>>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AuthError> {
    let count = session_manager.get_user_session_count(auth.user_id);
    Ok(HttpResponse::Ok().json(json!({ "session_count": count })))
}

pub async fn cleanup_oldest_session(
    session_manager: web::Data<Arc<SessionManager>>,
    auth: AuthenticatedUser,
) -> Result<HttpResponse, AuthError> {
    session_manager.cleanup_oldest_session(auth.user_id)?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Oldest session cleaned up successfully"
    })))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    let rate_limiter = RateLimiter::new(10.0, 1.0); // 10 requests per second
    let email_service = EmailService::new(
        &env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
        env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string()).parse().unwrap(),
        &env::var("SMTP_USERNAME").unwrap_or_else(|_| "".to_string()),
        &env::var("SMTP_PASSWORD").unwrap_or_else(|_| "".to_string()),
        &env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@example.com".to_string()),
    );
    let session_manager = Arc::new(SessionManager::new());

    cfg.app_data(web::Data::new(rate_limiter))
        .app_data(web::Data::new(Arc::new(email_service)))
        .app_data(web::Data::new(session_manager.clone()))
        .service(
            web::scope("/api")
                .route("/auth/login", web::post().to(login))
                .route("/auth/register", web::post().to(register))
                .route("/auth/refresh", web::post().to(refresh_token))
                .route("/auth/verify-email/{token}", web::get().to(verify_email))
                .route("/auth/resend-verification", web::post().to(resend_verification_email))
                .route("/auth/request-password-reset", web::post().to(request_password_reset))
                .route("/auth/reset-password", web::post().to(reset_password))
                .route("/auth/change-password", web::post().to(change_password))
                .route("/auth/logout", web::post().to(logout))
                .route("/auth/sessions", web::get().to(get_sessions))
                .route("/auth/sessions/count", web::get().to(get_session_count))
                .route("/auth/sessions/{session_id}/activities", web::get().to(get_session_activities))
                .route("/auth/sessions/{session_id}/revoke", web::post().to(revoke_session))
                .route("/auth/sessions/revoke-all", web::post().to(revoke_all_sessions))
                .route("/auth/sessions/cleanup", web::post().to(cleanup_sessions))
                .route("/auth/sessions/cleanup-oldest", web::post().to(cleanup_oldest_session))
                .route("/ws", web::get().to(ws_index))
                .service(
                    web::scope("/users")
                        .route("", web::post().to(create_user))
                        .route("/{user_id}", web::get().to(get_user))
                        .route("/{user_id}/chats", web::get().to(get_user_chats))
                        .route("/profile", web::get().to(get_user_profile))
                )
                .service(
                    web::scope("/chats")
                        .route("", web::post().to(create_chat))
                        .route("/{chat_id}", web::get().to(get_chat))
                        .route("/{chat_id}/messages", web::get().to(get_chat_messages))
                )
                .service(
                    web::scope("/messages")
                        .route("", web::post().to(send_message))
                )
        );
} 