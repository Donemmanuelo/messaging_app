#!/bin/bash

# Configuration
DOMAIN="your-domain.com"
EMAIL="admin@your-domain.com"
CERT_DIR="/etc/letsencrypt"
CERTBOT_IMAGE="certbot/certbot:latest"

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

# Function to obtain SSL certificate
obtain_certificate() {
    echo "Obtaining SSL certificate for $DOMAIN..."
    
    docker run --rm \
        -v $CERT_DIR:/etc/letsencrypt \
        -v /var/lib/letsencrypt:/var/lib/letsencrypt \
        -v /var/www/certbot:/var/www/certbot \
        $CERTBOT_IMAGE certonly \
        --webroot \
        --webroot-path=/var/www/certbot \
        --email $EMAIL \
        --agree-tos \
        --no-eff-email \
        -d $DOMAIN \
        --force-renewal
}

# Function to setup auto-renewal
setup_auto_renewal() {
    echo "Setting up automatic certificate renewal..."
    
    # Create renewal script
    cat > /etc/cron.d/certbot-renew << EOF
0 0 * * * root docker run --rm \
    -v $CERT_DIR:/etc/letsencrypt \
    -v /var/lib/letsencrypt:/var/lib/letsencrypt \
    -v /var/www/certbot:/var/www/certbot \
    $CERTBOT_IMAGE renew \
    --webroot \
    --webroot-path=/var/www/certbot \
    --quiet \
    && docker-compose restart nginx
EOF
    
    # Make the script executable
    chmod +x /etc/cron.d/certbot-renew
}

# Function to update Nginx configuration
update_nginx_config() {
    echo "Updating Nginx configuration..."
    
    # Create SSL configuration
    cat > nginx/ssl.conf << EOF
ssl_certificate $CERT_DIR/live/$DOMAIN/fullchain.pem;
ssl_certificate_key $CERT_DIR/live/$DOMAIN/privkey.pem;
ssl_trusted_certificate $CERT_DIR/live/$DOMAIN/chain.pem;

# SSL configuration
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:DHE-RSA-AES128-GCM-SHA256:DHE-RSA-AES256-GCM-SHA384;
ssl_prefer_server_ciphers off;

# HSTS (uncomment if you're sure)
# add_header Strict-Transport-Security "max-age=63072000" always;

# OCSP Stapling
ssl_stapling on;
ssl_stapling_verify on;
resolver 8.8.8.8 8.8.4.4 valid=300s;
resolver_timeout 5s;
EOF
    
    # Update main Nginx configuration
    cat > nginx/nginx.conf << EOF
user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log warn;
pid /var/run/nginx.pid;

events {
    worker_connections 1024;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;
    
    log_format main '\$remote_addr - \$remote_user [\$time_local] "\$request" '
                    '\$status \$body_bytes_sent "\$http_referer" '
                    '"\$http_user_agent" "\$http_x_forwarded_for"';
    
    access_log /var/log/nginx/access.log main;
    
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    
    # SSL configuration
    include /etc/nginx/ssl.conf;
    
    server {
        listen 80;
        server_name $DOMAIN;
        
        location /.well-known/acme-challenge/ {
            root /var/www/certbot;
        }
        
        location / {
            return 301 https://\$host\$request_uri;
        }
    }
    
    server {
        listen 443 ssl http2;
        server_name $DOMAIN;
        
        # SSL configuration
        include /etc/nginx/ssl.conf;
        
        location / {
            proxy_pass http://app:3000;
            proxy_http_version 1.1;
            proxy_set_header Upgrade \$http_upgrade;
            proxy_set_header Connection 'upgrade';
            proxy_set_header Host \$host;
            proxy_cache_bypass \$http_upgrade;
            proxy_set_header X-Real-IP \$remote_addr;
            proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto \$scheme;
        }
    }
}
EOF
}

# Main execution
echo "Starting SSL setup..."

# Check prerequisites
check_docker

# Create necessary directories
mkdir -p $CERT_DIR
mkdir -p /var/www/certbot
mkdir -p nginx

# Obtain certificate
obtain_certificate

# Setup auto-renewal
setup_auto_renewal

# Update Nginx configuration
update_nginx_config

echo "SSL setup completed successfully!"
echo "Please update your domain name and email in this script before running it."
echo "After running this script, make sure to:"
echo "1. Update your DNS records to point to your server"
echo "2. Start your Nginx container with the new configuration"
echo "3. Test your SSL configuration using SSL Labs (https://www.ssllabs.com/ssltest/)" 