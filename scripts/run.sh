#!/bin/bash

# Docker image to use. This can be overriden by adding `IMAGE=<whatever>` to the conf.env file that is stored in S3 (see lines below)
IMAGE="docker.io/samuelker/od-badge-signup:main"

# Get configuration
aws s3 cp s3://mybucket-onlydust-devops/deathnote/od-badge-signup/conf.env /tmp/conf.env
source /tmp/conf.env

# Get secrets
GITHUB_ID=$(aws secretsmanager get-secret-value --region eu-west-3 --query SecretString --output text --secret-id github-api-client-id)
GITHUB_SECRET=$(aws secretsmanager get-secret-value --region eu-west-3 --query SecretString --output text --secret-id github-api-client-secret)
STARKNET_ACCOUNT=$(aws secretsmanager get-secret-value --region eu-west-3 --query SecretString --output text --secret-id starknet-badge-registry-admin-account)
STARKNET_PRIVATE_KEY=$(aws secretsmanager get-secret-value --region eu-west-3 --query SecretString --output text --secret-id starknet-badge-registry-admin-private-key)

# Pull docker image
docker pull $IMAGE

# Bind logs
LOG_FILE=/var/log/docker-deathnote.txt
exec 3>&1 4>&2
trap 'exec 2>&4 1>&3' 0 1 2 3
exec 1>$LOG_FILE 2>&1

# Run the program
docker run -d -p 80:80 \
 --env GITHUB_ID="$GITHUB_ID" \
 --env GITHUB_SECRET="$GITHUB_SECRET" \
 --env STARKNET_ACCOUNT="$STARKNET_ACCOUNT" \
 --env STARKNET_PRIVATE_KEY="$STARKNET_PRIVATE_KEY" \
 --env STARKNET_BADGE_REGISTRY_ADDRESS="$STARKNET_BADGE_REGISTRY_ADDRESS" \
 --env STARKNET_CHAIN="$STARKNET_CHAIN" \
 --env ROCKET_LOG_LEVEL="$ROCKET_LOG_LEVEL" \
 $IMAGE
