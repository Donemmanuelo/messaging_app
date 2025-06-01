#!/bin/bash

# Create a cron job for daily backups at 2 AM
(crontab -l 2>/dev/null; echo "0 2 * * * /opt/messaging-app/scripts/backup.sh >> /var/log/messaging-app/backup.log 2>&1") | crontab -

# Create log directory
mkdir -p /var/log/messaging-app

echo "Cron job for automated backups has been set up" 