use web_push::{WebPushMessageBuilder, VapidSignatureBuilder, ContentEncoding, SubscriptionInfo, SubscriptionKeys};
use serde::{Deserialize, Serialize};
use base64::decode;
use std::env;
use web_push::IsahcWebPushClient;
use web_push::WebPushClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub endpoint: String,
    pub keys: PushSubscriptionKeys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

pub struct VapidConfig {
    pub subject: String,
    pub public_key: String,
    pub private_key: String,
}

impl VapidConfig {
    pub fn from_env() -> Self {
        Self {
            subject: env::var("VAPID_SUBJECT").unwrap_or_else(|_| "mailto:admin@example.com".to_string()),
            public_key: env::var("VAPID_PUBLIC_KEY").expect("VAPID_PUBLIC_KEY missing"),
            private_key: env::var("VAPID_PRIVATE_KEY").expect("VAPID_PRIVATE_KEY missing"),
        }
    }
}

pub async fn send_web_push(
    subscription: &PushSubscription,
    payload: &str,
    vapid: &VapidConfig,
) -> Result<(), web_push::WebPushError> {
    let subscription_info = SubscriptionInfo {
        endpoint: subscription.endpoint.clone(),
        keys: SubscriptionKeys {
            p256dh: decode(&subscription.keys.p256dh)
                .map_err(|e| web_push::WebPushError::Other(format!("Invalid p256dh: {e}")))?
                .try_into()
                .map_err(|e| web_push::WebPushError::Other(format!("Invalid p256dh length: {e}")))?,
            auth: decode(&subscription.keys.auth)
                .map_err(|e| web_push::WebPushError::Other(format!("Invalid auth: {e}")))?
                .try_into()
                .map_err(|e| web_push::WebPushError::Other(format!("Invalid auth length: {e}")))?,
        },
    };

    let mut builder = WebPushMessageBuilder::new(&subscription_info);
    builder.set_payload(ContentEncoding::AesGcm, payload.as_bytes());

    let mut vapid_builder = VapidSignatureBuilder::from_pem(
        vapid.private_key.as_bytes(),
        &subscription_info,
    )?;
    vapid_builder.add_claim("sub", vapid.subject.clone());

    let vapid_signature = vapid_builder.build()?;
    builder.set_vapid_signature(vapid_signature);
    let client = IsahcWebPushClient::new()?;
    client.send(builder.build()?).await?;
    Ok(())
}