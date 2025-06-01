#!/bin/bash

# Configuration
SCAN_DIR="security_scans"
LOG_DIR="/var/log/messaging-app/security"
CRON_JOB="0 0 * * *"  # Daily at midnight

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to create necessary directories
create_directories() {
    echo "Creating necessary directories..."
    sudo mkdir -p "$SCAN_DIR"
    sudo mkdir -p "$LOG_DIR"
    sudo chown -R $USER:$USER "$SCAN_DIR"
    sudo chown -R $USER:$USER "$LOG_DIR"
}

# Function to run dependency vulnerability scan
scan_dependencies() {
    echo "Scanning dependencies for vulnerabilities..."
    
    # Check for cargo-audit
    if command_exists cargo-audit; then
        cargo audit --json > "$SCAN_DIR/dependencies_$(date +%Y%m%d).json"
    else
        echo "Installing cargo-audit..."
        cargo install cargo-audit
        cargo audit --json > "$SCAN_DIR/dependencies_$(date +%Y%m%d).json"
    fi
}

# Function to run container vulnerability scan
scan_containers() {
    echo "Scanning containers for vulnerabilities..."
    
    # Check for trivy
    if command_exists trivy; then
        trivy image messaging-app:latest --format json --output "$SCAN_DIR/containers_$(date +%Y%m%d).json"
    else
        echo "Installing trivy..."
        curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh -s -- -b /usr/local/bin
        trivy image messaging-app:latest --format json --output "$SCAN_DIR/containers_$(date +%Y%m%d).json"
    fi
}

# Function to run network security scan
scan_network() {
    echo "Scanning network security..."
    
    # Check for nmap
    if command_exists nmap; then
        nmap -sV -sC -oX "$SCAN_DIR/network_$(date +%Y%m%d).xml" localhost
    else
        echo "Installing nmap..."
        sudo apt-get update && sudo apt-get install -y nmap
        nmap -sV -sC -oX "$SCAN_DIR/network_$(date +%Y%m%d).xml" localhost
    fi
}

# Function to check SSL/TLS configuration
check_ssl() {
    echo "Checking SSL/TLS configuration..."
    
    # Check for testssl.sh
    if [ -f "/usr/local/bin/testssl.sh" ]; then
        /usr/local/bin/testssl.sh --jsonfile "$SCAN_DIR/ssl_$(date +%Y%m%d).json" https://yourdomain.com
    else
        echo "Installing testssl.sh..."
        git clone https://github.com/drwetter/testssl.sh.git
        sudo cp testssl.sh/testssl.sh /usr/local/bin/
        /usr/local/bin/testssl.sh --jsonfile "$SCAN_DIR/ssl_$(date +%Y%m%d).json" https://yourdomain.com
    fi
}

# Function to set up automated scanning
setup_automated_scanning() {
    echo "Setting up automated security scanning..."
    
    # Create cron job
    (crontab -l 2>/dev/null; echo "$CRON_JOB $(pwd)/security_scan.sh >> $LOG_DIR/security_scan.log 2>&1") | crontab -
}

# Function to send security report
send_security_report() {
    echo "Sending security report..."
    
    # Create report
    cat > "$SCAN_DIR/security_report_$(date +%Y%m%d).md" << EOL
# Security Scan Report - $(date)

## Dependencies
$(cat "$SCAN_DIR/dependencies_$(date +%Y%m%d).json")

## Containers
$(cat "$SCAN_DIR/containers_$(date +%Y%m%d).json")

## Network
$(cat "$SCAN_DIR/network_$(date +%Y%m%d).xml")

## SSL/TLS
$(cat "$SCAN_DIR/ssl_$(date +%Y%m%d).json")

## Recommendations
1. Review and update dependencies
2. Patch container vulnerabilities
3. Address network security findings
4. Update SSL/TLS configuration if needed
EOL

    # Send report via email (if configured)
    if [ -f "/usr/bin/mail" ]; then
        cat "$SCAN_DIR/security_report_$(date +%Y%m%d).md" | mail -s "Security Scan Report - $(date)" admin@yourdomain.com
    fi
}

# Main execution
echo "Starting security scan..."

# Create directories
create_directories

# Run scans
scan_dependencies
scan_containers
scan_network
check_ssl

# Send report
send_security_report

# Setup automated scanning
setup_automated_scanning

echo "Security scan completed successfully!"
echo "Reports are available in $SCAN_DIR"
echo "Logs are available in $LOG_DIR" 