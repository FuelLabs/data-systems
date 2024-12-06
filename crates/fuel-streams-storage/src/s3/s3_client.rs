use aws_sdk_s3::Client;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum S3ClientError {
    #[error("AWS SDK Error: {0}")]
    AwsSdkError(String),
    #[error("Environment variable missing: {0}")]
    MissingEnvVar(String),
    #[error("Failed to stream objects because: {0}")]
    StreamingError(String),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct S3Client {
    client: Client,
    bucket: String,
}

impl S3Client {
    pub async fn new(bucket: &str) -> Result<Self, S3ClientError> {
        dotenvy::dotenv().expect(".env file not found");

        // Load AWS configuration
        let aws_config = aws_config::from_env().load().await;

        let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
            .force_path_style(true)
            .build();

        let client = aws_sdk_s3::Client::from_conf(s3_config);

        // Create bucket
        client
            .create_bucket()
            .bucket(bucket)
            .send()
            .await
            .map_err(|e| S3ClientError::AwsSdkError(e.to_string()))?;

        Ok(Self {
            client,
            bucket: bucket.to_string(),
        })
    }

    pub async fn put_object(
        &self,
        key: &str,
        object: Vec<u8>,
    ) -> Result<(), S3ClientError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(object.into())
            .send()
            .await
            .map_err(|e| S3ClientError::AwsSdkError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_object(
        &self,
        key: &str,
    ) -> Result<Vec<u8>, S3ClientError> {
        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| S3ClientError::AwsSdkError(e.to_string()))?;

        Ok(result
            .body
            .collect()
            .await
            .map_err(|e| S3ClientError::AwsSdkError(e.to_string()))?
            .into_bytes()
            .to_vec())
    }

    /// Delete a single object from S3.
    pub async fn delete_object(&self, key: &str) -> Result<(), S3ClientError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| S3ClientError::AwsSdkError(e.to_string()))?;

        Ok(())
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn new_for_testing() -> Self {
        let random_bucket = {
            use rand::Rng;
            let random_int: u32 = rand::thread_rng().gen();
            format!("fuel-streams-test-{}", random_int)
        };

        Self::new(&random_bucket).await.expect(
            "S3Client creation failed. Check AWS Env vars and Localstack setup",
        )
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn cleanup_after_testing(&self) {
        let client = &self.client;
        let bucket = &self.bucket;

        let objects = client
            .list_objects_v2()
            .bucket(bucket)
            .send()
            .await
            .unwrap();

        for object in objects.contents() {
            if let Some(key) = object.key() {
                client
                    .delete_object()
                    .bucket(bucket)
                    .key(key)
                    .send()
                    .await
                    .unwrap();
            }
        }

        client.delete_bucket().bucket(bucket).send().await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_put_and_get_object() {
        let s3_client = S3Client::new_for_testing().await;

        // Put object
        let key = "test-key";
        let content = b"Hello, LocalStack!".to_vec();
        s3_client
            .put_object(key, content.clone())
            .await
            .expect("Failed to put object");

        // Get object
        let result = s3_client
            .get_object(key)
            .await
            .expect("Failed to get object");

        assert_eq!(result, content);

        // Cleanup
        s3_client.cleanup_after_testing().await;
    }
}
