#!/bin/bash

# Check if PostgreSQL is running
if ! pg_isready; then
    echo "Starting PostgreSQL..."
    brew services start postgresql
    sleep 5  # Wait for PostgreSQL to initialize
fi

# Create database if it doesn't exist
createdb messaging_app 2>/dev/null || true

# Set DATABASE_URL
export DATABASE_URL="postgres://localhost/messaging_app"

# Run migrations
echo "Running database migrations..."
sqlx database create
sqlx migrate run

# Prepare SQLx
echo "Preparing SQLx..."
cargo sqlx prepare

echo "Database setup complete!" 