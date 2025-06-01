#!/bin/bash

# Configuration
APP_NAME="messaging-app"
BLUE_PORT=3000
GREEN_PORT=3001
CURRENT_COLOR=$(docker-compose ps -q app | xargs -I {} docker inspect -f '{{index .Config.Labels "color"}}' {} 2>/dev/null || echo "blue")
NEW_COLOR=$([ "$CURRENT_COLOR" = "blue" ] && echo "green" || echo "blue")
NEW_PORT=$([ "$NEW_COLOR" = "blue" ] && echo $BLUE_PORT || echo $GREEN_PORT)

# Function to check if a command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Function to check if Docker is running
check_docker() {
    if ! docker info &> /dev/null; then
        echo "Docker is not running. Please start Docker and try again."
        exit 1
    fi
}

# Function to build the new version
build_new_version() {
    echo "Building new version..."
    docker-compose build app
}

# Function to start the new version
start_new_version() {
    echo "Starting new version ($NEW_COLOR)..."
    COLOR=$NEW_COLOR PORT=$NEW_PORT docker-compose up -d app
}

# Function to wait for the new version to be healthy
wait_for_health() {
    echo "Waiting for new version to be healthy..."
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s "http://localhost:$NEW_PORT/health" | grep -q "healthy"; then
            echo "New version is healthy!"
            return 0
        fi
        
        echo "Attempt $attempt/$max_attempts: New version not healthy yet..."
        sleep 2
        attempt=$((attempt + 1))
    done
    
    echo "New version failed to become healthy within the timeout period"
    return 1
}

# Function to update the load balancer
update_load_balancer() {
    echo "Updating load balancer to point to new version..."
    # This is a placeholder for your actual load balancer update logic
    # You might use nginx, haproxy, or a cloud provider's load balancer
    # For example, with nginx:
    # sed -i "s/proxy_pass http:\/\/.*:.*/proxy_pass http:\/\/localhost:$NEW_PORT/" nginx/nginx.conf
    # docker-compose restart nginx
}

# Function to stop the old version
stop_old_version() {
    echo "Stopping old version ($CURRENT_COLOR)..."
    COLOR=$CURRENT_COLOR docker-compose stop app
}

# Function to rollback if something goes wrong
rollback() {
    echo "Rolling back to previous version..."
    COLOR=$CURRENT_COLOR docker-compose up -d app
    COLOR=$NEW_COLOR docker-compose stop app
    exit 1
}

# Main execution
echo "Starting deployment..."

# Check prerequisites
check_docker

# Build new version
build_new_version || rollback

# Start new version
start_new_version || rollback

# Wait for new version to be healthy
wait_for_health || rollback

# Update load balancer
update_load_balancer || rollback

# Stop old version
stop_old_version

echo "Deployment completed successfully!"
echo "New version is running on port $NEW_PORT" 