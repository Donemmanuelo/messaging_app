#!/bin/bash

# Configuration
MIGRATIONS_DIR="migrations"
DATABASE_URL=${DATABASE_URL:-"postgres://messaging_app:messaging_app@localhost:5432/messaging_app"}

# Function to check if a command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Function to check if SQLx CLI is installed
check_sqlx() {
    if ! command_exists sqlx; then
        echo "SQLx CLI is not installed. Installing..."
        cargo install sqlx-cli
    fi
}

# Function to create a new migration
create_migration() {
    local name=$1
    echo "Creating new migration: $name"
    sqlx migrate add -r "$name"
}

# Function to run migrations
run_migrations() {
    echo "Running database migrations..."
    sqlx database create
    sqlx migrate run
}

# Function to revert migrations
revert_migrations() {
    local steps=${1:-1}
    echo "Reverting last $steps migration(s)..."
    sqlx migrate revert
}

# Function to check migration status
check_status() {
    echo "Checking migration status..."
    sqlx migrate info
}

# Main execution
check_sqlx

case "$1" in
    "create")
        if [ -z "$2" ]; then
            echo "Please provide a name for the migration"
            exit 1
        fi
        create_migration "$2"
        ;;
    "up")
        run_migrations
        ;;
    "down")
        revert_migrations "$2"
        ;;
    "status")
        check_status
        ;;
    *)
        echo "Usage: $0 {create|up|down|status}"
        echo "  create <name>  Create a new migration"
        echo "  up            Run all pending migrations"
        echo "  down [steps]  Revert migrations (default: 1)"
        echo "  status        Check migration status"
        exit 1
        ;;
esac 