use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use log::{info, error};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn generate_jwt(claims: &Claims) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    info!("Generating JWT for user: {}", claims.sub);
    encode(&Header::default(), claims, &encoding_key)
}