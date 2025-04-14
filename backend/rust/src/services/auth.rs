use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpRequest};
use actix_web::error::ErrorUnauthorized;
use chrono::{Duration, Utc};
use futures::future::{Ready, ok, err};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::future::Future;
use std::pin::Pin;
use validator::Validate;
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;
use crate::services::email::EmailService;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt;

use crate::{
    models::user::{User, UserCreate, UserUpdate},
    services::db::Database,
    utils::error::ApiError,
};

const JWT_SECRET: &[u8] = b"your-secret-key"; // In production, use environment variable
const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 30;
const VERIFICATION_TOKEN_EXPIRY_DAYS: i64 = 7;
const PASSWORD_RESET_TOKEN_EXPIRY_HOURS: i64 = 1;
const SESSION_EXPIRY_SECONDS: i64 = 24 * 60 * 60; // 24 hours
const SESSION_INACTIVITY_SECONDS: i64 = 30 * 60; // 30 minutes
const MAX_SESSIONS_PER_USER: usize = 5;
const ACCESS_TOKEN_EXPIRY: i64 = 3600; // 1 hour
const REFRESH_TOKEN_EXPIRY: i64 = 2592000; // 30 days

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub token_type: TokenType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
    Verification,
    PasswordReset,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PasswordResetRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewPasswordRequest {
    pub token: String,
    #[validate(length(min = 8))]
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 8))]
    pub current_password: String,
    #[validate(length(min = 8))]
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionActivity {
    pub timestamp: i64,
    pub action: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionFingerprint {
    pub user_agent: String,
    pub ip_address: String,
    pub device_type: String,
    pub browser: String,
    pub os: String,
    pub screen_resolution: Option<String>,
    pub timezone: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionNotification {
    pub timestamp: i64,
    pub notification_type: NotificationType,
    pub message: String,
    pub severity: NotificationSeverity,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum NotificationType {
    Login,
    Logout,
    PasswordChange,
    SessionRevoked,
    SuspiciousActivity,
    DeviceChange,
    LocationChange,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub user_id: i32,
    pub device_id: String,
    pub created_at: i64,
    pub last_active: i64,
    pub ip_address: String,
    pub device_info: Option<String>,
    pub revoked: bool,
    pub activities: Vec<SessionActivity>,
    pub fingerprint: SessionFingerprint,
    pub notifications: Vec<SessionNotification>,
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Session(user_id: {}, device: {}, created: {}, last_active: {}, ip: {})",
            self.user_id,
            self.device_id,
            self.created_at,
            self.last_active,
            self.ip_address
        )
    }
}

pub struct SessionManager {
    sessions: Mutex<HashMap<String, Session>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn create_session(&self, user_id: i32, device_id: String, ip_address: String, device_info: Option<String>, fingerprint: SessionFingerprint) -> Result<String, AuthError> {
        let mut sessions = self.sessions.lock().unwrap();
        
        // Check session limit
        let user_session_count = sessions.values()
            .filter(|session| session.user_id == user_id && !session.revoked)
            .count();
            
        if user_session_count >= MAX_SESSIONS_PER_USER {
            return Err(AuthError::SessionLimitExceeded);
        }

        let session_id = Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let session = Session {
            user_id,
            device_id,
            created_at: now,
            last_active: now,
            ip_address,
            device_info,
            revoked: false,
            activities: vec![SessionActivity {
                timestamp: now,
                action: "created".to_string(),
                details: Some(format!("Device: {}, IP: {}", device_id, ip_address)),
            }],
            fingerprint,
            notifications: vec![SessionNotification {
                timestamp: now,
                notification_type: NotificationType::Login,
                message: "New session created".to_string(),
                severity: NotificationSeverity::Info,
            }],
        };

        sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    pub fn update_session_activity(&self, session_id: &str, action: &str, details: Option<String>) -> Result<(), AuthError> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            if session.revoked {
                return Err(AuthError::SessionRevoked);
            }
            if self.is_session_expired(session) {
                sessions.remove(session_id);
                return Err(AuthError::SessionExpired);
            }

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            session.last_active = now;
            session.activities.push(SessionActivity {
                timestamp: now,
                action: action.to_string(),
                details,
            });

            Ok(())
        } else {
            Err(AuthError::InvalidSession)
        }
    }

    pub fn get_session_activities(&self, session_id: &str) -> Result<Vec<SessionActivity>, AuthError> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            Ok(session.activities.clone())
        } else {
            Err(AuthError::InvalidSession)
        }
    }

    pub fn get_user_session_count(&self, user_id: i32) -> usize {
        self.sessions.lock().unwrap()
            .values()
            .filter(|session| session.user_id == user_id && !session.revoked)
            .count()
    }

    pub fn remove_session(&self, session_id: &str) {
        self.sessions.lock().unwrap().remove(session_id);
    }

    pub fn revoke_session(&self, session_id: &str) -> Result<(), AuthError> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.revoked = true;
            Ok(())
        } else {
            Err(AuthError::InvalidSession)
        }
    }

    pub fn revoke_all_sessions(&self, user_id: i32) {
        let mut sessions = self.sessions.lock().unwrap();
        for session in sessions.values_mut() {
            if session.user_id == user_id {
                session.revoked = true;
            }
        }
    }

    pub fn get_user_sessions(&self, user_id: i32) -> Vec<Session> {
        self.sessions.lock().unwrap()
            .values()
            .filter(|session| session.user_id == user_id)
            .cloned()
            .collect()
    }

    fn is_session_expired(&self, session: &Session) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Check if session is older than SESSION_EXPIRY_SECONDS
        if now - session.created_at > SESSION_EXPIRY_SECONDS {
            return true;
        }

        // Check if session has been inactive for more than SESSION_INACTIVITY_SECONDS
        if now - session.last_active > SESSION_INACTIVITY_SECONDS {
            return true;
        }

        false
    }

    pub fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.retain(|_, session| !self.is_session_expired(session));
    }

    pub fn cleanup_oldest_session(&self, user_id: i32) -> Result<(), AuthError> {
        let mut sessions = self.sessions.lock().unwrap();
        let oldest_session = sessions.values()
            .filter(|session| session.user_id == user_id && !session.revoked)
            .min_by_key(|session| session.created_at);

        if let Some(oldest) = oldest_session {
            if let Some((id, _)) = sessions.iter()
                .find(|(_, session)| session.user_id == user_id && session.created_at == oldest.created_at)
            {
                sessions.remove(id);
                return Ok(());
            }
        }
        Err(AuthError::NoSessionsToCleanup)
    }

    pub fn create_session_fingerprint(&self, req: &HttpRequest) -> SessionFingerprint {
        let user_agent = req.headers()
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let ip_address = req.connection_info()
            .remote_addr()
            .unwrap_or("unknown")
            .to_string();

        // Parse user agent to get device and browser info
        let (device_type, browser, os) = self.parse_user_agent(&user_agent);

        SessionFingerprint {
            user_agent,
            ip_address,
            device_type,
            browser,
            os,
            screen_resolution: None, // This would be sent from the client
            timezone: "UTC".to_string(), // This would be sent from the client
        }
    }

    fn parse_user_agent(&self, user_agent: &str) -> (String, String, String) {
        // Simple parsing - in production, use a proper user agent parser
        let device_type = if user_agent.contains("Mobile") {
            "Mobile"
        } else if user_agent.contains("Tablet") {
            "Tablet"
        } else {
            "Desktop"
        }.to_string();

        let browser = if user_agent.contains("Chrome") {
            "Chrome"
        } else if user_agent.contains("Firefox") {
            "Firefox"
        } else if user_agent.contains("Safari") {
            "Safari"
        } else {
            "Unknown"
        }.to_string();

        let os = if user_agent.contains("Windows") {
            "Windows"
        } else if user_agent.contains("Mac") {
            "macOS"
        } else if user_agent.contains("Linux") {
            "Linux"
        } else if user_agent.contains("Android") {
            "Android"
        } else if user_agent.contains("iOS") {
            "iOS"
        } else {
            "Unknown"
        }.to_string();

        (device_type, browser, os)
    }

    pub fn check_suspicious_activity(&self, session: &Session, new_fingerprint: &SessionFingerprint) -> bool {
        // Check for suspicious changes in session fingerprint
        if session.fingerprint.device_type != new_fingerprint.device_type {
            return true;
        }
        if session.fingerprint.browser != new_fingerprint.browser {
            return true;
        }
        if session.fingerprint.os != new_fingerprint.os {
            return true;
        }
        if session.fingerprint.ip_address != new_fingerprint.ip_address {
            return true;
        }
        false
    }

    pub fn add_notification(&self, session_id: &str, notification: SessionNotification) -> Result<(), AuthError> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.notifications.push(notification);
            Ok(())
        } else {
            Err(AuthError::InvalidSession)
        }
    }

    pub fn get_notifications(&self, session_id: &str) -> Result<Vec<SessionNotification>, AuthError> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            Ok(session.notifications.clone())
        } else {
            Err(AuthError::InvalidSession)
        }
    }

    pub fn clear_notifications(&self, session_id: &str) -> Result<(), AuthError> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.notifications.clear();
            Ok(())
        } else {
            Err(AuthError::InvalidSession)
        }
    }
}

