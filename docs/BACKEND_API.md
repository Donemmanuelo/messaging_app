# Messaging App – Backend API Documentation

## Overview

This project is a modern, scalable, and secure messaging application featuring:

- Real-time chat (1:1 and group)
- Media uploads (images, files)
- User authentication (JWT)
- Group management
- Read receipts, reactions, and message search
- Caching with Redis
- Monitoring and observability
- Dockerized deployment
- **Frontend:** Next.js (React)
- **Backend:** Rust (Axum, SQLx, PostgreSQL, Redis)

---

## Architecture

```mermaid
graph TD
  subgraph Frontend (Next.js)
    A[User Browser] --> B[React App]
  end
  subgraph Backend (Rust/Axum)
    B --> C[REST API/WebSocket]
    C --> D[PostgreSQL]
    C --> E[Redis]
    C --> F[Media Storage]
  end
```

---

## Environment Setup

### Backend

- **Rust** (nightly or stable)
- **PostgreSQL** (v13+)
- **Redis** (v6+)
- **Docker** (optional, for deployment)

#### Required Environment Variables

| Variable         | Description                        | Example                                             |
|------------------|------------------------------------|-----------------------------------------------------|
| `DATABASE_URL`   | PostgreSQL connection string       | `postgres://user:password@localhost:5432/messaging_app` |
| `REDIS_URL`      | Redis connection string            | `redis://localhost:6379`                            |
| `JWT_SECRET`     | Secret for JWT signing             | `supersecretkey`                                    |
| `CLOUDINARY_*`   | (Optional) For media uploads       | See `.env.example`                                  |

---

## API Endpoints

### Authentication

| Method | URL                | Description                | Body/Params                |
|--------|--------------------|----------------------------|----------------------------|
| POST   | `/api/register`    | Register a new user        | `{ username, email, password }` |
| POST   | `/api/login`       | Login and get JWT          | `{ email, password }`      |

---

### Users

| Method | URL                                 | Description                        |
|--------|-------------------------------------|------------------------------------|
| GET    | `/api/users`                        | List all users with online status  |
| PATCH  | `/api/users/:user_id`               | Update user profile                |
| POST   | `/api/users/:user_id/avatar`        | Upload user avatar (multipart)     |
| POST   | `/api/users/:user_id/public_key`    | Upload/update public key (E2EE)    |
| GET    | `/api/users/:user_id/public_key`    | Fetch public key                   |

---

### Chats & Messages

| Method | URL                                         | Description                        |
|--------|---------------------------------------------|------------------------------------|
| POST   | `/api/messages/:receiver_id`                | Send a direct message              |
| GET    | `/api/messages/:chat_id`                    | Fetch messages in a chat           |
| POST   | `/api/messages/:group_id/group`             | Send a group message               |
| GET    | `/api/messages/:group_id/group`             | Fetch group messages               |
| PATCH  | `/api/messages/:message_id`                 | Edit a message                     |
| DELETE | `/api/messages/:message_id`                 | Delete a message                   |

---

### Groups

| Method | URL                                 | Description                        |
|--------|-------------------------------------|------------------------------------|
| POST   | `/api/groups`                       | Create a group                     |
| GET    | `/api/groups`                       | List groups                        |
| GET    | `/api/groups/:group_id`             | Get group details                  |
| PATCH  | `/api/groups/:group_id`             | Update group                       |
| POST   | `/api/groups/:group_id/avatar`      | Upload group avatar                |
| POST   | `/api/groups/:group_id/members/:user_id` | Add member to group           |
| DELETE | `/api/groups/:group_id/members/:user_id` | Remove member from group      |
| GET    | `/api/groups/:group_id/members`     | List group members                 |

---

### Media

| Method | URL                | Description                |
|--------|--------------------|----------------------------|
| POST   | `/api/media/upload`| Upload media file (multipart) |

---

### Reactions & Read Receipts

| Method | URL                                         | Description                        |
|--------|---------------------------------------------|------------------------------------|
| POST   | `/api/messages/:message_id/reactions`       | Add a reaction                     |
| DELETE | `/api/messages/:message_id/reactions`       | Remove a reaction                  |
| GET    | `/api/messages/:message_id/reactions`       | Get reactions for a message        |
| POST   | `/api/messages/:message_id/read`            | Mark message as read               |
| GET    | `/api/messages/:message_id/readers`         | Get read receipts                  |

---

### WebSocket

| URL      | Description                  |
|----------|-----------------------------|
| `/ws`    | Real-time chat and events    |

---

## Error Handling
- All errors are returned as JSON with an `error` field and appropriate HTTP status code.
- Common errors: `400 Bad Request`, `401 Unauthorized`, `403 Forbidden`, `404 Not Found`, `500 Internal Server Error`.

---

## Deployment
- Use Docker Compose for local and production deployments.
- See `backend/docker-compose.yml` and `backend/docker-compose.prod.yml` for service definitions.
- Monitoring via Prometheus and Grafana (see `backend/grafana/` and `backend/prometheus.yml`).

---

## Contribution & Development
- See `docs/DEVELOPER_GUIDE.md` and `docs/DEVELOPMENT.md` for setup, testing, and contribution guidelines.
- Run migrations in `backend/migrations/` before starting the backend.

---

## Contact
For questions or support, see the project README or contact the maintainers. 