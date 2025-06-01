# Operations Guide

## System Requirements

### Hardware Requirements

1. **Production Servers**
   - CPU: 4+ cores
   - RAM: 16GB minimum
   - Storage: 100GB+ SSD
   - Network: 1Gbps

2. **Database Servers**
   - CPU: 8+ cores
   - RAM: 32GB minimum
   - Storage: 500GB+ SSD
   - Network: 1Gbps

3. **Cache Servers**
   - CPU: 4+ cores
   - RAM: 16GB minimum
   - Storage: 50GB+ SSD
   - Network: 1Gbps

### Software Requirements

1. **Operating System**
   - Ubuntu 20.04 LTS
   - CentOS 8
   - RHEL 8

2. **Dependencies**
   - Docker 20.10+
   - Docker Compose 2.0+
   - PostgreSQL 14+
   - Redis 6+
   - Nginx 1.18+

## Deployment

### Production Deployment

1. **Infrastructure Setup**
   ```bash
   # Create deployment directory
   mkdir -p /opt/messaging-app
   cd /opt/messaging-app

   # Clone repository
   git clone https://github.com/your-org/messaging-app.git .

   # Set up environment
   cp .env.example .env
   # Edit .env with production values
   ```

2. **Database Setup**
   ```bash
   # Initialize database
   docker-compose up -d postgres
   sleep 10  # Wait for database to start

   # Run migrations
   docker-compose run --rm backend cargo run --bin migrate
   ```

3. **Service Deployment**
   ```bash
   # Build and start services
   docker-compose -f docker-compose.prod.yml up -d

   # Verify deployment
   curl http://localhost:8080/health
   ```

### Scaling

1. **Horizontal Scaling**
   ```bash
   # Scale API servers
   docker-compose -f docker-compose.prod.yml up -d --scale api=3

   # Scale WebSocket servers
   docker-compose -f docker-compose.prod.yml up -d --scale websocket=2
   ```

2. **Load Balancer Configuration**
   ```nginx
   upstream api_servers {
       server api1:8080;
       server api2:8080;
       server api3:8080;
   }

   upstream ws_servers {
       server ws1:8081;
       server ws2:8081;
   }

   server {
       listen 80;
       server_name api.example.com;

       location / {
           proxy_pass http://api_servers;
       }

       location /ws {
           proxy_pass http://ws_servers;
           proxy_http_version 1.1;
           proxy_set_header Upgrade $http_upgrade;
           proxy_set_header Connection "upgrade";
       }
   }
   ```

## Monitoring

### Metrics Collection

1. **Prometheus Configuration**
   ```yaml
   global:
     scrape_interval: 15s
     evaluation_interval: 15s

   scrape_configs:
     - job_name: 'messaging-app'
       static_configs:
         - targets: ['api1:8080', 'api2:8080', 'api3:8080']
       metrics_path: '/metrics'

     - job_name: 'websocket'
       static_configs:
         - targets: ['ws1:8081', 'ws2:8081']
       metrics_path: '/metrics'
   ```

2. **Grafana Dashboards**
   ```json
   {
     "dashboard": {
       "panels": [
         {
           "title": "Request Rate",
           "type": "graph",
           "targets": [
             {
               "expr": "rate(http_requests_total[5m])",
               "legendFormat": "{{method}} {{path}}"
             }
           ]
         }
       ]
     }
   }
   ```

### Alerting

1. **Alert Rules**
   ```yaml
   groups:
     - name: critical
       rules:
         - alert: HighErrorRate
           expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) * 100 > 5
           for: 5m
           labels:
             severity: critical
           annotations:
             summary: High error rate
             description: Error rate is above 5% for 5 minutes
   ```

2. **Notification Channels**
   ```yaml
   receivers:
     - name: 'team-alerts'
       email_configs:
         - to: 'team@example.com'
       slack_configs:
         - channel: '#alerts'
           send_resolved: true
   ```

## Backup and Recovery

### Backup Procedures

1. **Database Backup**
   ```bash
   # Create backup script
   cat > /opt/messaging-app/scripts/backup.sh << 'EOF'
   #!/bin/bash
   TIMESTAMP=$(date +%Y%m%d_%H%M%S)
   BACKUP_DIR="/backups/database"
   
   # Create backup
   pg_dump -h $DB_HOST -U $DB_USER $DB_NAME > $BACKUP_DIR/backup_$TIMESTAMP.sql
   
   # Compress backup
   gzip $BACKUP_DIR/backup_$TIMESTAMP.sql
   
   # Upload to S3
   aws s3 cp $BACKUP_DIR/backup_$TIMESTAMP.sql.gz s3://your-backup-bucket/
   
   # Clean up old backups
   find $BACKUP_DIR -type f -mtime +7 -delete
   EOF
   
   chmod +x /opt/messaging-app/scripts/backup.sh
   ```

