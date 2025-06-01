#!/bin/bash

# Configuration
CONFIG_DIR="config"
SECURITY_CONFIG="$CONFIG_DIR/security.yaml"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to create security configuration
create_security_config() {
    echo "Creating security configuration..."
    mkdir -p "$CONFIG_DIR"

    cat > "$SECURITY_CONFIG" << EOL
# Security Headers Configuration
security_headers:
  # Content Security Policy
  content_security_policy: "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' https:;"
  
  # HTTP Strict Transport Security
  hsts:
    enabled: true
    max_age: 31536000  # 1 year
    include_subdomains: true
    preload: true
  
  # X-Frame-Options
  x_frame_options: "SAMEORIGIN"
  
  # X-Content-Type-Options
  x_content_type_options: "nosniff"
  
  # X-XSS-Protection
  x_xss_protection: "1; mode=block"
  
  # Referrer Policy
  referrer_policy: "strict-origin-when-cross-origin"
  
  # Permissions Policy
  permissions_policy: "camera=(), microphone=(), geolocation=(), payment=()"

# CORS Configuration
cors:
  allowed_origins:
    - "https://yourdomain.com"
    - "https://api.yourdomain.com"
  
  allowed_methods:
    - "GET"
    - "POST"
    - "PUT"
    - "DELETE"
    - "OPTIONS"
  
  allowed_headers:
    - "Authorization"
    - "Content-Type"
    - "X-Requested-With"
  
  exposed_headers:
    - "X-Total-Count"
    - "X-Rate-Limit-Remaining"
  
  allow_credentials: true
  max_age: 86400  # 24 hours

# Rate Limiting
rate_limiting:
  enabled: true
  requests_per_minute: 100
  burst_size: 50
  ip_based: true
  exclude_paths:
    - "/health"
    - "/metrics"

# JWT Configuration
jwt:
  algorithm: "HS256"
  access_token_expiry: 3600  # 1 hour
  refresh_token_expiry: 604800  # 7 days
  issuer: "messaging-app"
  audience: "messaging-app-users"

# Password Policy
password_policy:
  min_length: 12
  require_uppercase: true
  require_lowercase: true
  require_numbers: true
  require_special_chars: true
  max_age_days: 90
  prevent_reuse: 5  # Number of previous passwords to prevent reuse

# Session Security
session:
  secure: true
  http_only: true
  same_site: "Lax"
  max_age: 3600  # 1 hour
  rolling: true

# API Security
api_security:
  require_api_key: true
  api_key_header: "X-API-Key"
  api_key_length: 32
  api_key_prefix: "msg_"
EOL

    echo "Security configuration created at $SECURITY_CONFIG"
}

# Function to set up firewall rules
setup_firewall() {
    echo "Setting up firewall rules..."
    
    # Check if ufw is available
    if command_exists ufw; then
        # Allow SSH
        sudo ufw allow ssh
        
        # Allow HTTP
        sudo ufw allow 80/tcp
        
        # Allow HTTPS
        sudo ufw allow 443/tcp
        
        # Allow application port
        sudo ufw allow 3000/tcp
        
        # Enable firewall
        sudo ufw --force enable
        
        echo "Firewall rules configured"
    else
        echo "ufw not found, skipping firewall configuration"
    fi
}

# Function to set up fail2ban
setup_fail2ban() {
    echo "Setting up fail2ban..."
    
    # Check if fail2ban is available
    if command_exists fail2ban-server; then
        # Create fail2ban configuration
        sudo tee /etc/fail2ban/jail.local > /dev/null << EOL
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 5

[sshd]
enabled = true
port = ssh
filter = sshd
logpath = /var/log/auth.log
maxretry = 3

[nginx-http-auth]
enabled = true
filter = nginx-http-auth
port = http,https
logpath = /var/log/nginx/error.log

[messaging-app]
enabled = true
port = http,https
filter = messaging-app
logpath = /var/log/messaging-app/access.log
maxretry = 10
findtime = 600
bantime = 3600
EOL

        # Create custom filter
        sudo tee /etc/fail2ban/filter.d/messaging-app.conf > /dev/null << EOL
[Definition]
failregex = ^.*"POST /api/v1/auth/login".*"status":401.*$
            ^.*"POST /api/v1/auth/register".*"status":400.*$
ignoreregex =
EOL

        # Restart fail2ban
        sudo systemctl restart fail2ban
        echo "fail2ban configured"
    else
        echo "fail2ban not found, skipping fail2ban configuration"
    fi
}

# Main execution
echo "Starting security setup..."

# Create security configuration
create_security_config

# Setup firewall rules
setup_firewall

# Setup fail2ban
setup_fail2ban

echo "Security setup completed successfully!"
echo "Please update the following in $SECURITY_CONFIG:"
echo "1. Allowed origins in CORS configuration"
echo "2. JWT secret key"
echo "3. Rate limiting thresholds"
echo "4. Password policy requirements" 