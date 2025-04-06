use warp::ws::WebSocket;
use futures::sink::SinkExt;
use log::{info, error};

pub async fn handle_client(ws: WebSocket, user_id: String) {
    info!("Handling WebSocket client for user: {}", user_id);
    let (mut tx, mut rx) = ws.split();

    while let Some(result) = rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("WebSocket error for user {}: {}", user_id, e);
                break;
            }
        };

        if let Ok(text) = msg.to_str() {
            info!("Received message from user {}: {}", user_id, text);
            // Example: Echo the message back to the client
            if let Err(e) = tx.send(Message::text(text)).await {
                error!("Failed to send message to user {}: {}", user_id, e);
            }
        }
    }

    info!("WebSocket connection closed for user: {}", user_id);
}