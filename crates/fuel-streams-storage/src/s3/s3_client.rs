use aws_config::{meta::region::RegionProviderChain, Region};
use aws_sdk_s3::{
    config::http::HttpResponse,
    operation::{
        create_bucket::CreateBucketError,
        delete_bucket::DeleteBucketError,
        delete_object::DeleteObjectError,
        get_object::GetObjectError,
        put_object::PutObjectError,
    },
    Client,
};
use aws_smithy_runtime_api::client::result::SdkError;
use aws_smithy_types::byte_stream::error::Error as BytesStreamError;
use thiserror::Error;

use super::s3_client_opts::S3ClientOpts;

#[derive(Error, Debug)]
pub enum S3ClientError {
    #[error("AWS SDK Create Error: {0}")]
    CreateBucketError(#[from] SdkError<CreateBucketError, HttpResponse>),
    #[error("AWS SDK Delete bucket Error: {0}")]
    DeleteBucketError(#[from] SdkError<DeleteBucketError, HttpResponse>),
    #[error("AWS SDK Put Error: {0}")]
    PutObjectError(#[from] SdkError<PutObjectError, HttpResponse>),
    #[error("AWS SDK Get Error: {0}")]
    GetObjectError(#[from] SdkError<GetObjectError, HttpResponse>),
    #[error("Error aggregating bytes from S3: {0}")]
    BuildObjectAfterGettingError(#[from] BytesStreamError),
    #[error("AWS SDK Delete object Error: {0}")]
    DeleteObjectError(#[from] SdkError<DeleteObjectError, HttpResponse>),
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
    pub async fn new(opts: &S3ClientOpts) -> Result<Self, S3ClientError> {
        let region_provider =
            RegionProviderChain::first_try(Region::new(opts.region()));
        let region = region_provider.region().await.unwrap();

        // Load AWS configuration
        let aws_config = aws_config::from_env()
            .endpoint_url(opts.endpoint_url())
            .region(region)
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
            .force_path_style(true)
            .build();

        let client = aws_sdk_s3::Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket: opts.bucket.clone(),
        })
    }

    pub fn arc(self) -> std::sync::Arc<Self> {
        std::sync::Arc::new(self)
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
            .await?;

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
            .await?;

        Ok(result.body.collect().await?.into_bytes().to_vec())
    }

    /// Delete a single object from S3.
    pub async fn delete_object(&self, key: &str) -> Result<(), S3ClientError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        Ok(())
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn create_bucket(&self) -> Result<(), S3ClientError> {
        // Create bucket
        self.client
            .create_bucket()
            .bucket(&self.bucket)
            .send()
            .await?;

        Ok(())
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn new_for_testing() -> Self {
        use fuel_networks::FuelNetwork;

        dotenvy::dotenv().expect(".env file not found");

        let random_bucket = {
            use rand::Rng;
            let random_int: u32 = rand::thread_rng().gen();
            format!("fuel-streams-test-{}", random_int)
        };

        let s3_client = Self::new(
            &S3ClientOpts::new(FuelNetwork::Local).with_bucket(&random_bucket),
        )
        .await
        .expect(
            "S3Client creation failed. Check AWS Env vars and Localstack setup",
        );

        s3_client
            .create_bucket()
            .await
            .expect("Failed to create bucket");

        s3_client
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
