use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use messaging_app_backend::handlers::{
    media::{delete_media, upload_media},
    messages::{delete_message, send_message, update_message},
};
use messaging_app_backend::models::message::CreateMessageRequest;
use messaging_app_backend::models::{CreateChatRequest, Chat};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use axum::ServiceExt;
use uuid::Uuid;
use axum::body::to_bytes;
use messaging_app_backend::{create_app, AppState};
use std::sync::Arc;
use tokio::sync::broadcast;
use messaging_app_backend::services::ws::WsService;
use redis::Client as RedisClient;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/messaging_app_test".to_string()
    });

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn setup_test_chat(pool: &PgPool) -> Chat {
    let chat = sqlx::query_as!(Chat,
        "INSERT INTO chats (name, is_group, created_at, updated_at) VALUES ($1, $2, NOW(), NOW()) RETURNING id, name, is_group, created_at, updated_at",
        Some("Test Chat".to_string()),
        false
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test chat");
    chat
}

async fn setup_test_app() -> Router<Arc<AppState>> {
    let pool = setup_test_db().await;
    let redis = RedisClient::open("redis://127.0.0.1/").unwrap();
    let (ws_tx, _) = broadcast::channel(100);
    let ws_manager = Arc::new(messaging_app_backend::websocket::WebSocketManager::new());
    let state = Arc::new(AppState {
        pool: pool.clone(),
        redis: redis.clone(),
        ws_tx,
        ws_manager,
    });
    let app = create_app(pool.clone(), redis.clone());
    app
}

#[tokio::test]
async fn test_send_and_get_message() {
    let pool = setup_test_db().await;
    let chat = setup_test_chat(&pool).await;
    let app = setup_test_app().await;

    // Create a test message
    let message = CreateMessageRequest {
        chat_id: chat.id,
        content: "Test message".to_string(),
        media_url: None,
        media_type: None,
        reply_to_id: None,
    };

    // Send the message
    let response = app.clone().oneshot(
        Request::builder()
            .uri("/api/messages")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&message).unwrap()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Get the message ID from the response
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let message_id = response["id"].as_str().unwrap();

    // Get the message
    let response = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/messages/{}", message_id))
            .method("GET")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_update_message() {
    let pool = setup_test_db().await;
    let chat = setup_test_chat(&pool).await;
    let app = setup_test_app().await;

    // Create a test message
    let message = CreateMessageRequest {
        chat_id: chat.id,
        content: "Original message".to_string(),
        media_url: None,
        media_type: None,
        reply_to_id: None,
    };

    // Send the message
    let response = app.clone().oneshot(
        Request::builder()
            .uri("/api/messages")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&message).unwrap()))
            .unwrap(),
    ).await.unwrap();

    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let message_id = response["id"].as_str().unwrap();

    // Update the message
    let update = serde_json::json!({
        "content": "Updated message"
    });

    let response = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/messages/{}", message_id))
            .method("PUT")
            .header("content-type", "application/json")
            .body(Body::from(update.to_string()))
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_delete_message() {
    let pool = setup_test_db().await;
    let chat = setup_test_chat(&pool).await;
    let app = setup_test_app().await;

    // Create a test message
    let message = CreateMessageRequest {
        chat_id: chat.id,
        content: "Message to delete".to_string(),
        media_url: None,
        media_type: None,
        reply_to_id: None,
    };

    // Send the message
    let response = app.clone().oneshot(
        Request::builder()
            .uri("/api/messages")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&message).unwrap()))
            .unwrap(),
    ).await.unwrap();

    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let message_id = response["id"].as_str().unwrap();

    // Delete the message
    let response = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/messages/{}", message_id))
            .method("DELETE")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_rate_limiting() {
    let app = setup_test_app().await;

    // Make multiple requests in quick succession
    for _ in 0..101 {
        let response = app.clone().oneshot(
            Request::builder()
                .uri("/api/messages")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return;
        }
    }

    panic!("Rate limiting not working");
}
