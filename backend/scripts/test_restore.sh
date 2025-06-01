#!/bin/bash

# Configuration
BACKUP_DIR="/backups"
RESTORE_DIR="/tmp/restore_test"
DATE=$(date +%Y%m%d_%H%M%S)

# Create restore directory
mkdir -p $RESTORE_DIR

# Function to check if a file exists
file_exists() {
    [ -f "$1" ]
}

# Function to restore database
restore_database() {
    local backup_file=$1
    echo "Restoring database from $backup_file..."
    
    # Stop the application
    docker-compose stop app
    
    # Restore the database
    gunzip -c $backup_file | docker exec -i messaging-app_db_1 psql -U messaging_app
    
    # Start the application
    docker-compose start app
    
    echo "Database restore completed"
}

# Function to restore volumes
restore_volumes() {
    local postgres_backup=$1
    local redis_backup=$2
    
    echo "Restoring volumes..."
    
    # Stop the services
    docker-compose stop db redis
    
    # Restore PostgreSQL volume
    if file_exists $postgres_backup; then
        echo "Restoring PostgreSQL volume..."
        docker run --rm \
            -v messaging-app_postgres_data:/target \
            -v $BACKUP_DIR:/backup \
            alpine sh -c "cd /target && tar -xzf /backup/$(basename $postgres_backup)"
    fi
    
    # Restore Redis volume
    if file_exists $redis_backup; then
        echo "Restoring Redis volume..."
        docker run --rm \
            -v messaging-app_redis_data:/target \
            -v $BACKUP_DIR:/backup \
            alpine sh -c "cd /target && tar -xzf /backup/$(basename $redis_backup)"
    fi
    
    # Start the services
    docker-compose start db redis
    
    echo "Volume restore completed"
}

# Function to verify restore
verify_restore() {
    echo "Verifying restore..."
    
    # Check if database is accessible
    if docker exec messaging-app_db_1 pg_isready -U messaging_app; then
        echo "Database is accessible"
    else
        echo "Database is not accessible"
        return 1
    fi
    
    # Check if Redis is accessible
    if docker exec messaging-app_redis_1 redis-cli ping; then
        echo "Redis is accessible"
    else
        echo "Redis is not accessible"
        return 1
    fi
    
    # Check if application is running
    if curl -s http://localhost:3000/health | grep -q "healthy"; then
        echo "Application is healthy"
    else
        echo "Application is not healthy"
        return 1
    fi
    
    echo "Restore verification completed successfully"
    return 0
}

# Main restore process
echo "Starting restore test..."

# Find the latest backup files
LATEST_DB_BACKUP=$(ls -t $BACKUP_DIR/db_backup_*.sql.gz | head -n1)
LATEST_POSTGRES_BACKUP=$(ls -t $BACKUP_DIR/postgres_data_*.tar.gz | head -n1)
LATEST_REDIS_BACKUP=$(ls -t $BACKUP_DIR/redis_data_*.tar.gz | head -n1)

if [ -z "$LATEST_DB_BACKUP" ] || [ -z "$LATEST_POSTGRES_BACKUP" ] || [ -z "$LATEST_REDIS_BACKUP" ]; then
    echo "No backup files found"
    exit 1
fi

# Create a snapshot of current state
echo "Creating snapshot of current state..."
docker-compose exec db pg_dump -U messaging_app > $RESTORE_DIR/pre_restore_$DATE.sql

# Perform restore
restore_database $LATEST_DB_BACKUP
restore_volumes $LATEST_POSTGRES_BACKUP $LATEST_REDIS_BACKUP

# Verify restore
if verify_restore; then
    echo "Restore test completed successfully"
else
    echo "Restore test failed"
    # Restore from snapshot
    echo "Restoring from snapshot..."
    docker-compose exec -T db psql -U messaging_app < $RESTORE_DIR/pre_restore_$DATE.sql
    exit 1
fi

# Cleanup
rm -rf $RESTORE_DIR

echo "Restore test process completed" 