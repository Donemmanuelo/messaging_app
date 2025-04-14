#!/bin/bash

# Exit on error
set -e

echo "🚀 Setting up the messaging application..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first."
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo is not installed. Please install Cargo first."
    exit 1
fi

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file..."
    cp .env.example .env
    echo "✅ .env file created. Please update the values in .env file."
fi

# Start PostgreSQL container
echo "🐘 Starting PostgreSQL container..."
docker-compose up -d postgres

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
until docker-compose exec postgres pg_isready -U postgres; do
    sleep 1
done

# Run database migrations
echo "📦 Running database migrations..."
cargo install diesel_cli --no-default-features --features postgres
diesel migration run

# Build the application
echo "🔨 Building the application..."
cargo build --release

echo "✨ Setup completed successfully!"
echo "📋 Next steps:"
echo "1. Update the values in .env file"
echo "2. Run the application with: cargo run --release"
echo "3. Access the application at http://localhost:8080" 