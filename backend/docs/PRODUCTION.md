# Production Deployment Guide

## Prerequisites

- Docker and Docker Compose installed
- Domain name pointing to your server
- Sentry account for error tracking
- SSL certificate (Let's Encrypt)

## Initial Setup

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd messaging-app
   ```

2. Run the production setup script:
   ```bash
   ./scripts/setup_production.sh
   ```

3. Update the environment variables in `/opt/messaging-app/.env.production`:
   - Set a secure `JWT_SECRET`
   - Add your `SENTRY_DSN`
   - Update database credentials if needed

## SSL Configuration

1. Update the domain name in `scripts/setup_ssl.sh`
2. Run the SSL setup script:
   ```bash
   ./scripts/setup_ssl.sh
   ```

## Database Setup

1. Run initial migrations:
   ```bash
   ./scripts/migrate.sh up
   ```

2. Verify database connection:
   ```bash
   ./scripts/migrate.sh status
   ```

## Monitoring Setup

1. Start the monitoring stack:
   ```bash
   ./scripts/setup_monitoring.sh
   ```

2. Access Grafana at `http://your-domain:3001`
   - Default credentials: admin/admin
   - Change the default password immediately

3. Configure alerts in Grafana:
   - Import the dashboard from `grafana/dashboards/messaging_app.json`
   - Set up notification channels
   - Configure alert rules

## Backup Configuration

1. Set up automated backups:
   ```bash
   ./scripts/setup_cron.sh
   ```

2. Test backup and restore:
   ```bash
   ./scripts/test_restore.sh
   ```

## Security Checklist

- [ ] Update all default passwords
- [ ] Configure firewall rules
- [ ] Enable HTTPS only
- [ ] Set up rate limiting
- [ ] Configure CORS
- [ ] Set up security headers
- [ ] Enable HSTS
- [ ] Configure backup retention
- [ ] Set up monitoring alerts
- [ ] Configure error tracking

## Deployment Process

1. Build and deploy:
   ```bash
   ./scripts/deploy.sh
   ```

2. Verify deployment:
   - Check application logs: `journalctl -u messaging-app`
   - Monitor metrics in Grafana
   - Test critical functionality
   - Verify SSL configuration

## Monitoring and Maintenance

### Regular Tasks

1. Monitor logs and metrics:
   - Application logs: `journalctl -u messaging-app`
   - Docker logs: `docker-compose logs`
   - Grafana dashboards

2. Review and rotate logs:
   - Logs are automatically rotated daily
   - Kept for 14 days
   - Compressed after rotation

3. Check backup status:
   - Verify backup completion
   - Test restore process
   - Clean up old backups

4. Update dependencies:
   - Check for security updates
   - Update Docker images
   - Update Rust dependencies

5. SSL certificate renewal:
   - Automatic renewal with Let's Encrypt
   - Verify renewal status

### Troubleshooting

1. Application Issues:
   - Check application logs
   - Review Sentry for errors
   - Verify database connection
   - Check Redis connection

2. Performance Issues:
   - Monitor Grafana dashboards
   - Check resource usage
   - Review rate limiting
   - Analyze slow queries

3. Backup Issues:
   - Verify backup script execution
   - Check backup directory permissions
   - Test restore process
   - Verify cloud storage uploads

## API Documentation

Access the API documentation at:
- Swagger UI: `https://your-domain/api/docs`
- OpenAPI spec: `https://your-domain/api/docs/openapi.json`

## Support and Maintenance

1. Regular maintenance:
   - Daily: Check logs and alerts
   - Weekly: Review metrics and performance
   - Monthly: Security updates and dependency checks
   - Quarterly: Full backup restore test

2. Emergency procedures:
   - Database restore: `./scripts/restore.sh`
   - Rollback deployment: `./scripts/deploy.sh rollback`
   - Emergency backup: `./scripts/backup.sh emergency`

3. Contact information:
   - System administrator: [contact details]
   - Emergency support: [contact details]
   - Security issues: [contact details]

## Scaling Considerations

1. Database scaling:
   - Connection pool size
   - Query optimization
   - Index management

2. Application scaling:
   - Horizontal scaling with Docker
   - Load balancer configuration
   - Session management

3. Media storage:
   - Cloud storage integration
   - CDN configuration
   - Cache management

## Disaster Recovery

1. Backup strategy:
   - Daily full backups
   - Point-in-time recovery
   - Cloud storage replication

2. Recovery procedures:
   - Database restore
   - Application rollback
   - Configuration recovery

3. Business continuity:
   - Failover procedures
   - Data replication
   - Service redundancy 