# Messaging App: Full-Stack Architecture & Documentation

## Overview

This document provides a comprehensive overview of the messaging application's architecture, backend and frontend design, security, DevOps, and extension points. It is intended for developers, DevOps engineers, and contributors.

---

## Table of Contents
1. [System Architecture](#system-architecture)
2. [Backend](#backend)
3. [Frontend](#frontend)
4. [DevOps & Deployment](#devops--deployment)
5. [Security](#security)
6. [Testing](#testing)
7. [Extension Points](#extension-points)
8. [Environment Variables](#environment-variables)
9. [References](#references)

---

## 1. System Architecture

- **Backend:** Rust (Axum), PostgreSQL, Redis, JWT, WebSocket, REST APIs, Docker
- **Frontend:** Next.js (React), HTML/CSS/JS, WebSocket, REST APIs
- **Real-Time:** WebSocket for chat, Redis for pub/sub
- **Media:** File uploads (images, videos, etc.)
- **Security:** JWT, CORS, rate limiting, E2EE-ready
- **Deployment:** Docker Compose, Nginx, environment variables

### High-Level Diagram

```
[User] <-> [Next.js Frontend] <-> [Axum Backend] <-> [PostgreSQL]
                                      |                |
                                      |                |
                                      +--> [Redis] <---+
                                      |
                                      +--> [Media Storage]
```

---

## 2. Backend

### 2.1. Tech Stack
- **Language:** Rust
- **Framework:** Axum
- **Database:** PostgreSQL (diesel/sqlx/sea-orm)
- **Cache/Queue:** Redis
- **Auth:** JWT (with strong password policy)
- **Real-Time:** WebSocket
- **Media:** File upload endpoints
- **Containerization:** Docker

### 2.2. Main Features
- User registration/login (JWT)
- Password policy: min 12 chars, upper/lower/number/special
- Group chat (admin roles, add/remove users)
- One-to-one and group messaging
- Message status (sent, delivered, read)
- Media upload/download
- E2EE-ready: public/private key fields, key exchange endpoints
- OpenAPI documentation

### 2.3. Directory Structure
- `src/models/` - DB models (User, Message, Group, etc.)
- `src/routes/` - REST API endpoints
- `src/websocket/` - WebSocket logic
- `src/services/` - Business logic
- `src/db/` - DB connection, migrations
- `src/middleware/` - Auth, CORS, rate limiting

### 2.4. Key Endpoints
- `/api/auth/register` - Register user
- `/api/auth/login` - Login
- `/api/users/public_key` - Upload/fetch public key
- `/api/chats/` - List/create chats
- `/api/chats/{id}/messages` - Send/fetch messages
- `/api/groups/` - Group chat management
- `/api/media/upload` - Media upload
- `/ws/` - WebSocket endpoint

### 2.5. E2EE Support
- User model: `public_key`, `private_key_encrypted`
- Endpoints for key upload/fetch
- (Client-side encryption/decryption recommended)

---

## 3. Frontend

### 3.1. Tech Stack
- **Framework:** Next.js (React)
- **State:** React Context/Redux/Zustand
- **WebSocket:** Real-time chat
- **Media:** File upload UI
- **Testing:** Jest, React Testing Library

### 3.2. Main Features
- Login/registration UI
- Chat list, chat window
- Group chat creation, admin controls
- Real-time chat (WebSocket)
- Media upload
- Message status indicators
- E2EE UI: upload/fetch public key

### 3.3. Directory Structure
- `src/app/` - App routes
- `src/components/` - UI components
- `src/hooks/` - Custom hooks
- `src/store/` - State management
- `src/types/` - TypeScript types
- `src/socket/` - WebSocket logic

### 3.4. E2EE UI
- Upload public key after registration
- Fetch public key after login
- (Encrypt/decrypt messages client-side)

---

## 4. DevOps & Deployment

### 4.1. Docker
- `docker-compose.yml` for multi-service orchestration
- Backend, frontend, PostgreSQL, Redis, Nginx

### 4.2. Nginx
- Reverse proxy for frontend/backend
- SSL/TLS termination (recommended for production)

### 4.3. Environment Variables
- `.env.example` files for backend/frontend
- Document all required variables

### 4.4. CI/CD (Recommended)
- Lint, test, build, deploy
- GitHub Actions/GitLab CI

---

## 5. Security

- JWT authentication
- Strong password policy
- CORS configuration
- Security headers (Nginx, backend)
- Rate limiting
- E2EE-ready (public/private key fields, endpoints)
- Media upload validation

---

## 6. Testing

### 6.1. Backend
- Integration tests: group chat, E2EE endpoints, WebSocket
- Unit tests: services, models

### 6.2. Frontend
- Jest, React Testing Library
- E2EE UI, group chat, media upload, WebSocket reconnect

---

## 7. Extension Points

- **E2EE:** Implement client-side encryption/decryption
- **Push Notifications:** Integrate with FCM/APNs
- **Scalability:** Use Kubernetes, horizontal scaling
- **Monitoring:** Grafana, Prometheus
- **Mobile App:** React Native/Flutter

---

## 8. Environment Variables

### Backend (`.env.example`)
```
DATABASE_URL=postgres://user:password@db:5432/messaging_app
REDIS_URL=redis://redis:6379
JWT_SECRET=your_jwt_secret
MEDIA_UPLOAD_PATH=./media
CORS_ORIGIN=http://localhost:3000
```

### Frontend (`.env.example`)
```
NEXT_PUBLIC_API_URL=http://localhost:8000/api
NEXT_PUBLIC_WS_URL=ws://localhost:8000/ws
```

---

## 9. References
- [Axum](https://github.com/tokio-rs/axum)
- [Next.js](https://nextjs.org/)
- [PostgreSQL](https://www.postgresql.org/)
- [Redis](https://redis.io/)
- [Docker](https://www.docker.com/)
- [Nginx](https://nginx.org/)
- [Jest](https://jestjs.io/)
- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/) 