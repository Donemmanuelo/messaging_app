use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("Failed to send email: {0}")]
    SendError(#[from] lettre::transport::smtp::Error),
    #[error("Failed to create email: {0}")]
    CreateError(#[from] lettre::error::Error),
}

pub struct EmailService {
    smtp_transport: SmtpTransport,
    from_address: String,
}

impl EmailService {
    pub fn new(
        smtp_host: &str,
        smtp_port: u16,
        smtp_username: &str,
        smtp_password: &str,
        from_address: &str,
    ) -> Self {
        let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());
        let smtp_transport = SmtpTransport::relay(smtp_host)
            .unwrap()
            .port(smtp_port)
            .credentials(creds)
            .build();

        Self {
            smtp_transport,
            from_address: from_address.to_string(),
        }
    }

    pub fn send_verification_email(&self, to: &str, token: &str) -> Result<(), EmailError> {
        let verification_url = format!("http://localhost:3000/verify-email?token={}", token);
        let email = Message::builder()
            .from(self.from_address.parse()?)
            .to(to.parse()?)
            .subject("Verify your email address")
            .header(ContentType::TEXT_HTML)
            .body(format!(
                r#"
                <h1>Welcome to Messaging App!</h1>
                <p>Please click the link below to verify your email address:</p>
                <a href="{}">Verify Email</a>
                <p>If you did not create an account, please ignore this email.</p>
                "#,
                verification_url
            ))?;

        self.smtp_transport.send(&email)?;
        Ok(())
    }

    pub fn send_password_reset_email(&self, to: &str, token: &str) -> Result<(), EmailError> {
        let reset_url = format!("http://localhost:3000/reset-password?token={}", token);
        let email = Message::builder()
            .from(self.from_address.parse()?)
            .to(to.parse()?)
            .subject("Reset your password")
            .header(ContentType::TEXT_HTML)
            .body(format!(
                r#"
                <h1>Password Reset Request</h1>
                <p>You have requested to reset your password. Click the link below to proceed:</p>
                <a href="{}">Reset Password</a>
                <p>If you did not request a password reset, please ignore this email.</p>
                <p>This link will expire in 1 hour.</p>
                "#,
                reset_url
            ))?;

        self.smtp_transport.send(&email)?;
        Ok(())
    }
} 