# Backend Environment Variables Example (.env)

# PostgreSQL connection string
DATABASE_URL=postgres://user:password@localhost:5432/messaging_app

# Redis connection string
REDIS_URL=redis://localhost:6379

# JWT signing secret (use a strong, random value)
JWT_SECRET=your_jwt_secret_here

# Path for storing uploaded media (absolute or relative)
MEDIA_UPLOAD_PATH=./media

# Allowed CORS origin (frontend URL)
CORS_ORIGIN=http://localhost:3000

# API server port
PORT=8000

# Rust log level (e.g., info, debug, warn, error)
RUST_LOG=info 