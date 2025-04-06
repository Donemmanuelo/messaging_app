use actix_web::{web, HttpResponse, Responder};
use crate::database::db_client::DbClient;
use serde::{Deserialize, Serialize};
use log::{info, error};

#[derive(Serialize)]
pub struct UserResponse {
    id: i32,
    username: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/users/{username}").route(web::get().to(get_user_by_username)));
}

pub async fn get_user_by_username(
    web::Path(username): web::Path<String>,
    db: web::Data<DbClient>,
) -> impl Responder {
    match crate::database::operations::get_user_by_username(&username, &db).await {
        Ok(Some(user)) => {
            info!("User found: {}", user.username);
            HttpResponse::Ok().json(UserResponse {
                id: user.id.unwrap(),
                username: user.username,
            })
        }
        Ok(None) => {
            info!("User not found: {}", username);
            HttpResponse::NotFound().json("User not found")
        }
        Err(e) => {
            error!("Failed to fetch user: {}", e);
            HttpResponse::InternalServerError().json("Failed to fetch user")
        }
    }
}