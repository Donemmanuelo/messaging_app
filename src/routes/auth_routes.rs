use actix_web::{web, HttpResponse, Responder};
use crate::auth::AuthService;
use crate::database::db_client::DbClient;
use serde::{Deserialize, Serialize};
use log::{info, error};

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/register").route(web::post().to(register_user)))
       .service(web::resource("/login").route(web::post().to(login_user)));
}

pub async fn register_user(
    req: web::Json<RegisterRequest>,
    db: web::Data<DbClient>,
) -> impl Responder {
    let auth_service = AuthService;
    match auth_service.register_user(&req.username, &req.password, &db).await {
        Ok(_) => {
            info!("User registered successfully: {}", req.username);
            HttpResponse::Ok().json("User registered successfully")
        }
        Err(e) => {
            error!("Failed to register user: {}", e);
            HttpResponse::InternalServerError().json("Failed to register user")
        }
    }
}

pub async fn login_user(
    req: web::Json<LoginRequest>,
    db: web::Data<DbClient>,
) -> impl Responder {
    let auth_service = AuthService;
    match auth_service.login_user(&req.username, &req.password, &db).await {
        Ok(token) => {
            info!("User logged in successfully: {}", req.username);
            HttpResponse::Ok().json(AuthResponse { token })
        }
        Err(e) => {
            error!("Failed to log in user: {}", e);
            HttpResponse::InternalServerError().json("Failed to log in user")
        }
    }
}