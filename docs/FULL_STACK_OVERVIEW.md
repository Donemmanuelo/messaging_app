# Messaging App – Full-Stack Documentation

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Backend](#backend)
    - [Tech Stack](#backend-tech-stack)
    - [Features](#backend-features)
    - [Project Structure](#backend-project-structure)
    - [Environment Variables](#backend-environment-variables)
    - [API Endpoints](#backend-api-endpoints)
    - [WebSocket Protocol](#websocket-protocol)
    - [Database](#database)
    - [Security](#backend-security)
    - [Testing](#backend-testing)
4. [Frontend](#frontend)
    - [Tech Stack](#frontend-tech-stack)
    - [Features](#frontend-features)
    - [Project Structure](#frontend-project-structure)
    - [Environment Variables](#frontend-environment-variables)
    - [Key UI Flows](#key-ui-flows)
    - [Testing](#frontend-testing)
5. [DevOps & Deployment](#devops--deployment)
    - [Docker & Compose](#docker--compose)
    - [Production Considerations](#production-considerations)
6. [Development Workflow](#development-workflow)
7. [Extending the App](#extending-the-app)
8. [FAQ](#faq)

---

## Overview

This project is a scalable, production-ready messaging application inspired by WhatsApp, featuring real-time chat, group messaging, media uploads, and robust authentication. It is built with a modern Rust backend (Axum/Actix-Web) and a Next.js frontend.

---

## Architecture

```mermaid
graph TD
  subgraph Frontend (Next.js)
    A[Login/Register]
    B[Chat List]
    C[Chat Window]
    D[Group Chat UI]
    E[Media Upload UI]
  end
  subgraph Backend (Rust)
    F[REST API]
    G[WebSocket Server]
    H[Media Service]
    I[Group Management]
    J[User Service]
    K[JWT Auth]
    L[E2EE Key Service]
  end
  subgraph DBs
    M[(PostgreSQL)]
    N[(Redis)]
  end
  A-->|REST/JWT|F
  B-->|REST/JWT|F
  C-->|WebSocket|G
  D-->|REST|I
  E-->|REST|H
  F-->|SQL|M
  G-->|Session|N
  H-->|SQL|M
  I-->|SQL|M
  J-->|SQL|M
  K-->|Session|N
  L-->|SQL|M
```

---

## Backend

### Backend Tech Stack

- **Language:** Rust
- **Framework:** Axum (or Actix-Web)
- **Database:** PostgreSQL (SQLx)
- **Cache/Session:** Redis
- **Authentication:** JWT
- **WebSocket:** Real-time chat
- **Media:** Local/Cloudinary uploads
- **Containerization:** Docker

### Backend Features

- JWT-based authentication (login, registration)
- REST APIs for users, messages, groups, media
- WebSocket for real-time chat
- Group chat with admin roles
- Message status: sent, delivered, seen
- Media uploads (images, videos, docs)
- E2EE-ready: public/private key fields and endpoints
- Rate limiting, CORS, security headers
- OpenAPI/Swagger documentation

### Backend Project Structure

```
backend/
  src/
    handlers/      # Request handlers (controllers)
    models/        # Data models (User, Message, Group, etc.)
    routes/        # Route definitions
    services/      # Business logic
    websocket/     # WebSocket logic
    db/            # Database utilities
    middleware/    # Middleware (auth, CORS, etc.)
  migrations/      # SQL migrations
  tests/           # Integration tests
  scripts/         # Utility scripts
  Dockerfile
  docker-compose.yml
  README.md
```

### Backend Environment Variables

Create a `.env` file (see `.env.example`):

```
DATABASE_URL=postgres://postgres:password@db:5432/messaging_app
REDIS_URL=redis://redis:6379
JWT_SECRET=your_jwt_secret
CLOUDINARY_URL=cloudinary://api_key:api_secret@cloud_name
FRONTEND_URL=http://localhost:3000
RATE_LIMIT_PER_MIN=60
LOG_LEVEL=info
```

### Backend API Endpoints

- **Auth:**  
  - `POST /api/register` – Register a new user  
  - `POST /api/login` – Login and receive JWT

- **Users:**  
  - `GET /api/users` – List users  
  - `PATCH /api/users/:user_id` – Update profile  
  - `POST /api/users/:user_id/avatar` – Upload avatar  
  - `POST /api/users/:user_id/public_key` – Upload E2EE public key  
  - `GET /api/users/:user_id/public_key` – Fetch E2EE public key

- **Groups:**  
  - `POST /api/groups` – Create group  
  - `GET /api/groups` – List groups  
  - `POST /api/groups/:group_id/avatar` – Upload group avatar

- **Messages:**  
  - `POST /api/messages` – Send message  
  - `GET /api/messages/:chat_id` – Fetch messages

- **Media:**  
  - `POST /api/media/upload` – Upload media

- **Message Reads:**  
  - `POST /api/message_reads` – Mark as read  
  - `GET /api/message_reads/:message_id` – Get read status

- **WebSocket:**  
  - `ws://<host>/ws` – Real-time chat

### WebSocket Protocol

- **Connect:**  
  - Authenticate with JWT on connect
- **Events:**  
  - `message` – Send/receive messages
  - `status` – Typing, delivered, seen
  - `group` – Group events (join, leave, update)
- **Message Format:**  
  ```json
  {
    "type": "message",
    "chat_id": "uuid",
    "content": "Hello!",
    "media_url": null,
    "status": "sent"
  }
  ```

### Database

- **PostgreSQL**: Stores users, messages, groups, media metadata, E2EE keys
- **Redis**: Caches sessions, rate limits, and WebSocket presence

### Backend Security

- **JWT**: All protected endpoints require a valid JWT
- **Password Policy**: Enforced on registration (min 12 chars, upper/lower/number/special)
- **CORS**: Only allows requests from `FRONTEND_URL`
- **Rate Limiting**: Per-user, per-IP
- **Security Headers**: Set via middleware

### Backend Testing

- **Integration Tests**: For all major endpoints and flows
- **Load Testing**: With k6 scripts
- **Test Coverage**: Group chat, E2EE, media, WebSocket

---

## Frontend

### Frontend Tech Stack

- **Framework:** Next.js (React)
- **Language:** TypeScript
- **Styling:** Tailwind CSS
- **State Management:** React Context/Store
- **Testing:** Jest, Vitest, React Testing Library
- **WebSocket:** Native/WebSocketClient wrapper

### Frontend Features

- Login/registration with JWT
- Chat list and chat window
- Group chat creation and management
- Real-time chat via WebSocket
- Media upload UI
- Message status indicators (sent, delivered, seen)
- E2EE key upload and management

### Frontend Project Structure

```
frontend/
  src/
    app/            # App directory (Next.js routing)
      api/          # API route handlers
      auth/         # Auth pages
      chat/         # Chat pages
      group/        # Group chat pages
    components/     # Reusable UI components
    hooks/          # Custom React hooks
    lib/            # Utilities and WebSocket client
    store/          # State management
    types/          # TypeScript types
  public/           # Static assets
  styles/           # Tailwind and global styles
  socket/           # WebSocket logic
  pages/            # (If using pages directory)
  Dockerfile
  package.json
  README.md
```

### Frontend Environment Variables

Create a `.env.local` file (see `.env.example`):

```
NEXT_PUBLIC_API_URL=http://localhost:8080/api
NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws
```

### Key UI Flows

- **Registration:**  
  - User registers, then uploads/generates E2EE public key
- **Login:**  
  - User logs in, public key is fetched and stored
- **Chat:**  
  - Real-time messages, media, and status indicators
- **Group Chat:**  
  - Create group, add/remove members, assign admin roles
- **Media Upload:**  
  - Upload images, videos, docs to chat

### Frontend Testing

- **Component Tests:** For chat, sidebar, message, group UI
- **Integration Tests:** For auth, chat flows
- **Mocked WebSocket and API calls**

---

## DevOps & Deployment

### Docker & Compose

- **Backend** and **frontend** each have a `Dockerfile`
- `docker-compose.yml` orchestrates:
  - Backend (Rust)
  - Frontend (Next.js)
  - PostgreSQL
  - Redis

**Example:**
```yaml
services:
  backend:
    build: ./backend
    env_file: ./backend/.env
    ports: ["8080:8080"]
    depends_on: [db, redis]
  frontend:
    build: ./frontend
    env_file: ./frontend/.env.local
    ports: ["3000:3000"]
    depends_on: [backend]
  db:
    image: postgres:15
    environment: ...
  redis:
    image: redis:7
```

### Production Considerations

- Use strong secrets and secure environment variables
- Enable HTTPS (via reverse proxy like Nginx)
- Use persistent storage for DB and media
- Monitor with Prometheus/Grafana (already scaffolded)
- Scale backend/frontend with Docker Swarm/Kubernetes if needed

---

## Development Workflow

1. **Clone the repo**
2. **Set up environment variables** (`.env`, `.env.local`)
3. **Run with Docker Compose:**  
   ```sh
   docker-compose up --build
   ```
4. **Run tests:**  
   - Backend: `cargo test`
   - Frontend: `npm test` or `npx vitest`

---

## Extending the App

- **End-to-End Encryption:**  
  - Implement client-side key generation and message encryption
- **Push Notifications:**  
  - Integrate with service workers and backend events
- **Mobile App:**  
  - Use React Native with the same API/WebSocket
- **Advanced Moderation:**  
  - Add admin tools, reporting, and analytics

---

## FAQ

**Q: How do I reset the database?**  
A: Use `cargo sqlx database reset` in the backend directory.

**Q: How do I add a new feature?**  
A: Add a new handler/service in the backend, expose via API, and consume in the frontend.

**Q: How do I debug WebSocket issues?**  
A: Use browser dev tools and backend logs. WebSocket events are logged with user/session info.

---

**For more details, see the `README.md` files in each directory.**  
If you need further architectural diagrams, onboarding guides, or API reference, let me know! 