pub struct AuthenticatedUser {
    pub user_id: i32,
    pub email_verified: bool,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization");
        
        if auth_header.is_none() {
            return err(ErrorUnauthorized("No authorization header"));
        }

        let auth_header = auth_header.unwrap().to_str().unwrap();
        if !auth_header.starts_with("Bearer ") {
            return err(ErrorUnauthorized("Invalid authorization header format"));
        }

        let token = &auth_header[7..];
        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::default(),
        ) {
            Ok(token_data) => {
                if token_data.claims.exp < Utc::now().timestamp() {
                    return err(ErrorUnauthorized("Token expired"));
                }
                if token_data.claims.token_type != TokenType::Access {
                    return err(ErrorUnauthorized("Invalid token type"));
                }
                ok(AuthenticatedUser {
                    user_id: token_data.claims.sub.parse::<i32>().unwrap(),
                    email_verified: false,
                })
            }
            Err(_) => err(ErrorUnauthorized("Invalid token")),
        }
    }
}

pub fn create_access_token(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(1))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
        iat: Utc::now().timestamp(),
        token_type: TokenType::Access,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

pub fn create_refresh_token(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(REFRESH_TOKEN_EXPIRY_DAYS))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
        iat: Utc::now().timestamp(),
        token_type: TokenType::Refresh,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

