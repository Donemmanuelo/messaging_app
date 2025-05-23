# Messaging Application Documentation

## Architecture Overview

The application is built using a microservices architecture with the following components:

1. **Frontend (React)**
   - User interface built with React
   - Real-time updates using WebSocket
   - State management with React Context
   - Responsive design for all devices

2. **Backend (Rust)**
   - RESTful API using Actix-web (located in `backend/rust/`)
   - WebSocket server for real-time communication
   - Authentication and authorization
   - Rate limiting and security features
   The repository also contains a `java-backend` directory with a Java-based backend implementation, which is available for future development or alternative use-cases but is not part of the default active service stack.

3. **Database (PostgreSQL)**
   - User data storage
   - Message persistence
   - Chat room management

4. **Cache (Redis)**
   - Session management
   - Rate limiting
   - Real-time presence tracking

## Data Flow

### Authentication Flow

1. **Registration**
   ```
   User -> Frontend -> Backend -> Database
   ```
   - User submits registration form
   - Backend validates input
   - Password is hashed
   - User is created in database
   - Verification email is sent
   - JWT tokens are generated

2. **Login**
   ```
   User -> Frontend -> Backend -> Database
   ```
   - User submits credentials
   - Backend verifies password hash
   - JWT tokens are generated
   - Session is created in Redis

3. **Token Refresh**
   ```
   Frontend -> Backend -> Redis
   ```
   - Frontend sends refresh token
   - Backend validates token
   - New access token is generated
   - Session is updated in Redis

### Messaging Flow

1. **Sending Messages**
   ```
   User -> Frontend -> WebSocket -> Backend -> Database -> Redis -> Other Users
   ```
   - User types and sends message
   - Message is sent via WebSocket
   - Backend validates and stores message
   - Message is broadcast to other users
   - Message is persisted in database

2. **Receiving Messages**
   ```
   WebSocket -> Frontend -> User
   ```
   - WebSocket connection receives message
   - Frontend updates UI
   - Message is displayed in chat

3. **Status Updates**
   ```
   User -> Frontend -> WebSocket -> Backend -> Redis -> Other Users
   ```
   - User status changes
   - Update is sent via WebSocket
   - Backend updates Redis
   - Other users are notified

## Security Implementation

### Authentication Security

1. **Password Security**
   - Passwords are hashed using bcrypt
   - Salt is automatically generated
   - Minimum password requirements enforced

2. **Token Security**
   - JWT tokens with expiration
   - Refresh token rotation
   - Token blacklisting for logout

3. **Session Security**
   - Session fingerprinting
   - IP address tracking
   - Device information tracking
   - Suspicious activity detection

### API Security

1. **Rate Limiting**
   - Token bucket algorithm
   - IP-based limiting
   - User-based limiting
   - Custom error responses

2. **Input Validation**
   - Request body validation
   - Query parameter validation
   - Path parameter validation
   - Custom validation rules

3. **CORS Protection**
   - Origin validation
   - Method validation
   - Header validation
   - Credential handling

## Real-time Features

### WebSocket Implementation

1. **Connection Management**
   - Heartbeat mechanism
   - Connection state tracking
   - Automatic reconnection
   - Error handling

2. **Message Types**
   ```typescript
   interface WebSocketMessage {
     type: 'message' | 'status' | 'typing' | 'read';
     data: any;
   }
   ```

3. **Event Handling**
   - Message events
   - Status events
   - Typing indicators
   - Read receipts

### Presence System

1. **User Status**
   - Online/Offline tracking
   - Last seen timestamp
   - Status messages
   - Custom status support

2. **Chat Room Presence**
   - User count
   - Active users
   - Typing indicators
   - Read status

## Database Schema

### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    is_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

### Chats Table
```sql
CREATE TABLE chats (
    id UUID PRIMARY KEY,
    name VARCHAR(255),
    is_group BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

### Messages Table
```sql
CREATE TABLE messages (
    id UUID PRIMARY KEY,
    chat_id UUID REFERENCES chats(id),
    sender_id UUID REFERENCES users(id),
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

## Error Handling

### HTTP Error Responses

1. **Authentication Errors**
   ```json
   {
     "error": "Unauthorized",
     "message": "Invalid credentials"
   }
   ```

2. **Validation Errors**
   ```json
   {
     "error": "ValidationError",
     "fields": {
       "email": ["Invalid email format"],
       "password": ["Password too short"]
     }
   }
   ```

3. **Rate Limit Errors**
   ```json
   {
     "error": "RateLimitExceeded",
     "message": "Too many requests",
     "retry_after": 60
   }
   ```

### WebSocket Error Handling

1. **Connection Errors**
   - Automatic reconnection
   - Error logging
   - User notification

2. **Message Errors**
   - Error responses
   - Message retry
   - State synchronization

## Testing

### Unit Tests

1. **Backend Tests**
   - Authentication tests
   - Message handling tests
   - Database operation tests
   - WebSocket tests

2. **Frontend Tests**
   - Component tests
   - Integration tests
   - WebSocket tests
   - UI interaction tests

### Integration Tests

1. **API Tests**
   - Endpoint testing
   - Authentication flow
   - Error handling
   - Rate limiting

2. **WebSocket Tests**
   - Connection handling
   - Message broadcasting
   - Presence system
   - Error recovery

## Deployment

### Docker Deployment

1. **Building Images**
   ```bash
   docker-compose build
   ```

2. **Running Containers**
   ```bash
   docker-compose up -d
   ```

3. **Scaling Services**
   ```bash
   docker-compose up -d --scale backend=3
   ```

### Environment Configuration

1. **Production Settings**
   - Secure JWT secrets
   - Production database
   - SSL/TLS configuration
   - Logging setup

2. **Monitoring**
   - Health checks
   - Performance metrics
   - Error tracking
   - Usage analytics

## Maintenance

### Database Maintenance

1. **Backups**
   - Automated backups
   - Point-in-time recovery
   - Backup verification

2. **Optimization**
   - Index maintenance
   - Query optimization
   - Storage optimization

### Application Maintenance

1. **Updates**
   - Dependency updates
   - Security patches
   - Feature updates

2. **Monitoring**
   - Error tracking
   - Performance monitoring
   - Usage analytics
   - Security monitoring 