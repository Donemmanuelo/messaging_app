# Troubleshooting Guide

## Common Issues and Solutions

### Database Issues

#### Connection Refused
```bash
Error: Connection refused (os error 111)
```

Solutions:
1. Check if PostgreSQL is running:
```bash
sudo systemctl status postgresql
```

2. Verify database credentials in `.env`:
```env
DB_HOST=postgres
DB_PORT=5432
DB_NAME=messaging_app
DB_USER=postgres
DB_PASSWORD=your_password
```

3. Check database logs:
```bash
docker-compose logs postgres
```

#### Migration Errors
```bash
Error: Migration failed: relation "users" already exists
```

Solutions:
1. Reset database:
```bash
docker-compose down -v
docker-compose up -d
```

2. Run migrations manually:
```bash
cargo run --bin migrate
```

### Redis Issues

#### Connection Timeout
```bash
Error: Connection timed out
```

Solutions:
1. Check Redis status:
```bash
docker-compose ps redis
```

2. Verify Redis configuration:
```env
REDIS_HOST=redis
REDIS_PORT=6379
```

3. Check Redis logs:
```bash
docker-compose logs redis
```

### WebSocket Issues

#### Connection Failed
```javascript
WebSocket connection to 'wss://api.example.com/ws' failed
```

Solutions:
1. Check SSL certificate:
```bash
./scripts/check_ssl.sh
```

2. Verify WebSocket endpoint:
```bash
curl -v wss://api.example.com/ws
```

3. Check Nginx configuration:
```bash
nginx -t
```

### Media Upload Issues

#### Upload Failed
```bash
Error: File size exceeds limit
```

Solutions:
1. Check file size limit in `.env`:
```env
MAX_FILE_SIZE=10485760  # 10MB
```

2. Verify storage permissions:
```bash
sudo chown -R www-data:www-data /app/media
```

3. Check disk space:
```bash
df -h
```

### Performance Issues

#### High Latency
```bash
Warning: Request latency > 1s
```

Solutions:
1. Check database performance:
```sql
EXPLAIN ANALYZE SELECT * FROM messages WHERE conversation_id = 'conv_123';
```

2. Monitor Redis memory:
```bash
redis-cli info memory
```

3. Check system resources:
```bash
docker stats
```

### Security Issues

#### Rate Limiting
```json
{
    "error": "RateLimitExceeded",
    "message": "Too many requests"
}
```

Solutions:
1. Adjust rate limits in configuration:
```env
RATE_LIMIT_REQUESTS=60
RATE_LIMIT_WINDOW=60
```

2. Check for abuse:
```bash
tail -f /var/log/nginx/access.log
```

#### Authentication Failures
```json
{
    "error": "Unauthorized",
    "message": "Invalid token"
}
```

Solutions:
1. Verify JWT secret:
```env
JWT_SECRET=your_jwt_secret
```

2. Check token expiration:
```env
JWT_EXPIRATION=86400
```

### Monitoring Issues

#### Prometheus Scrape Failures
```bash
Error: Failed to scrape target
```

Solutions:
1. Check target configuration:
```yaml
- job_name: 'messaging-app'
  static_configs:
    - targets: ['localhost:8080']
```

2. Verify metrics endpoint:
```bash
curl http://localhost:8080/metrics
```

#### Grafana Dashboard Issues
```bash
Error: Failed to load dashboard
```

Solutions:
1. Check data source:
```bash
curl http://localhost:9090/api/v1/query?query=up
```

2. Verify dashboard JSON:
```bash
cat /etc/grafana/provisioning/dashboards/messaging-app.json
```

### Backup Issues

#### Backup Failed
```bash
Error: Failed to create backup
```

Solutions:
1. Check storage space:
```bash
df -h /backups
```

2. Verify backup script:
```bash
./scripts/backup.sh --verbose
```

3. Check cloud credentials:
```bash
aws s3 ls s3://your-backup-bucket
```

### Deployment Issues

#### Container Startup Failed
```bash
Error: Container failed to start
```

Solutions:
1. Check container logs:
```bash
docker-compose logs backend
```

2. Verify environment variables:
```bash
docker-compose config
```

3. Check resource limits:
```bash
docker stats
```

## Debugging Tools

### Logging
```bash
# View application logs
docker-compose logs -f backend

# View Nginx logs
docker-compose logs -f nginx

# View database logs
docker-compose logs -f postgres
```

### Monitoring
```bash
# Check Prometheus targets
curl http://localhost:9090/api/v1/targets

# Check Grafana health
curl http://localhost:3000/api/health
```

### Network
```bash
# Check container network
docker network inspect messaging-app_default

# Test internal connectivity
docker-compose exec backend ping postgres
```

### Performance
```bash
# Profile application
cargo flamegraph

# Monitor system resources
htop
```

## Recovery Procedures

### Database Recovery
```bash
# Restore from backup
./scripts/restore.sh --database

# Verify data
cargo run --bin verify-data
```

### Application Recovery
```bash
# Rollback to previous version
./scripts/deploy.sh --rollback

# Verify deployment
curl http://localhost:8080/health
```

### Media Recovery
```bash
# Restore media files
./scripts/restore.sh --media

# Verify media integrity
cargo run --bin verify-media
```

## Support

For additional support:
1. Check the [GitHub Issues](https://github.com/yourusername/messaging-app/issues)
2. Review the [Documentation](https://github.com/yourusername/messaging-app/docs)
3. Contact the development team at support@example.com 