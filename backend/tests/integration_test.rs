use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use crate::handlers::{
    media::{delete_media, upload_media},
    messages::{delete_message, get_message, send_message, update_message},
};
use crate::models::message::CreateMessageRequest;
use crate::models::{CreateChatRequest, Chat};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use hyper::body::to_bytes;

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

async fn setup_test_app() -> Router {
    let pool = setup_test_db().await;
    Router::new()
        .route("/api/messages", axum::routing::post(send_message))
        .route("/api/messages/:id", axum::routing::get(get_message))
        .route("/api/messages/:id", axum::routing::put(update_message))
        .route("/api/messages/:id", axum::routing::delete(delete_message))
        .route("/api/media", axum::routing::post(upload_media))
        .route("/api/media/:id", axum::routing::delete(delete_media))
        .with_state(pool.clone())
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
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/messages")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&message).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Get the message ID from the response
    let body = to_bytes(response.into_body()).await.unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let message_id = response["id"].as_str().unwrap();

    // Get the message
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/messages/{}", message_id))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

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
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/messages")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&message).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body()).await.unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let message_id = response["id"].as_str().unwrap();

    // Update the message
    let update = serde_json::json!({
        "content": "Updated message"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/messages/{}", message_id))
                .method("PUT")
                .header("content-type", "application/json")
                .body(Body::from(update.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

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
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/messages")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&message).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body()).await.unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let message_id = response["id"].as_str().unwrap();

    // Delete the message
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/messages/{}", message_id))
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_rate_limiting() {
    let app = setup_test_app().await;

    // Make multiple requests in quick succession
    for _ in 0..101 {
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/messages")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return;
        }
    }

    panic!("Rate limiting not working");
}
