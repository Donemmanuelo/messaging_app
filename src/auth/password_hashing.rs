use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{Error, SaltString};
use rand::rngs::OsRng;
use log::{info, error};

pub fn hash_password(password: &str) -> Result<String, Error> {
    info!("Hashing password for user");
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    info!("Password hashed successfully");
    Ok(password_hash.to_string())
}

pub fn verify_password(password: &[u8], hashed_password: &str) -> Result<(), Error> {
    info!("Verifying password for user");
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(hashed_password)?;
    match argon2.verify_password(password, &password_hash) {
        Ok(_) => {
            info!("Password verified successfully");
            Ok(())
        }
        Err(e) => {
            error!("Password verification failed: {}", e);
            Err(e)
        }
    }
}