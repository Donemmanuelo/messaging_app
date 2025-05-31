# WhatsApp Clone

A full-featured WhatsApp clone built with Rust (Axum) for the backend and Next.js for the frontend.

## Features

- User authentication (register, login, logout)
- Real-time messaging with WebSockets
- One-to-one and group chats
- Message statuses (sent, delivered, read)
- Typing indicators
- Media sharing (images, videos, audio, documents)
- User presence (online/offline status)
- Message history
- Reply to messages
- User profiles

## Tech Stack

### Backend
- Rust
- Axum web framework
- PostgreSQL with SQLx
- Redis for WebSocket pub/sub
- JWT authentication

### Frontend
- Next.js
- Tailwind CSS
- Zustand for state management
- WebSockets for real-time communication

## Getting Started

### Prerequisites
- Docker and Docker Compose
- Node.js (for local development)
- Rust (for local development)

### Running with Docker Compose

1. Clone the repository:pl