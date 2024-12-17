#!/bin/bash
set -e

echo "Creating S3 bucket in LocalStack..."

awslocal s3 mb s3://fuel-streams-test
echo "Bucket created: fuel-streams-test"
