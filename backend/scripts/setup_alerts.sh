#!/bin/bash

# Configuration
GRAFANA_API_URL="http://localhost:3001/api"
GRAFANA_USER="admin"
GRAFANA_PASSWORD="admin"  # Should be changed after first login
ALERT_RULES_DIR="grafana/provisioning/alerting"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if curl is available
check_curl() {
    if ! command_exists curl; then
        echo "Error: curl is not installed"
        exit 1
    fi
}

# Function to create alert rules directory
create_alert_rules_dir() {
    echo "Creating alert rules directory..."
    mkdir -p "$ALERT_RULES_DIR"
}

# Function to create alert rules
create_alert_rules() {
    echo "Creating alert rules..."

    # High CPU Usage Alert
    cat > "$ALERT_RULES_DIR/cpu_usage.yaml" << EOL
apiVersion: 1
groups:
  - name: System
    folder: System
    interval: 1m
    rules:
      - name: High CPU Usage
        condition: avg(rate(process_cpu_seconds_total[5m])) * 100 > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: High CPU usage detected
          description: CPU usage is above 80% for 5 minutes
EOL

    # High Memory Usage Alert
    cat > "$ALERT_RULES_DIR/memory_usage.yaml" << EOL
apiVersion: 1
groups:
  - name: System
    folder: System
    interval: 1m
    rules:
      - name: High Memory Usage
        condition: (node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100 > 85
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: High memory usage detected
          description: Memory usage is above 85% for 5 minutes
EOL

    # Database Connection Alert
    cat > "$ALERT_RULES_DIR/database_connections.yaml" << EOL
apiVersion: 1
groups:
  - name: Database
    folder: Database
    interval: 1m
    rules:
      - name: High Database Connections
        condition: pg_stat_activity_count > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: High number of database connections
          description: Database connection count is above 80 for 5 minutes
EOL

    # Redis Memory Alert
    cat > "$ALERT_RULES_DIR/redis_memory.yaml" << EOL
apiVersion: 1
groups:
  - name: Redis
    folder: Redis
    interval: 1m
    rules:
      - name: High Redis Memory Usage
        condition: redis_memory_used_bytes / redis_memory_max_bytes * 100 > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: High Redis memory usage
          description: Redis memory usage is above 80% for 5 minutes
EOL

    # API Error Rate Alert
    cat > "$ALERT_RULES_DIR/api_errors.yaml" << EOL
apiVersion: 1
groups:
  - name: API
    folder: API
    interval: 1m
    rules:
      - name: High API Error Rate
        condition: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) * 100 > 5
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: High API error rate
          description: API error rate is above 5% for 5 minutes
EOL

    # Message Queue Alert
    cat > "$ALERT_RULES_DIR/message_queue.yaml" << EOL
apiVersion: 1
groups:
  - name: Messaging
    folder: Messaging
    interval: 1m
    rules:
      - name: Message Queue Backlog
        condition: message_queue_size > 1000
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: Message queue backlog detected
          description: Message queue size is above 1000 for 10 minutes
EOL
}

# Function to create notification channels
create_notification_channels() {
    echo "Creating notification channels..."

    # Email notification channel
    curl -X POST "$GRAFANA_API_URL/alert-notifications" \
        -H "Content-Type: application/json" \
        -u "$GRAFANA_USER:$GRAFANA_PASSWORD" \
        -d '{
            "name": "Email Alerts",
            "type": "email",
            "isDefault": true,
            "settings": {
                "addresses": "admin@yourdomain.com"
            }
        }'

    # Slack notification channel
    curl -X POST "$GRAFANA_API_URL/alert-notifications" \
        -H "Content-Type: application/json" \
        -u "$GRAFANA_USER:$GRAFANA_PASSWORD" \
        -d '{
            "name": "Slack Alerts",
            "type": "slack",
            "isDefault": false,
            "settings": {
                "url": "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK",
                "channel": "#alerts"
            }
        }'
}

# Main execution
echo "Starting alert setup..."

# Check prerequisites
check_curl

# Create alert rules directory
create_alert_rules_dir

# Create alert rules
create_alert_rules

# Create notification channels
create_notification_channels

echo "Alert setup completed successfully!"
echo "Please update the following:"
echo "1. Grafana admin password"
echo "2. Email notification addresses"
echo "3. Slack webhook URL"
echo "4. Alert thresholds if needed" 