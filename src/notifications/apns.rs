use apns2::client::Client;
use apns2::token::Token;
use apns2::notification::Notification;
use log::{info, error};

pub struct ApnsService {
    client: Client,
}

impl ApnsService {
    pub fn new(cert_path: &str, key_path: &str, key_id: &str, team_id: &str) -> Self {
        info!("Initializing APNs service");
        let token = Token::from_p8_file(cert_path, key_path, key_id, team_id).expect("Failed to read APNs key files");
        let client = Client::new(token, "production").expect("Failed to create APNs client");
        ApnsService { client }
    }

    pub async fn send_notification(&self, token: &str, message: &str) -> Result<(), apns2::Error> {
        info!("Sending APNs notification to token: {}", token);
        let notification = Notification::new(token, message);
        match self.client.send(notification).await {
            Ok(response) => {
                info!("APNs notification sent successfully: {:?}", response);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send APNs notification: {}", e);
                Err(e)
            }
        }
    }
}