#!/bin/bash

# Configuration
ENV_FILE="/opt/messaging-app/.env.production"
APP_DIR="/opt/messaging-app"
LOG_DIR="/var/log/messaging-app"
BACKUP_DIR="/backups"
UPLOAD_DIR="/opt/messaging-app/uploads"
SYSTEMD_SERVICE="/etc/systemd/system/messaging-app.service"
LOGROTATE_CONFIG="/etc/logrotate.d/messaging-app"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if Docker is running
check_docker() {
    if ! command_exists docker; then
        echo "Error: Docker is not installed"
        exit 1
    fi

    if ! docker info >/dev/null 2>&1; then
        echo "Error: Docker is not running"
        exit 1
    fi
}

# Create production environment file
create_env_file() {
    echo "Creating production environment file..."
    cat > "$ENV_FILE" << EOL
# Application
APP_ENV=production
APP_PORT=3000
APP_HOST=0.0.0.0
RUST_LOG=info

# Database
DATABASE_URL=postgres://messaging_app:messaging_app@db:5432/messaging_app
DATABASE_POOL_SIZE=20

# Redis
REDIS_URL=redis://redis:6379
REDIS_POOL_SIZE=20

# Security
JWT_SECRET=your-jwt-secret-key
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_DURATION=60

# Sentry
SENTRY_DSN=your-sentry-dsn

# Media
MEDIA_UPLOAD_DIR=/uploads
MAX_MEDIA_SIZE=10485760  # 10MB

# Monitoring
PROMETHEUS_METRICS=true
EOL
    echo "Environment file created at $ENV_FILE"
}

# Create necessary directories
create_directories() {
    echo "Creating necessary directories..."
    sudo mkdir -p "$APP_DIR"
    sudo mkdir -p "$LOG_DIR"
    sudo mkdir -p "$UPLOAD_DIR"
    sudo mkdir -p "$BACKUP_DIR"

    # Set permissions
    sudo chown -R $USER:$USER "$APP_DIR"
    sudo chown -R $USER:$USER "$LOG_DIR"
    sudo chown -R $USER:$USER "$BACKUP_DIR"
    echo "Directories created and permissions set"
}

# Setup systemd service
setup_systemd() {
    echo "Setting up systemd service..."
    sudo tee "$SYSTEMD_SERVICE" > /dev/null << EOL
[Unit]
Description=Messaging App
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$APP_DIR
EnvironmentFile=$ENV_FILE
ExecStart=/usr/local/bin/docker-compose up
ExecStop=/usr/local/bin/docker-compose down
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOL

    sudo systemctl daemon-reload
    sudo systemctl enable messaging-app
    sudo systemctl start messaging-app
    echo "Systemd service setup complete"
}

# Setup log rotation
setup_logrotate() {
    echo "Setting up log rotation..."
    sudo tee "$LOGROTATE_CONFIG" > /dev/null << EOL
$LOG_DIR/*.log {
    daily
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 $USER $USER
    sharedscripts
    postrotate
        systemctl reload messaging-app
    endscript
}
EOL
    echo "Log rotation setup complete"
}

# Main execution
echo "Starting production setup..."

# Check prerequisites
check_docker

# Create environment file
create_env_file

# Create directories
create_directories

# Setup systemd service
setup_systemd

# Setup log rotation
setup_logrotate

echo "Production setup completed successfully!"
echo "Please update the following in $ENV_FILE:"
echo "1. JWT_SECRET with a secure random string"
echo "2. SENTRY_DSN with your Sentry project DSN"
echo "3. Database credentials if different from defaults" 