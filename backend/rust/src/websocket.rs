use actix::{Actor, AsyncContext, Context, Handler, Message};
use actix_web_actors::ws;
use chrono::Utc;
use serde_json::json;
use std::time::{Duration, Instant};
use std::sync::Mutex;
use std::collections::HashMap;
use crate::models::{WsMessage, MessageDTO};
use crate::AppState;
use crate::{
    models::message::Message,
    services::auth::AuthService,
    utils::error::ApiError,
};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct Session {
    id: String,
    user_id: String,
    hb: Instant,
    auth: AuthService,
}

impl Session {
    pub fn new(user_id: String, auth: AuthService) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            hb: Instant::now(),
            auth,
        }
    }

    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }

    fn handle_message(&self, msg: String) -> Result<(), ApiError> {
        let msg: serde_json::Value = serde_json::from_str(&msg)?;
        
        match msg["type"].as_str() {
            Some("message") => {
                let content = msg["content"].as_str()
                    .ok_or_else(|| ApiError::new(400, "Missing message content"))?;
                
                // Create message in database
                let message = Message {
                    id: uuid::Uuid::new_v4().to_string(),
                    chat_id: msg["chat_id"].as_str()
                        .ok_or_else(|| ApiError::new(400, "Missing chat_id"))?
                        .to_string(),
                    sender_id: self.user_id.clone(),
                    content: content.to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                // TODO: Broadcast message to chat participants
                Ok(())
            }
            Some("status") => {
                // Handle status updates (online, typing, etc.)
                Ok(())
            }
            _ => Err(ApiError::new(400, "Invalid message type")),
        }
    }
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl Handler<ws::Message> for Session {
    type Result = ();

    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                if let Err(e) = self.handle_message(text.to_string()) {
                    ctx.text(json!({
                        "type": "error",
                        "message": e.to_string()
                    }).to_string());
                }
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage {
    pub chat_id: String,
    pub message: Message,
}

impl Handler<BroadcastMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        if msg.message.chat_id == msg.chat_id {
            ctx.text(json!({
                "type": "message",
                "data": msg.message
            }).to_string());
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateUserStatus {
    pub user_id: String,
    pub status: String,
}

impl Handler<UpdateUserStatus> for Session {
    type Result = ();

    fn handle(&mut self, msg: UpdateUserStatus, ctx: &mut Self::Context) {
        if msg.user_id == self.user_id {
            ctx.text(json!({
                "type": "status",
                "data": {
                    "user_id": msg.user_id,
                    "status": msg.status
                }
            }).to_string());
        }
    }
} 