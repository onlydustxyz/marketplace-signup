#!/bin/bash

# Get configuration
aws s3 cp s3://mybucket-onlydust-devops/deathnote/od-badge-signup/conf.env /tmp/conf.env

# Get secrets
alias getawssecret="aws secretsmanager get-secret-value --region eu-west-3 --query SecretString --output text --secret-id"
GITHUB_ID=$(getawssecret github-api-client-id)
GITHUB_SECRET=$(getawssecret github-api-client-secret)
STARKNET_ACCOUNT=$(getawssecret starknet-badge-registry-admin-account)
STARKNET_PRIVATE_KEY=$(getawssecret starknet-badge-registry-admin-private-key)

# Pull docker image
docker pull public.ecr.aws/a3g2g4b8/od-badge-signup:main

# Bind logs
LOG_FILE=/var/log/docker-deathnote.txt
exec 3>&1 4>&2
trap 'exec 2>&4 1>&3' 0 1 2 3
exec 1>$LOG_FILE 2>&1

# Run the program
docker run -d -p 80:8000 \
 --env GITHUB_ID="$GITHUB_ID" \
 --env GITHUB_SECRET="$GITHUB_SECRET" \
 --env STARKNET_ACCOUNT="$STARKNET_ACCOUNT" \
 --env STARKNET_PRIVATE_KEY="$STARKNET_PRIVATE_KEY" \
 --env-file /tmp/conf.env \
 od-badge-signup
