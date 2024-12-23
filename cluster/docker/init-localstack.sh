#!/bin/bash
set -e

echo "Creating S3 bucket in LocalStack..."

BUCKET_NAME=${AWS_S3_BUCKET_NAME:-fuel-streams-test}
awslocal s3 mb "s3://${BUCKET_NAME}"
echo "Bucket created: ${BUCKET_NAME}"
