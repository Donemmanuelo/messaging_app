use crate::auth::jwt::generate_jwt;
use crate::auth::password_hashing::hash_password;
use crate::database::models::User;
use crate::database::db_client::DbClient;
use argon2::password_hash::Error;
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use log::{info, error};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct AuthService;

impl AuthService {
    pub async fn register_user(&self, username: &str, password: &str, db: &DbClient) -> Result<String, Error> {
        let hashed_password = hash_password(password)?;
        let user = User {
            id: None,
            username: username.to_string(),
            password: hashed_password,
        };
        if let Err(e) = db.create_user(user).await {
            error!("Failed to create user: {}", e);
            return Err(e);
        }
        info!("User registered successfully: {}", username);
        Ok("User registered successfully".to_string())
    }

    pub async fn login_user(&self, username: &str, password: &str, db: &DbClient) -> Result<String, Error> {
        let user = match db.get_user_by_username(username).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                error!("User not found: {}", username);
                return Err(argon2::password_hash::Error::Password);
            }
            Err(e) => {
                error!("Failed to get user: {}", e);
                return Err(e);
            }
        };

        if let Err(e) = argon2::verify_password(password.as_bytes(), &user.password) {
            error!("Failed to verify password: {}", e);
            return Err(e);
        }

        let claims = Claims {
            sub: user.id.unwrap().to_string(),
            exp: (std::time::SystemTime::now() + std::time::Duration::from_secs(3600))
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize,
        };

        let token = match generate_jwt(&claims) {
            Ok(token) => token,
            Err(e) => {
                error!("Failed to generate JWT: {}", e);
                return Err(e);
            }
        };

        info!("User logged in successfully: {}", username);
        Ok(token)
    }
}