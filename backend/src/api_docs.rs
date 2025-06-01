use utoipa::OpenApi;
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::messages::send_message,
        crate::handlers::messages::get_message,
        crate::handlers::messages::update_message,
        crate::handlers::messages::delete_message,
        crate::handlers::media::upload_media,
        crate::handlers::media::delete_media
    ),
    components(
        schemas(Message, MediaUpload, ErrorResponse)
    ),
    tags(
        (name = "messages", description = "Message management endpoints"),
        (name = "media", description = "Media management endpoints")
    )
)]
pub struct ApiDoc;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Message {
    /// Unique identifier for the message
    pub id: i32,
    /// Content of the message
    pub content: String,
    /// ID of the sender
    pub sender_id: i32,
    /// ID of the receiver
    pub receiver_id: i32,
    /// Timestamp when the message was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when the message was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MediaUpload {
    /// Type of media (image, video, etc.)
    pub media_type: String,
    /// Size of the media file in bytes
    pub size: i64,
    /// MIME type of the media
    pub mime_type: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// HTTP status code
    pub status: u16,
}

pub fn get_api_docs() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
} 