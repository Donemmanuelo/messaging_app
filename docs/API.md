# API Documentation

## Overview
The messaging application provides a comprehensive REST API and WebSocket interface for real-time communication. This documentation details all available endpoints, authentication mechanisms, and data formats.

## Authentication

### JWT Authentication
All API requests require a valid JWT token in the Authorization header:
```http
Authorization: Bearer <token>
```

#### Token Generation
```http
POST /api/auth/login
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "secure_password"
}
```

Response:
```json
{
    "access_token": "eyJhbGciOiJIUzI1NiIs...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
    "expires_in": 3600,
    "token_type": "Bearer"
}
```

#### Token Refresh
```http
POST /api/auth/refresh
Content-Type: application/json

{
    "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

### Rate Limiting
- Authentication endpoints: 5 requests per minute
- API endpoints: 100 requests per minute
- WebSocket connections: 10 connections per minute

Rate limit headers:
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 99
X-RateLimit-Reset: 1619123456
```

## Endpoints

### Users

#### Register User
```http
POST /api/users
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "secure_password",
    "username": "username",
    "display_name": "Display Name"
}
```

Response:
```json
{
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "email": "user@example.com",
    "username": "username",
    "display_name": "Display Name",
    "created_at": "2024-03-20T12:00:00Z"
}
```

#### Get User Profile
```http
GET /api/users/{user_id}
Authorization: Bearer <token>
```

Response:
```json
{
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "username": "username",
    "display_name": "Display Name",
    "status": "online",
    "last_seen": "2024-03-20T12:00:00Z",
    "profile": {
        "bio": "User bio",
        "avatar_url": "https://example.com/avatars/user.jpg",
        "location": "City, Country"
    }
}
```

#### Update User Profile
```http
PATCH /api/users/{user_id}
Authorization: Bearer <token>
Content-Type: application/json

{
    "display_name": "New Display Name",
    "bio": "Updated bio",
    "location": "New City, Country"
}
```

### Messages

#### Send Message
```http
POST /api/messages
Authorization: Bearer <token>
Content-Type: application/json

{
    "receiver_id": "123e4567-e89b-12d3-a456-426614174000",
    "content": "Hello, world!",
    "type": "text",
    "metadata": {
        "reply_to": "optional_message_id",
        "mentions": ["user_id1", "user_id2"]
    }
}
```

Response:
```json
{
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "sender_id": "sender_id",
    "receiver_id": "receiver_id",
    "content": "Hello, world!",
    "type": "text",
    "status": "sent",
    "created_at": "2024-03-20T12:00:00Z",
    "metadata": {
        "reply_to": "optional_message_id",
        "mentions": ["user_id1", "user_id2"]
    }
}
```

#### Get Messages
```http
GET /api/messages?conversation_id={conversation_id}&limit=50&before={message_id}
Authorization: Bearer <token>
```

Response:
```json
{
    "messages": [
        {
            "id": "message_id",
            "sender_id": "sender_id",
            "content": "Message content",
            "type": "text",
            "status": "delivered",
            "created_at": "2024-03-20T12:00:00Z"
        }
    ],
    "has_more": true,
    "next_cursor": "next_message_id"
}
```

### Groups

#### Create Group
```http
POST /api/groups
Authorization: Bearer <token>
Content-Type: application/json

{
    "name": "Group Name",
    "description": "Group description",
    "members": ["user_id1", "user_id2"],
    "settings": {
        "is_private": false,
        "allow_invites": true,
        "message_retention_days": 30
    }
}
```

Response:
```json
{
    "id": "group_id",
    "name": "Group Name",
    "description": "Group description",
    "owner_id": "owner_id",
    "created_at": "2024-03-20T12:00:00Z",
    "members": [
        {
            "user_id": "user_id1",
            "role": "member",
            "joined_at": "2024-03-20T12:00:00Z"
        }
    ],
    "settings": {
        "is_private": false,
        "allow_invites": true,
        "message_retention_days": 30
    }
}
```

## WebSocket API

### Connection
```javascript
const ws = new WebSocket('wss://api.example.com/ws?token=<jwt_token>');
```

### Events

#### Message Events
```json
{
    "type": "message",
    "data": {
        "id": "message_id",
        "sender_id": "sender_id",
        "content": "Message content",
        "type": "text",
        "created_at": "2024-03-20T12:00:00Z"
    }
}
```

#### Typing Events
```json
{
    "type": "typing",
    "data": {
        "user_id": "user_id",
        "conversation_id": "conversation_id",
        "is_typing": true
    }
}
```

#### Presence Events
```json
{
    "type": "presence",
    "data": {
        "user_id": "user_id",
        "status": "online",
        "last_seen": "2024-03-20T12:00:00Z"
    }
}
```

### Error Responses

#### Authentication Error
```json
{
    "error": {
        "code": "unauthorized",
        "message": "Invalid or expired token",
        "details": {
            "token_expired": true
        }
    }
}
```

#### Validation Error
```json
{
    "error": {
        "code": "validation_error",
        "message": "Invalid request data",
        "details": {
            "field": "email",
            "reason": "Invalid email format"
        }
    }
}
```

#### Rate Limit Error
```json
{
    "error": {
        "code": "rate_limit_exceeded",
        "message": "Too many requests",
        "details": {
            "retry_after": 60
        }
    }
}
```

## Pagination

### Query Parameters
- `limit`: Number of items per page (default: 50, max: 100)
- `before`: Cursor for pagination (ID of the last item from previous page)
- `after`: Cursor for pagination (ID of the first item from previous page)

### Response Headers
```http
X-Pagination-Total: 1000
X-Pagination-Pages: 20
X-Pagination-Current-Page: 1
X-Pagination-Per-Page: 50
```

## Filtering

### Query Parameters
- `status`: Filter by status (e.g., `status=active`)
- `type`: Filter by type (e.g., `type=text`)
- `from`: Filter by date range start
- `to`: Filter by date range end

Example:
```http
GET /api/messages?status=delivered&type=text&from=2024-03-01&to=2024-03-20
```

## Versioning

The API is versioned using the URL path:
```http
https://api.example.com/v1/messages
```

Current versions:
- v1: Current stable version
- v2: Beta version (requires special access)

## Webhooks

### Configuration
```http
POST /api/webhooks
Authorization: Bearer <token>
Content-Type: application/json

{
    "url": "https://your-server.com/webhook",
    "events": ["message.created", "user.updated"],
    "secret": "webhook_secret"
}
```

### Event Payload
```json
{
    "event": "message.created",
    "timestamp": "2024-03-20T12:00:00Z",
    "data": {
        "message_id": "message_id",
        "sender_id": "sender_id",
        "content": "Message content"
    },
    "signature": "sha256=..."
}
```

## SDKs

Official SDKs are available for:
- [JavaScript/TypeScript](https://github.com/your-org/messaging-sdk-js)
- [Python](https://github.com/your-org/messaging-sdk-python)
- [Java](https://github.com/your-org/messaging-sdk-java)
- [Go](https://github.com/your-org/messaging-sdk-go)

## Best Practices

1. **Error Handling**
   - Always check response status codes
   - Implement exponential backoff for retries
   - Handle rate limiting gracefully

2. **Security**
   - Store tokens securely
   - Rotate refresh tokens
   - Validate webhook signatures

3. **Performance**
   - Use pagination for large datasets
   - Implement caching where appropriate
   - Monitor rate limits

4. **WebSocket**
   - Implement heartbeat mechanism
   - Handle reconnection gracefully
   - Process messages in order 