# Messaging App Backend

A production-ready backend for a real-time messaging application built with Rust, Axum, and PostgreSQL.

## Features

- Real-time messaging with WebSocket support
- Group chat functionality
- Media upload and management
- Rate limiting and security features
- Health monitoring and logging
- Production-ready configuration

## Prerequisites

- Rust (latest stable version)
- PostgreSQL 13+
- Redis 6+
- Cloudinary account (for media storage)

## Setup

1. Clone the repository
2. Create a `.env` file in the `backend` directory with the following variables:

```env
# Application
APP_ENV=development
PORT=3000
FRONTEND_URL=http://localhost:3000

# Database
DATABASE_URL=postgres://postgres:postgres@localhost:5432/messaging_app

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-secret-key-here

# Cloudinary
CLOUDINARY_CLOUD_NAME=your-cloud-name
CLOUDINARY_API_KEY=your-api-key
CLOUDINARY_API_SECRET=your-api-secret

# Rate Limiting
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW=60

# Logging
RUST_LOG=info
```

3. Set up the database:
```bash
./scripts/setup_db.sh
```

4. Build and run the application:
```bash
cargo build --release
cargo run --release
```

## API Endpoints

### Health Checks
- `GET /health` - Overall system health
- `GET /ready` - Readiness probe
- `GET /live` - Liveness probe

### Messages
- `POST /api/messages` - Send a message
- `GET /api/messages/:id` - Get a message
- `PUT /api/messages/:id` - Update a message
- `DELETE /api/messages/:id` - Delete a message
- `POST /api/messages/group` - Send a group message

### Media
- `POST /api/media` - Upload media
- `DELETE /api/media/:id` - Delete media

## Production Deployment

For production deployment:

1. Set `APP_ENV=production`
2. Configure proper SSL/TLS termination
3. Set secure values for all secrets
4. Configure proper database and Redis credentials
5. Set appropriate rate limits
6. Configure proper logging levels

## Security Features

- Rate limiting per IP
- CORS protection
- Input validation
- Secure media upload handling
- JWT authentication
- HTTPS enforcement in production

## Monitoring

The application includes:
- Health check endpoints
- Structured logging
- Request tracing
- Error tracking
- Performance monitoring

## Development

To run in development mode:
```bash
cargo run
```

To run tests:
```bash
cargo test
```

## License

MIT 