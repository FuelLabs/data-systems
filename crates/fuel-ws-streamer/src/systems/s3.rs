use aws_config::BehaviorVersion;
use aws_sdk_s3::{config::Region, Client as S3Client};

use crate::config::S3Config;

// Initializes the S3 client.
pub async fn s3_connect(config: S3Config) -> S3Client {
    // Create region for the S3 bucket.
    let region = Region::new(config.region);
    // Create an AWS config with a custom endpoint to interact with MinIO.
    let cfg = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url(config.endpoint);

    // Establish a new session with the AWS S3 API.
    let cfg = cfg.region(region).load().await;

    // Return the S3 client.
    S3Client::new(&cfg)
}
