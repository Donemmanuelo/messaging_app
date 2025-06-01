# Production Deployment Guide

This guide outlines the steps to deploy the messaging app in a production environment.

## Prerequisites

- Docker and Docker Compose installed
- Docker Swarm or Kubernetes cluster
- Domain name and SSL certificates
- Cloud provider account (AWS, GCP, Azure, etc.)

## Environment Setup

1. Create a `.env.production` file with the following variables:

```env
# Database Configuration
POSTGRES_USER=messaging_app
POSTGRES_PASSWORD=<secure_password>
POSTGRES_DB=messaging_app

# Redis Configuration
REDIS_URL=redis://redis:6379

# Application Configuration
APP_ENV=production
RUST_LOG=info
FRONTEND_URL=https://your-frontend-domain.com

# Docker Configuration
DOCKERHUB_USERNAME=your-dockerhub-username

# Grafana Configuration
GRAFANA_PASSWORD=<secure_password>

# Security
JWT_SECRET=<secure_random_string>
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_DURATION=60

# Cloudinary Configuration
CLOUDINARY_CLOUD_NAME=your_cloud_name
CLOUDINARY_API_KEY=your_api_key
CLOUDINARY_API_SECRET=your_api_secret
```

## Deployment Steps

1. **Initialize Docker Swarm**:
   ```bash
   docker swarm init
   ```

2. **Create Docker Secrets**:
   ```bash
   echo "<secure_password>" | docker secret create postgres_password -
   echo "<secure_password>" | docker secret create grafana_password -
   echo "<secure_random_string>" | docker secret create jwt_secret -
   ```

3. **Deploy the Stack**:
   ```bash
   docker stack deploy -c docker-compose.prod.yml messaging-app
   ```

4. **Verify Deployment**:
   ```bash
   docker service ls
   docker stack ps messaging-app
   ```

## Monitoring Setup

1. **Access Grafana**:
   - Open `http://your-domain:3000`
   - Default credentials: admin / password (set in .env.production)
   - Add Prometheus as a data source (URL: http://prometheus:9090)

2. **Import Dashboards**:
   - Import the following dashboards in Grafana:
     - Node Exporter Full
     - Prometheus 2.0 Overview
     - Docker Swarm and Container Overview

## Security Considerations

1. **SSL/TLS**:
   - Set up SSL termination using a reverse proxy (e.g., Traefik, Nginx)
   - Configure automatic SSL certificate renewal

2. **Network Security**:
   - Configure firewall rules
   - Use private networks for internal communication
   - Implement proper access controls

3. **Data Security**:
   - Enable database encryption
   - Implement regular backups
   - Use secure secrets management

## Backup and Recovery

1. **Database Backups**:
   ```bash
   # Create backup
   docker exec -t messaging-app_db_1 pg_dump -U messaging_app > backup.sql

   # Restore from backup
   cat backup.sql | docker exec -i messaging-app_db_1 psql -U messaging_app
   ```

2. **Volume Backups**:
   ```bash
   # Backup volumes
   docker run --rm -v messaging-app_postgres_data:/source -v $(pwd):/backup alpine tar -czf /backup/postgres_data.tar.gz -C /source .

   # Restore volumes
   docker run --rm -v messaging-app_postgres_data:/target -v $(pwd):/backup alpine sh -c "cd /target && tar -xzf /backup/postgres_data.tar.gz"
   ```

## Scaling

1. **Horizontal Scaling**:
   ```bash
   docker service scale messaging-app_app=4
   ```

2. **Resource Limits**:
   - Adjust resource limits in docker-compose.prod.yml
   - Monitor resource usage in Grafana

## Maintenance

1. **Update Application**:
   ```bash
   # Pull new images
   docker stack deploy -c docker-compose.prod.yml messaging-app

   # Remove old images
   docker image prune -f
   ```

2. **Database Maintenance**:
   ```bash
   # Vacuum database
   docker exec -t messaging-app_db_1 psql -U messaging_app -c "VACUUM FULL;"
   ```

## Troubleshooting

1. **View Logs**:
   ```bash
   docker service logs messaging-app_app
   docker service logs messaging-app_db
   docker service logs messaging-app_redis
   ```

2. **Check Service Health**:
   ```bash
   curl http://your-domain:3000/health
   curl http://your-domain:3000/metrics
   ```

## Disaster Recovery

1. **Create Recovery Plan**:
   - Document recovery procedures
   - Test recovery process regularly
   - Maintain backup rotation

2. **Recovery Steps**:
   - Restore from latest backup
   - Verify data integrity
   - Test application functionality

## Support

For issues and support:
1. Check the logs and metrics
2. Review the documentation
3. Contact the development team
4. Open an issue on GitHub 