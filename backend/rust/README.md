# Messaging App Backend

A secure and scalable messaging application backend built with Rust and Actix.

## Features

- User authentication with JWT
- Email verification
- Password reset functionality
- Rate limiting
- Real-time messaging with WebSocket
- Session management
- PostgreSQL database integration

## Prerequisites

- Rust (latest stable version)
- Docker and Docker Compose
- PostgreSQL (if not using Docker)

## Setup

1. Clone the repository:
```bash
git clone <repository-url>
cd messaging_app/backend/rust
```

2. Create environment file:
```bash
cp .env.example .env
```
Edit the `.env` file with your configuration values.

3. Start PostgreSQL using Docker Compose:
```bash
docker-compose up -d
```

4. Build and run the application:
```bash
cargo build
cargo run
```

## API Endpoints

### Authentication
- `POST /api/auth/register` - Register a new user
- `POST /api/auth/login` - Login with credentials
- `POST /api/auth/refresh` - Refresh access token
- `POST /api/auth/verify-email` - Verify email address
- `POST /api/auth/request-password-reset` - Request password reset
- `POST /api/auth/reset-password` - Reset password with token

### Messages
- `GET /api/messages` - Get message history
- `POST /api/messages` - Send a new message
- `GET /api/messages/{id}` - Get a specific message

### WebSocket
- `WS /ws` - WebSocket endpoint for real-time messaging

## Security Features

- Password hashing with bcrypt
- JWT-based authentication
- Rate limiting
- Email verification
- Session management
- Input validation
- CORS protection

## Development

### Running Tests
```bash
cargo test
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details. 