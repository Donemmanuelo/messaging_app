use actix_web::{web, HttpResponse, Responder};
use crate::database::db_client::DbClient;
use serde::{Deserialize, Serialize};
use log::{info, error};
use crate::notifications::fcm::FcmService;
use crate::notifications::apns::ApnsService;

#[derive(Deserialize)]
pub struct CreateMessageRequest {
    from_user_id: i32,
    to_user_id: i32,
    content: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    id: i32,
    from_user_id: i32,
    to_user_id: i32,
    content: String,
    timestamp: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/messages").route(web::post().to(create_message)))
       .service(web::resource("/messages/{chat_id}").route(web::get().to(get_messages_by_chat)));
}

pub async fn create_message(
    req: web::Json<CreateMessageRequest>,
    db: web::Data<DbClient>,
    fcm_service: web::Data<FcmService>,
    apns_service: web::Data<ApnsService>,
) -> impl Responder {
    let message = crate::database::models::message::Message {
        id: None,
        from_user_id: req.from_user_id,
        to_user_id: req.to_user_id,
        content: req.content.clone(),
        timestamp: chrono::Utc::now().naive_utc(),
    };

    match crate::database::operations::create_message(message, &db).await {
        Ok(new_message) => {
            info!("Message created successfully: {}", new_message.content);

            // Send FCM notification
            let fcm_token = "user_fcm_token"; // Replace with actual FCM token
            if let Err(e) = fcm_service.send_notification(fcm_token, &new_message.content).await {
                error!("Failed to send FCM notification: {}", e);
            }

            // Send APNs notification
            let apns_token = "user_apns_token"; // Replace with actual APNs token
            if let Err(e) = apns_service.send_notification(apns_token, &new_message.content).await {
                error!("Failed to send APNs notification: {}", e);
            }

            HttpResponse::Ok().json(MessageResponse {
                id: new_message.id.unwrap(),
                from_user_id: new_message.from_user_id,
                to_user_id: new_message.to_user_id,
                content: new_message.content,
                timestamp: new_message.timestamp.to_string(),
            })
        }
        Err(e) => {
            error!("Failed to create message: {}", e);
            HttpResponse::InternalServerError().json("Failed to create message")
        }
    }
}

pub async fn get_messages_by_chat(
    web::Path(chat_id): web::Path<i32>,
    db: web::Data<DbClient>,
) -> impl Responder {
    match crate::database::operations::get_messages_by_chat(chat_id, &db).await {
        Ok(messages) => {
            info!("Fetched messages for chat ID: {}", chat_id);
            let messages_response: Vec<MessageResponse> = messages.into_iter().map(|msg| MessageResponse {
                id: msg.id.unwrap(),
                from_user_id: msg.from_user_id,
                to_user_id: msg.to_user_id,
                content: msg.content,
                timestamp: msg.timestamp.to_string(),
            }).collect();
            HttpResponse::Ok().json(messages_response)
        }
        Err(e) => {
            error!("Failed to fetch messages: {}", e);
            HttpResponse::InternalServerError().json("Failed to fetch messages")
        }
    }
}