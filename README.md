# Messaging Application

A real-time messaging application built with Rust and Actix Web.

## Features

- Real-time messaging using WebSocket
- User authentication with JWT
- Email verification
- Password reset functionality
- Rate limiting
- Session management
- PostgreSQL database

## Prerequisites

- Rust (latest stable version)
- Docker and Docker Compose
- PostgreSQL (if not using Docker)

## Setup

1. Clone the repository:
```bash
git clone <repository-url>
cd messaging_app
```

2. Copy the environment file template:
```bash
cp backend/rust/.env.example backend/rust/.env
```

3. Update the `.env` file with your configuration:
- Set your JWT secret
- Configure email settings
- Adjust rate limiting parameters
- Update database connection string if needed

4. Start all services (Rust backend, frontend, database, Redis) using Docker Compose from the project root directory:
   ```bash
   # Make sure you are in the root directory of the project
   docker-compose up -d
   ```

5. Build and run the application:
```bash
cargo build --release
cargo run
```

The application will be available at `http://localhost:8080`.

## API Endpoints

### Authentication

- `POST /api/auth/register` - Register a new user
- `POST /api/auth/login` - Login and get JWT tokens
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

- JWT-based authentication
- Password hashing with bcrypt
- Rate limiting
- Email verification
- Session management
- Input validation

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
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.