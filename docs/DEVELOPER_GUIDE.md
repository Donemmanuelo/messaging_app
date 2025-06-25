# Developer Guide

## Onboarding
- Clone the repo and read the root README.md
- See `docs/ARCHITECTURE.md` for system overview
- See `docs/API.md` for API reference

## Code Structure
- **frontend/**: Next.js app (pages, components, socket, utils, styles)
- **backend/**: Axum app (handlers, models, services, websocket, db, utils)
- **docker-compose.yml**: Orchestrates all services
- **nginx.conf**: Reverse proxy config
- **.github/workflows/**: CI/CD pipeline
- **docs/**: Architecture, API, user/developer guides

## Adding Features
- **Backend:**
  - Add new endpoints in `backend/src/handlers/` and `routes/`
  - Update models in `backend/src/models/`
  - Add DB migrations in `backend/migrations/`
  - Document new endpoints in `api_docs.rs` (OpenAPI)
- **Frontend:**
  - Add new pages/components in `frontend/pages/` or `frontend/components/`
  - Use `frontend/utils/api.js` for API calls
  - Use `frontend/socket/` for WebSocket logic

## Running Tests
- **Backend:**
  ```sh
  cd backend
  cargo test
  ```
- **Frontend:**
  ```sh
  cd frontend
  npm test
  ```

## CI/CD
- See `.github/workflows/deploy.yml` for build/test pipeline
- Add secrets for Docker/image deploys as needed

## Deployment
- Use `docker-compose up --build` for local/prod
- See `frontend/README.md` for Vercel/Netlify deploy
- Use `nginx.conf` for SSL/reverse proxy

## Monitoring & Logging
- Use Prometheus/Grafana for metrics (see backend/prometheus.yml)
- Use Sentry for frontend error monitoring
- Use `tracing` for backend logs

## Contributing
- Fork, branch, and submit PRs
- Write tests for new features
- Update docs as needed 