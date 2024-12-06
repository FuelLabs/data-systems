use aws_config::{meta::region::RegionProviderChain, BehaviorVersion, Region};
use aws_sdk_s3::Client;
use futures::{Stream, StreamExt};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum S3ClientError {
    #[error("AWS SDK Error: {0}")]
    AwsSdkError(String),
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
    /// Create a new `S3Client`.
    pub async fn new(
        bucket: &str,
        region: &str,
    ) -> Result<Self, S3ClientError> {
        // Create a region provider
        let region_provider =
            RegionProviderChain::first_try(Region::new(region.to_string()));

        // Load AWS configuration
        let aws_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .region(region_provider)
            .load()
            .await;

        // Create S3 client directly from the loaded configuration
        let client = Client::new(&aws_config);

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

    /// Get files in the bucket.
    pub fn get_files_stream(
        &self,
        batch_size: u8,
    ) -> impl Stream<Item = Result<Vec<u8>, S3ClientError>> + '_ {
        let s3_client = self.client.clone();
        let bucket = self.bucket.clone();

        futures::stream::unfold(None, move |continuation_token| {
            let s3_client = s3_client.clone();
            let bucket = bucket.clone();

            async move {
                let mut request = s3_client
                    .list_objects_v2()
                    .bucket(&bucket)
                    .max_keys(batch_size as i32);
                if let Some(token) = continuation_token {
                    request = request.continuation_token(token);
                }

                match request.send().await {
                    Ok(output) => {
                        // Collect object keys from the current page
                        let keys = output
                            .contents()
                            .iter()
                            .filter_map(|obj| {
                                obj.key().map(|k| k.as_bytes().to_vec())
                            })
                            .collect::<Vec<_>>();

                        // Get the next continuation token
                        let next_token =
                            output.next_continuation_token().map(String::from);

                        // Return the current keys and the next state
                        Some((Ok(keys), next_token))
                    }
                    Err(err) => Some((
                        Err(S3ClientError::AwsSdkError(err.to_string())),
                        None,
                    )),
                }
            }
        })
        .flat_map(|result| match result {
            Ok(keys) => {
                futures::stream::iter(keys.into_iter().map(Ok)).boxed_local()
            }
            Err(err) => futures::stream::iter(vec![Err(err)]).boxed_local(),
        })
    }

    /// Delete a single object from S3.
    /// Would be useful for clean ups
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
}
