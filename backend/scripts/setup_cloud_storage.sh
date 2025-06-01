#!/bin/bash

# Function to check if a command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# AWS S3 Setup
if command_exists aws; then
    echo "Setting up AWS S3 credentials..."
    aws configure set aws_access_key_id "${AWS_ACCESS_KEY_ID}"
    aws configure set aws_secret_access_key "${AWS_SECRET_ACCESS_KEY}"
    aws configure set default.region "${AWS_REGION}"
    
    # Create S3 bucket if it doesn't exist
    aws s3api create-bucket \
        --bucket "${CLOUD_BACKUP_BUCKET}" \
        --region "${AWS_REGION}" \
        --create-bucket-configuration LocationConstraint="${AWS_REGION}" || true
fi

# Google Cloud Storage Setup
if command_exists gsutil; then
    echo "Setting up Google Cloud Storage credentials..."
    echo "${GOOGLE_CLOUD_CREDENTIALS}" > /tmp/google-credentials.json
    gcloud auth activate-service-account --key-file=/tmp/google-credentials.json
    rm /tmp/google-credentials.json
    
    # Create GCS bucket if it doesn't exist
    gsutil mb -l "${GCP_REGION}" gs://"${CLOUD_BACKUP_BUCKET}" || true
fi

# Azure Blob Storage Setup
if command_exists az; then
    echo "Setting up Azure Blob Storage credentials..."
    az login --service-principal \
        --username "${AZURE_CLIENT_ID}" \
        --password "${AZURE_CLIENT_SECRET}" \
        --tenant "${AZURE_TENANT_ID}"
    
    # Create storage account if it doesn't exist
    az storage account create \
        --name "${AZURE_STORAGE_ACCOUNT}" \
        --resource-group "${AZURE_RESOURCE_GROUP}" \
        --location "${AZURE_LOCATION}" \
        --sku Standard_LRS || true
    
    # Create container if it doesn't exist
    az storage container create \
        --account-name "${AZURE_STORAGE_ACCOUNT}" \
        --name "${CLOUD_BACKUP_BUCKET}" || true
fi

echo "Cloud storage setup completed" 