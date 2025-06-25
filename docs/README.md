# Messaging App Documentation

## Table of Contents
- [Overview](#overview)
- [Architecture](#architecture)
- [Features](#features)
- [Project Structure](#project-structure)
- [Environment Variables](#environment-variables)
- [API Endpoints](#api-endpoints)
- [WebSocket Protocol](#websocket-protocol)
- [Database Schema](#database-schema)
- [Security](#security)
- [Testing](#testing)
- [DevOps & Deployment](#devops--deployment)
- [Development Workflow](#development-workflow)
- [Extensibility](#extensibility)
- [FAQ](#faq)

---

## Overview
A full-stack messaging application inspired by WhatsApp, featuring real-time chat, group messaging, media sharing, and robust security. The backend is built with Rust (Axum), PostgreSQL, and Redis, while the frontend uses Next.js.

---

## Architecture
- **Backend:** Rust (Axum), REST API, WebSocket, JWT Auth, PostgreSQL, Redis, Docker
- **Frontend:** Next.js, React, HTML/CSS/JS, WebSocket, REST API
- **DevOps:** Docker Compose, Nginx, Grafana (monitoring)

```
[Client (Next.js)] <--> [Nginx] <--> [Axum API/WebSocket] <--> [PostgreSQL, Redis]
```

---

## Features
### Backend
- JWT authentication (login, register, refresh)
- WebSocket real-time chat (1:1, group)
- REST APIs for users, messages, media, groups
- Group chat with admin roles
- Message status (sent, delivered, read)
- Media upload (images, files)
- PostgreSQL (persistent data)
- Redis (caching, pub/sub)
- E2EE preparation (public/private key fields, endpoints)
- OpenAPI documentation
- Dockerized

### Frontend
- Login, registration
- Chat list, chat window
- Group chat UI, admin controls
- Real-time chat (WebSocket)
- Media upload
- Message status indicators
- E2EE key upload/fetch
- Responsive, modern UI

---

## Project Structure
```
backend/
  src/           # Rust source code
    db/          # Database logic
    handlers/    # HTTP/WebSocket handlers
    middleware/  # Auth, CORS, etc.
    models/      # Data models
    routes/      # API routes
    services/    # Business logic
    websocket/   # WebSocket logic
  migrations/    # SQL migrations
  grafana/       # Monitoring dashboards
  scripts/       # Utility scripts
  tests/         # Integration/unit tests
frontend/
  src/app/       # Next.js app directory
    api/         # API route handlers
    components/  # React components
    hooks/       # Custom hooks
    lib/         # Utilities
    store/       # State management
    types/       # TypeScript types
  public/        # Static assets
  styles/        # CSS
  utils/         # Utilities
  stores/        # Zustand stores
  pages/         # (If using pages router)
docs/            # Documentation
```

---

## Environment Variables
### Backend (`backend/.env`)
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `JWT_SECRET` - JWT signing secret
- `MEDIA_UPLOAD_PATH` - Path for storing uploaded media
- `CORS_ORIGIN` - Allowed frontend origin
- `PORT` - API server port
- `RUST_LOG` - Logging level

### Frontend (`frontend/.env`)
- `NEXT_PUBLIC_API_URL` - Backend API base URL
- `NEXT_PUBLIC_WS_URL` - WebSocket URL
- `NEXT_PUBLIC_MEDIA_URL` - Media base URL

> See `docs/backend_env.example.md` and `docs/frontend_env.example.md` for templates.

---

## API Endpoints
### Auth
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login
- `POST /api/auth/refresh` - Refresh JWT

### Users
- `GET /api/users/me` - Get current user
- `GET /api/users/:id` - Get user by ID
- `POST /api/users/public_key` - Upload public key (E2EE)
- `GET /api/users/:id/public_key` - Fetch user's public key

### Chats
- `GET /api/chats` - List user chats
- `POST /api/chats` - Create chat
- `GET /api/chats/:id` - Get chat details
- `GET /api/chats/:id/messages` - List messages
- `POST /api/chats/:id/messages` - Send message

### Groups
- `POST /api/groups` - Create group
- `GET /api/groups/:id` - Get group info
- `POST /api/groups/:id/members` - Add member
- `DELETE /api/groups/:id/members/:userId` - Remove member
- `POST /api/groups/:id/admins` - Add admin
- `DELETE /api/groups/:id/admins/:userId` - Remove admin

### Media
- `POST /api/media/upload` - Upload media
- `GET /api/media/:id` - Fetch media

### Message Status
- `POST /api/messages/:id/status` - Update message status

> All endpoints are prefixed with `/api/`.

---

## WebSocket Protocol
- **Endpoint:** `ws(s)://<host>/api/ws`
- **Auth:** JWT token in query or header
- **Events:**
  - `message:new` - New message
  - `message:status` - Message status update
  - `chat:created` - New chat
  - `group:updated` - Group changes
  - `typing` - Typing indicator
- **Payloads:** JSON objects (see OpenAPI docs)

---

## Database Schema (Simplified)
- **users**: id, username, email, password_hash, public_key, private_key_encrypted, ...
- **chats**: id, is_group, name, ...
- **chat_members**: chat_id, user_id, is_admin
- **messages**: id, chat_id, sender_id, content, media_id, status, timestamp
- **media**: id, url, uploader_id, type, ...
- **message_status**: message_id, user_id, status

---

## Security
- JWT authentication (access/refresh tokens)
- Argon2 password hashing
- Strong password policy
- CORS restrictions
- Input validation (backend & frontend)
- Rate limiting (recommended)
- E2EE preparation (public/private key storage, endpoints)
- HTTPS (recommended in production)

---

## Testing
- **Backend:**
  - Unit tests (handlers, services)
  - Integration tests (API, WebSocket)
  - Group chat, E2EE, media, and WebSocket robustness
- **Frontend:**
  - Component tests
  - API integration tests
  - E2EE key upload/fetch
  - Group chat UI

---

## DevOps & Deployment
- **Docker Compose** for local development
- **Nginx** as reverse proxy
- **Grafana** for monitoring
- **.env** files for configuration
- **CI/CD** (recommended: GitHub Actions, etc.)
- **Production:**
  - Use HTTPS
  - Secure secrets
  - Scale with Docker/Kubernetes

---

## Development Workflow
1. Clone repo & set up `.env` files (see examples)
2. `docker-compose up` to start backend, frontend, db, redis
3. Run migrations (`backend`)
4. Access frontend at `localhost:3000`, backend at `localhost:8000`
5. Run tests as needed

---

## Extensibility
- Add new features by extending API routes and frontend components
- Add new message types (e.g., voice, video)
- Integrate push notifications
- Complete E2EE (key exchange, message encryption)
- Add mobile app (React Native, etc.)

---

## FAQ
**Q: How do I reset my password?**
A: Implement password reset endpoints and UI (not included by default).

**Q: How do I enable full E2EE?**
A: Use the provided key endpoints and extend with client-side encryption/decryption.

**Q: How do I deploy to production?**
A: Use Docker Compose or Kubernetes, secure secrets, enable HTTPS, and monitor with Grafana.

**Q: Where are media files stored?**
A: On the backend server at the path specified by `MEDIA_UPLOAD_PATH`.

---

For further details, see code comments, OpenAPI docs, and the rest of the documentation in this directory.

## Quick Start

1. Clone the repository
   ```