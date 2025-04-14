use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use validator::Validate;

use crate::{
    models::user::{UserCreate, UserUpdate},
    services::auth::{AuthService, LoginRequest, RefreshRequest},
    utils::{error::ApiError, validation::validate_request},
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh_token))
            .route("/verify/{token}", web::get().to(verify_email))
            .route("/reset-password", web::post().to(request_password_reset))
            .route("/reset-password/{token}", web::post().to(reset_password)),
    );
}

async fn register(
    user_data: web::Json<UserCreate>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse, ApiError> {
    validate_request(&user_data)?;
    
    let result = auth_service.register(user_data.into_inner()).await?;
    Ok(HttpResponse::Created().json(json!({
        "message": "User registered successfully",
        "data": result
    })))
}

async fn login(
    login_data: web::Json<LoginRequest>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse, ApiError> {
    validate_request(&login_data)?;
    
    let result = auth_service.login(login_data.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

async fn refresh_token(
    refresh_data: web::Json<RefreshRequest>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse, ApiError> {
    validate_request(&refresh_data)?;
    
    let result = auth_service.refresh_token(refresh_data.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

async fn verify_email(
    token: web::Path<String>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse, ApiError> {
    auth_service.verify_email(token.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Email verified successfully"
    })))
}

async fn request_password_reset(
    email: web::Json<String>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse, ApiError> {
    auth_service.request_password_reset(email.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Password reset email sent"
    })))
}

async fn reset_password(
    token: web::Path<String>,
    new_password: web::Json<String>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse, ApiError> {
    auth_service.reset_password(token.into_inner(), new_password.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Password reset successfully"
    })))
} 