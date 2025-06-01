#!/bin/bash

# Configuration
LOG_DIR="/var/log/messaging-app/updates"
BACKUP_DIR="/backups/dependencies"
CRON_JOB="0 2 * * 0"  # Weekly at 2 AM on Sunday

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to create necessary directories
create_directories() {
    echo "Creating necessary directories..."
    sudo mkdir -p "$LOG_DIR"
    sudo mkdir -p "$BACKUP_DIR"
    sudo chown -R $USER:$USER "$LOG_DIR"
    sudo chown -R $USER:$USER "$BACKUP_DIR"
}

# Function to backup current dependencies
backup_dependencies() {
    echo "Backing up current dependencies..."
    timestamp=$(date +%Y%m%d_%H%M%S)
    
    # Backup Cargo.lock
    cp Cargo.lock "$BACKUP_DIR/Cargo.lock.$timestamp"
    
    # Backup package.json if it exists
    if [ -f "package.json" ]; then
        cp package.json "$BACKUP_DIR/package.json.$timestamp"
    fi
    
    # Backup Docker images
    docker save messaging-app:latest > "$BACKUP_DIR/messaging-app.$timestamp.tar"
}

# Function to update Rust dependencies
update_rust_dependencies() {
    echo "Updating Rust dependencies..."
    
    # Update cargo
    rustup update
    
    # Update dependencies
    cargo update
    
    # Check for outdated dependencies
    cargo outdated
    
    # Run cargo audit
    cargo audit
    
    # Build to verify updates
    cargo build --release
}

# Function to update system packages
update_system_packages() {
    echo "Updating system packages..."
    
    if command_exists apt-get; then
        sudo apt-get update
        sudo apt-get upgrade -y
        sudo apt-get autoremove -y
    elif command_exists yum; then
        sudo yum update -y
        sudo yum autoremove -y
    elif command_exists brew; then
        brew update
        brew upgrade
        brew cleanup
    fi
}

# Function to update Docker images
update_docker_images() {
    echo "Updating Docker images..."
    
    # Pull latest images
    docker-compose pull
    
    # Rebuild images
    docker-compose build --no-cache
    
    # Verify new images
    docker-compose up -d --no-deps
}

# Function to run tests after updates
run_tests() {
    echo "Running tests after updates..."
    
    # Run Rust tests
    cargo test
    
    # Run integration tests if they exist
    if [ -f "tests/integration_tests.rs" ]; then
        cargo test --test integration_tests
    fi
}

# Function to rollback if needed
rollback() {
    echo "Rolling back updates..."
    timestamp=$(date +%Y%m%d_%H%M%S)
    
    # Restore Cargo.lock
    cp "$BACKUP_DIR/Cargo.lock.$timestamp" Cargo.lock
    
    # Restore package.json if it exists
    if [ -f "$BACKUP_DIR/package.json.$timestamp" ]; then
        cp "$BACKUP_DIR/package.json.$timestamp" package.json
    fi
    
    # Restore Docker image
    docker load < "$BACKUP_DIR/messaging-app.$timestamp.tar"
    
    # Rebuild and restart
    docker-compose up -d --build
}

# Function to set up automated updates
setup_automated_updates() {
    echo "Setting up automated updates..."
    
    # Create cron job
    (crontab -l 2>/dev/null; echo "$CRON_JOB $(pwd)/update_dependencies.sh >> $LOG_DIR/updates.log 2>&1") | crontab -
}

# Function to send update report
send_update_report() {
    echo "Sending update report..."
    
    # Create report
    cat > "$LOG_DIR/update_report_$(date +%Y%m%d).md" << EOL
# Dependency Update Report - $(date)

## Rust Dependencies
$(cargo outdated)

## System Packages
$(if command_exists apt-get; then apt list --upgradable; elif command_exists yum; then yum check-update; elif command_exists brew; then brew outdated; fi)

## Docker Images
$(docker images --format "{{.Repository}}:{{.Tag}}")

## Test Results
$(cargo test -- --nocapture)

## Recommendations
1. Review updated dependencies
2. Monitor application performance
3. Check for any new security advisories
4. Update documentation if needed
EOL

    # Send report via email (if configured)
    if [ -f "/usr/bin/mail" ]; then
        cat "$LOG_DIR/update_report_$(date +%Y%m%d).md" | mail -s "Dependency Update Report - $(date)" admin@yourdomain.com
    fi
}

# Main execution
echo "Starting dependency updates..."

# Create directories
create_directories

# Backup current state
backup_dependencies

# Update dependencies
update_rust_dependencies
update_system_packages
update_docker_images

# Run tests
run_tests

# Check if tests passed
if [ $? -eq 0 ]; then
    echo "Tests passed, proceeding with updates..."
    send_update_report
else
    echo "Tests failed, rolling back..."
    rollback
    exit 1
fi

# Setup automated updates
setup_automated_updates

echo "Dependency updates completed successfully!"
echo "Logs are available in $LOG_DIR"
echo "Backups are available in $BACKUP_DIR" 