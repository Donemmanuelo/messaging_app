use fcm::FcmClient;
use log::{info, error};

pub struct FcmService {
    client: FcmClient,
}

impl FcmService {
    pub fn new(api_key: &str) -> Self {
        info!("Initializing FCM service");
        let client = FcmClient::new(api_key);
        FcmService { client }
    }

    pub async fn send_notification(&self, token: &str, message: &str) -> Result<(), fcm::Error> {
        info!("Sending FCM notification to token: {}", token);
        let message = fcm::Message {
            token: Some(token.to_string()),
            notification: Some(fcm::Notification {
                title: "New Message".to_string(),
                body: message.to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        match self.client.send(message).await {
            Ok(response) => {
                info!("FCM notification sent successfully: {:?}", response);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send FCM notification: {}", e);
                Err(e)
            }
        }
    }
}