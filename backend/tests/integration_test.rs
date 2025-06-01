use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use messaging_app::{
    handlers::{
        messages::{send_message, get_message, update_message, delete_message},
        media::{upload_media, delete_media},
    },
    models::message::CreateMessageRequest,
};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup_test_db() -> PgPoolOptions {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/messaging_app_test".to_string());
    
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
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
        .with_state(pool)
}

#[tokio::test]
async fn test_send_and_get_message() {
    let app = setup_test_app().await;
    
    // Create a test message
    let message = CreateMessageRequest {
        content: "Test message".to_string(),
        sender_id: Uuid::new_v4(),
        receiver_id: Uuid::new_v4(),
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
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
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
    let app = setup_test_app().await;
    
    // Create a test message
    let message = CreateMessageRequest {
        content: "Original message".to_string(),
        sender_id: Uuid::new_v4(),
        receiver_id: Uuid::new_v4(),
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

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
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
    let app = setup_test_app().await;
    
    // Create a test message
    let message = CreateMessageRequest {
        content: "Message to delete".to_string(),
        sender_id: Uuid::new_v4(),
        receiver_id: Uuid::new_v4(),
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

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
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