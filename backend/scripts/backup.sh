#!/bin/bash

# Configuration
BACKUP_DIR="/backups"
DATE=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=7
CLOUD_BACKUP_BUCKET="your-backup-bucket"
CLOUD_BACKUP_PATH="messaging-app"

# Create backup directory if it doesn't exist
mkdir -p $BACKUP_DIR

# Database backup
echo "Creating database backup..."
docker exec messaging-app_db_1 pg_dump -U messaging_app > $BACKUP_DIR/db_backup_$DATE.sql

# Compress database backup
gzip $BACKUP_DIR/db_backup_$DATE.sql

# Volume backups
echo "Creating volume backups..."

# PostgreSQL volume
docker run --rm \
    -v messaging-app_postgres_data:/source \
    -v $BACKUP_DIR:/backup \
    alpine tar -czf /backup/postgres_data_$DATE.tar.gz -C /source .

# Redis volume
docker run --rm \
    -v messaging-app_redis_data:/source \
    -v $BACKUP_DIR:/backup \
    alpine tar -czf /backup/redis_data_$DATE.tar.gz -C /source .

# Clean up old backups
echo "Cleaning up old backups..."
find $BACKUP_DIR -type f -mtime +$RETENTION_DAYS -delete

# Verify backups
echo "Verifying backups..."
if [ -f "$BACKUP_DIR/db_backup_$DATE.sql.gz" ] && \
   [ -f "$BACKUP_DIR/postgres_data_$DATE.tar.gz" ] && \
   [ -f "$BACKUP_DIR/redis_data_$DATE.tar.gz" ]; then
    echo "Backup completed successfully"
else
    echo "Backup failed"
    exit 1
fi

# Upload to cloud storage
echo "Uploading backups to cloud storage..."

# AWS S3
if command -v aws &> /dev/null; then
    echo "Uploading to AWS S3..."
    aws s3 sync $BACKUP_DIR s3://$CLOUD_BACKUP_BUCKET/$CLOUD_BACKUP_PATH/
    
    # Set lifecycle policy for old backups
    aws s3api put-bucket-lifecycle-configuration \
        --bucket $CLOUD_BACKUP_BUCKET \
        --lifecycle-configuration '{
            "Rules": [
                {
                    "ID": "DeleteOldBackups",
                    "Status": "Enabled",
                    "Expiration": {
                        "Days": 30
                    }
                }
            ]
        }'
fi

# Google Cloud Storage
if command -v gsutil &> /dev/null; then
    echo "Uploading to Google Cloud Storage..."
    gsutil -m cp -r $BACKUP_DIR/* gs://$CLOUD_BACKUP_BUCKET/$CLOUD_BACKUP_PATH/
    
    # Set lifecycle policy
    gsutil lifecycle set '{
        "rule": [
            {
                "action": {"type": "Delete"},
                "condition": {"age": 30}
            }
        ]
    }' gs://$CLOUD_BACKUP_BUCKET
fi

# Azure Blob Storage
if command -v az &> /dev/null; then
    echo "Uploading to Azure Blob Storage..."
    az storage blob upload-batch \
        --account-name $AZURE_STORAGE_ACCOUNT \
        --auth-mode key \
        --destination $CLOUD_BACKUP_BUCKET/$CLOUD_BACKUP_PATH \
        --source $BACKUP_DIR
fi

echo "Backup process completed" 