2. **Media Backup**
   ```bash
   # Backup media files
   aws s3 sync /opt/messaging-app/media s3://your-backup-bucket/media/
   ```

### Recovery Procedures

1. **Database Recovery**
   ```bash
   # Restore from backup
   gunzip -c backup_20240320_120000.sql.gz | psql -h $DB_HOST -U $DB_USER $DB_NAME
   ```

2. **Media Recovery**
   ```bash
   # Restore media files
   aws s3 sync s3://your-backup-bucket/media/ /opt/messaging-app/media/
   ```

## Security

### Security Hardening

1. **Firewall Configuration**
   ```bash
   # Configure UFW
   ufw default deny incoming
   ufw default allow outgoing
   ufw allow ssh
   ufw allow http
   ufw allow https
   ufw enable
   ```

2. **SSL Configuration**
   ```nginx
   server {
       listen 443 ssl;
       server_name api.example.com;

       ssl_certificate /etc/letsencrypt/live/api.example.com/fullchain.pem;
       ssl_certificate_key /etc/letsencrypt/live/api.example.com/privkey.pem;
       ssl_protocols TLSv1.2 TLSv1.3;
       ssl_ciphers HIGH:!aNULL:!MD5;
   }
   ```

### Security Monitoring

1. **Log Analysis**
   ```bash
   # Set up log rotation
   cat > /etc/logrotate.d/messaging-app << 'EOF'
   /var/log/messaging-app/*.log {
       daily
       rotate 7
       compress
       delaycompress
       missingok
       notifempty
       create 0640 www-data www-data
   }
   EOF
   ```

2. **Security Scanning**
   ```bash
   # Run security scan
   docker-compose run --rm security-scanner
   ```

## Maintenance

### Regular Maintenance

1. **System Updates**
   ```bash
   # Update system
   apt update && apt upgrade -y

   # Update containers
   docker-compose pull
   docker-compose up -d
   ```

2. **Database Maintenance**
   ```sql
   -- Vacuum database
   VACUUM ANALYZE;

   -- Update statistics
   ANALYZE;
   ```

### Performance Tuning

1. **Database Tuning**
   ```conf
   # postgresql.conf
   max_connections = 200
   shared_buffers = 4GB
   effective_cache_size = 12GB
   maintenance_work_mem = 1GB
   checkpoint_completion_target = 0.9
   wal_buffers = 16MB
   default_statistics_target = 100
   random_page_cost = 1.1
   effective_io_concurrency = 200
   work_mem = 6553kB
   min_wal_size = 1GB
   max_wal_size = 4GB
   ```

2. **Redis Tuning**
   ```conf
   # redis.conf
   maxmemory 8gb
   maxmemory-policy allkeys-lru
   appendonly yes
   appendfsync everysec
   ```

## Disaster Recovery

### Recovery Procedures

1. **Service Recovery**
   ```bash
   # Check service status
   docker-compose ps

   # Restart failed services
   docker-compose restart api websocket

   # Check logs
   docker-compose logs --tail=100 api websocket
   ```

2. **Data Recovery**
   ```bash
   # Restore from latest backup
   ./scripts/restore.sh latest
   ```

### Business Continuity

1. **Failover Procedures**
   ```bash
   # Switch to backup database
   docker-compose -f docker-compose.prod.yml up -d postgres-backup

   # Update connection strings
   sed -i 's/DB_HOST=postgres/DB_HOST=postgres-backup/' .env
   ```

2. **Incident Response**
   ```bash
   # Incident response script
   cat > /opt/messaging-app/scripts/incident-response.sh << 'EOF'
   #!/bin/bash
   
   # Log incident
   echo "$(date) - Incident: $1" >> /var/log/incidents.log
   
   # Notify team
   curl -X POST $SLACK_WEBHOOK -d "{\"text\":\"Incident: $1\"}"
   
   # Take action based on incident type
   case $1 in
     "database_down")
       ./scripts/failover.sh
       ;;
     "high_error_rate")
       ./scripts/scale-up.sh
       ;;
   esac
   EOF
   ```

## Compliance

### Data Protection

1. **Data Retention**
   ```sql
   -- Retention policy
   CREATE POLICY retention_policy ON messages
   FOR DELETE
   USING (created_at < NOW() - INTERVAL '1 year');
   ```

2. **Data Export**
   ```bash
   # Export user data
   ./scripts/export-user-data.sh $USER_ID
   ```

### Audit Logging

1. **Audit Configuration**
   ```sql
   -- Enable audit logging
   CREATE EXTENSION pgaudit;
   
   ALTER SYSTEM SET pgaudit.log = 'all';
   ALTER SYSTEM SET pgaudit.log_relation = on;
   ```

2. **Log Analysis**
   ```bash
   # Analyze audit logs
   ./scripts/analyze-audit-logs.sh
   ``` 