pub fn create_verification_token(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(VERIFICATION_TOKEN_EXPIRY_DAYS))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
        iat: Utc::now().timestamp(),
        token_type: TokenType::Verification,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

pub fn create_password_reset_token(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(PASSWORD_RESET_TOKEN_EXPIRY_HOURS))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
        iat: Utc::now().timestamp(),
        token_type: TokenType::PasswordReset,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    verify(password, hash).unwrap_or(false)
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn validate_password_strength(password: &str) -> Result<(), AuthError> {
    let password_regex = Regex::new(r"^(?=.*[A-Z])(?=.*[a-z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$")
        .map_err(|_| AuthError::PasswordValidation("Invalid regex pattern"))?;

    if !password_regex.is_match(password) {
        return Err(AuthError::PasswordValidation(
            "Password must be at least 8 characters long and contain at least one uppercase letter, one lowercase letter, one number, and one special character"
        ));
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Password hashing failed: {0}")]
    PasswordHashing(#[from] bcrypt::BcryptError),
    #[error("Validation failed: {0}")]
    Validation(#[from] validator::ValidationErrors),
    #[error("Invalid refresh token")]
    InvalidRefreshToken,
    #[error("Token generation failed: {0}")]
    TokenGeneration(#[from] jsonwebtoken::errors::Error),
    #[error("Email verification failed")]
    EmailVerification,
    #[error("Password reset failed")]
    PasswordReset,
    #[error("Email sending failed: {0}")]
    EmailError(#[from] crate::services::email::EmailError),
    #[error("Password validation failed: {0}")]
    PasswordValidation(&'static str),
    #[error("Email not verified")]
    EmailNotVerified,
    #[error("Email already verified")]
    EmailAlreadyVerified,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid session")]
    InvalidSession,
    #[error("Current password is incorrect")]
    IncorrectPassword,
    #[error("Session expired")]
    SessionExpired,
    #[error("Session revoked")]
    SessionRevoked,
    #[error("Session limit exceeded")]
    SessionLimitExceeded,
    #[error("No sessions to cleanup")]
    NoSessionsToCleanup,
}

impl From<AuthError> for actix_web::Error {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::InvalidCredentials => ErrorUnauthorized("Invalid credentials"),
            AuthError::PasswordHashing(_) => actix_web::error::ErrorInternalServerError("Password hashing failed"),
            AuthError::Validation(errors) => {
                let error_messages: Vec<String> = errors
                    .field_errors()
                    .into_iter()
                    .flat_map(|(field, errors)| {
                        errors.iter().map(move |e| {
                            format!("{}: {}", field, e.message.as_ref().unwrap_or(&"Invalid".to_string()))
                        })
                    })
                    .collect();
                actix_web::error::ErrorBadRequest(error_messages.join(", "))
            }
            AuthError::InvalidRefreshToken => ErrorUnauthorized("Invalid refresh token"),
            AuthError::TokenGeneration(_) => actix_web::error::ErrorInternalServerError("Token generation failed"),
            AuthError::EmailVerification => ErrorUnauthorized("Email verification failed"),
            AuthError::PasswordReset => ErrorUnauthorized("Password reset failed"),
            AuthError::EmailError(e) => actix_web::error::ErrorInternalServerError(e.to_string()),
            AuthError::PasswordValidation(msg) => actix_web::error::ErrorBadRequest(msg.to_string()),
            AuthError::EmailNotVerified => ErrorUnauthorized("Email not verified"),
            AuthError::EmailAlreadyVerified => ErrorUnauthorized("Email already verified"),
            AuthError::UserNotFound => ErrorUnauthorized("User not found"),
            AuthError::InvalidSession => ErrorUnauthorized("Invalid session"),
            AuthError::IncorrectPassword => ErrorUnauthorized("Current password is incorrect"),
            AuthError::SessionExpired => ErrorUnauthorized("Session expired"),
            AuthError::SessionRevoked => ErrorUnauthorized("Session revoked"),
            AuthError::SessionLimitExceeded => ErrorUnauthorized("Maximum number of sessions reached"),
            AuthError::NoSessionsToCleanup => ErrorUnauthorized("No sessions available to cleanup"),
        }
    }
}

pub struct AuthService {
    db: Database,
}

impl AuthService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn register(&self, user_data: UserCreate) -> Result<User, ApiError> {
        // Check if user already exists
        if self.db.get_user_by_email(&user_data.email).await?.is_some() {
            return Err(ApiError::new(400, "Email already registered"));
        }

        // Hash password
        let password_hash = hash_password(&user_data.password)?;

        // Create user
        let user = self.db.create_user(UserCreate {
            email: user_data.email,
            username: user_data.username,
            password: password_hash,
        }).await?;

        Ok(user)
    }

    pub async fn login(&self, login_data: LoginRequest) -> Result<LoginResponse, ApiError> {
        // Get user by email
        let user = self.db.get_user_by_email(&login_data.email)
            .await?
            .ok_or_else(|| ApiError::new(401, "Invalid credentials"))?;

        // Verify password
        if !verify_password(&login_data.password, &user.password_hash)? {
            return Err(ApiError::new(401, "Invalid credentials"));
        }

        // Generate tokens
        let access_token = self.generate_token(&user.id, TokenType::Access)?;
        let refresh_token = self.generate_token(&user.id, TokenType::Refresh)?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            user,
        })
    }

    pub async fn refresh_token(&self, refresh_data: RefreshRequest) -> Result<LoginResponse, ApiError> {
        // Verify refresh token
        let claims = self.verify_token(&refresh_data.refresh_token)?;
        
        if claims.token_type != TokenType::Refresh {
            return Err(ApiError::new(401, "Invalid token type"));
        }

        // Get user
        let user = self.db.get_user_by_id(&claims.sub)
            .await?
            .ok_or_else(|| ApiError::new(401, "User not found"))?;

        // Generate new tokens
        let access_token = self.generate_token(&user.id, TokenType::Access)?;
        let refresh_token = self.generate_token(&user.id, TokenType::Refresh)?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            user,
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, ApiError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    fn generate_token(&self, user_id: &str, token_type: TokenType) -> Result<String, ApiError> {
        let now = Utc::now().timestamp();
        let expiry = match token_type {
            TokenType::Access => ACCESS_TOKEN_EXPIRY,
            TokenType::Refresh => REFRESH_TOKEN_EXPIRY,
        };

        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + expiry,
            iat: now,
            token_type,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET),
        ).map_err(|e| ApiError::new(500, format!("Token generation error: {}", e)))
    }
} 