#!/usr/bin/env bash

# verifyOS Backend Manual Deployment Script
# This script builds the Docker image and pushes it to Amazon ECR.

set -e

# Configuration
REGION="ap-southeast-1"
REPO_NAME="verifyos-backend"
SERVICE_NAME="verifyos-backend"

echo "🚀 Starting manual deployment for $SERVICE_NAME..."

# 1. Login to ECR
echo "🔑 Logging in to Amazon ECR..."
AWS_ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
ECR_URL="$AWS_ACCOUNT_ID.dkr.ecr.$REGION.amazonaws.com"
aws ecr get-login-password --region $REGION | docker login --username AWS --password-stdin $ECR_URL

# 2. Build Docker Image
echo "📦 Building Docker image for linux/amd64..."
# Run build from project root to include core library
cd ../..
docker build --platform linux/amd64 -t $REPO_NAME -f apps/backend/Dockerfile .
docker tag $REPO_NAME:latest $ECR_URL/$REPO_NAME:latest

# 3. Push to ECR
echo "📤 Pushing image to ECR..."
docker push $ECR_URL/$REPO_NAME:latest

# 4. Deploy to App Runner
echo "🔄 Updating App Runner service..."
# Find the Service ARN
SERVICE_ARN=$(aws apprunner list-services --region $REGION --query "ServiceSummaryList[?ServiceName=='$SERVICE_NAME'].ServiceArn" --output text)

if [ -z "$SERVICE_ARN" ] || [ "$SERVICE_ARN" == "None" ]; then
    echo "❌ Error: Could not find App Runner service named $SERVICE_NAME in $REGION"
    exit 1
fi

echo "📍 Found Service ARN: $SERVICE_ARN"
aws apprunner start-deployment --region $REGION --service-arn "$SERVICE_ARN"

echo "✅ Deployment initiated! Check AWS Console for progress."
