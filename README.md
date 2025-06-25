# Messaging App

A full-stack, production-ready messaging application inspired by WhatsApp. Features include real-time chat, group chat, media uploads, message status, JWT authentication, end-to-end encryption (E2EE) ready, PostgreSQL, Redis, and Docker support.

---

## Table of Contents
- [Features](#features)
- [Architecture](#architecture)
- [Requirements](#requirements)
- [Quick Start (Docker)](#quick-start-docker)
- [Manual Setup](#manual-setup)
  - [Backend](#backend)
  - [Frontend](#frontend)
- [Environment Variables](#environment-variables)
- [Database Migrations](#database-migrations)
- [Running Tests](#running-tests)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)

---

## Features
- JWT authentication (secure login/register)
- Real-time chat via WebSocket
- Group chat with admin roles
- Message status (delivered/read)
- Media upload (images, files)
- End-to-end encryption (E2EE) ready
- PostgreSQL for persistent storage
- Redis for caching and pub/sub
- Dockerized for easy deployment
- Comprehensive test coverage

---

## Architecture
- **Backend:** Rust (Axum), SQLx, JWT, WebSocket, PostgreSQL, Redis
- **Frontend:** Next.js (React), TypeScript, HTML/CSS/JS
- **DevOps:** Docker, docker-compose, Nginx (optional), environment variable management

---

## Requirements
- Docker & Docker Compose (recommended for easiest setup)
- Or, for manual setup:
  - Rust (latest stable)
  - Node.js (v18+ recommended)
  - PostgreSQL (v14+)
  - Redis (v6+)

---

## Quick Start (Docker)

1. **Clone the repository:**
   ```sh
   git clone <repo-url>
   cd messaging_app
   ```

2. **Copy environment variable templates:**
   ```sh
   cp backend/.env.example backend/.env
   cp frontend/.env.example frontend/.env
   ```
   Edit the `.env` files as needed (see [Environment Variables](#environment-variables)).

3. **Start all services:**
   ```sh
   docker-compose up --build
   ```
   - Backend: http://localhost:8000
   - Frontend: http://localhost:3000
   - PostgreSQL: localhost:5432
   - Redis: localhost:6379

4. **Access the app:**
   - Open [http://localhost:3000](http://localhost:3000) in your browser.

---

## Manual Setup

### Backend
1. **Install dependencies:**
   - Rust: https://rustup.rs/
   - Install SQLx CLI (for migrations):
     ```sh
     cargo install sqlx-cli --no-default-features --features postgres
     ```
2. **Set up environment variables:**
   ```sh
   cp backend/.env.example backend/.env
   # Edit backend/.env with your DB/Redis credentials
   ```
3. **Start PostgreSQL and Redis** (if not using Docker):
   - PostgreSQL: `sudo service postgresql start`
   - Redis: `sudo service redis-server start`
4. **Run database migrations:**
   ```sh
   cd backend
   sqlx migrate run
   ```
5. **Run the backend server:**
   ```sh
   cargo run
   ```
   - API: http://localhost:8000/api/

### Frontend
1. **Install dependencies:**
   ```sh
   cd frontend
   npm install
   ```
2. **Set up environment variables:**
   ```sh
   cp .env.example .env
   # Edit .env as needed
   ```
3. **Run the frontend dev server:**
   ```sh
   npm run dev
   ```
   - App: http://localhost:3000

---

## Environment Variables

### Backend (`backend/.env`)
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `JWT_SECRET` - Secret for JWT signing
- `MEDIA_UPLOAD_PATH` - Path for storing uploaded media
- `RUST_LOG` - Log level (e.g., info, debug)

### Frontend (`frontend/.env`)
- `NEXT_PUBLIC_API_URL` - Backend API base URL (e.g., http://localhost:8000/api)
- `NEXT_PUBLIC_WS_URL` - WebSocket URL (e.g., ws://localhost:8000/ws)

See `.env.example` files in each directory for all options.

---

## Database Migrations
- **Run migrations:**
  ```sh
  cd backend
  sqlx migrate run
  ```
- **Prepare SQLx query cache (optional, for offline builds):**
  ```sh
  cargo sqlx prepare
  ```

---

## Running Tests

### Backend
```sh
cd backend
cargo test
```

### Frontend
```sh
cd frontend
npm test
```

---

## Troubleshooting
- **Port conflicts:** Change ports in `.env` or `docker-compose.yml` if needed.
- **Database errors:** Ensure PostgreSQL and Redis are running and credentials are correct.
- **SQLx errors:** Set `DATABASE_URL` or run `cargo sqlx prepare` before building.
- **Frontend API errors:** Check `NEXT_PUBLIC_API_URL` in `frontend/.env`.
- **CORS issues:** Ensure backend CORS settings allow frontend origin.

---

## Contributing
Pull requests and issues are welcome! Please see [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

---

## License
MIT 