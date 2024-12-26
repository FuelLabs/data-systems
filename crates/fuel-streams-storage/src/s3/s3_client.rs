use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;

use super::s3_client_opts::S3StorageOpts;
use crate::{
    storage::{Storage, StorageError},
    StorageConfig,
};

#[derive(Debug, Clone)]
pub struct S3Storage {
    client: Client,
    config: S3StorageOpts,
}

#[async_trait]
impl Storage for S3Storage {
    type Config = S3StorageOpts;

    async fn new(config: Self::Config) -> Result<Self, StorageError> {
        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(config.endpoint_url())
            .region(config.region())
            .no_credentials()
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
            .force_path_style(true)
            .disable_s3_express_session_auth(true)
            .build();

        let client = aws_sdk_s3::Client::from_conf(s3_config);
        Ok(Self { client, config })
    }

    async fn store(
        &self,
        key: &str,
        data: Vec<u8>,
    ) -> Result<(), StorageError> {
        #[allow(clippy::identity_op)]
        const LARGE_FILE_THRESHOLD: usize = 1 * 1024 * 1024; // 1MB
        if data.len() >= LARGE_FILE_THRESHOLD {
            tracing::debug!("Uploading file to S3 using multipart_upload");
            self.upload_multipart(key, data).await
        } else {
            tracing::debug!("Uploading file to S3 using put_object");
            self.put_object(key, data).await
        }
    }

    async fn retrieve(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let result = self
            .client
            .get_object()
            .bucket(self.config.bucket())
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError::RetrieveError(e.to_string()))?;

        Ok(result
            .body
            .collect()
            .await
            .map_err(|e| StorageError::RetrieveError(e.to_string()))?
            .into_bytes()
            .to_vec())
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        self.client
            .delete_object()
            .bucket(self.config.bucket())
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError::DeleteError(e.to_string()))?;
        Ok(())
    }
}

impl S3Storage {
    pub async fn create_bucket(&self) -> Result<(), StorageError> {
        self.client
            .create_bucket()
            .bucket(self.config.bucket())
            .send()
            .await
            .map_err(|e| StorageError::StoreError(e.to_string()))?;
        Ok(())
    }

    async fn put_object(
        &self,
        key: &str,
        object: Vec<u8>,
    ) -> Result<(), StorageError> {
        self.client
            .put_object()
            .bucket(self.config.bucket())
            .key(key)
            .body(object.into())
            .send()
            .await
            .map_err(|e| StorageError::StoreError(e.to_string()))?;

        Ok(())
    }

    async fn upload_multipart(
        &self,
        key: &str,
        data: Vec<u8>,
    ) -> Result<(), StorageError> {
        const CHUNK_SIZE: usize = 5 * 1024 * 1024; // 5MB chunks

        // Create multipart upload
        let create_multipart = self
            .client
            .create_multipart_upload()
            .bucket(self.config.bucket())
            .key(key)
            .send()
            .await
            .map_err(|e| {
                StorageError::StoreError(format!(
                    "Failed to create multipart upload: {}",
                    e
                ))
            })?;

        let upload_id = create_multipart.upload_id().ok_or_else(|| {
            StorageError::StoreError("Failed to get upload ID".to_string())
        })?;

        let mut completed_parts = Vec::new();
        let chunks = data.chunks(CHUNK_SIZE);
        let total_chunks = chunks.len();

        // Upload parts
        for (i, chunk) in chunks.enumerate() {
            let part_number = (i + 1) as i32;

            match self
                .client
                .upload_part()
                .bucket(self.config.bucket())
                .key(key)
                .upload_id(upload_id)
                .body(chunk.to_vec().into())
                .part_number(part_number)
                .send()
                .await
            {
                Ok(response) => {
                    if let Some(e_tag) = response.e_tag() {
                        completed_parts.push(
                            aws_sdk_s3::types::CompletedPart::builder()
                                .e_tag(e_tag)
                                .part_number(part_number)
                                .build(),
                        );
                    }
                }
                Err(err) => {
                    // Abort the multipart upload if a part fails
                    self.client
                        .abort_multipart_upload()
                        .bucket(self.config.bucket())
                        .key(key)
                        .upload_id(upload_id)
                        .send()
                        .await
                        .map_err(|e| {
                            StorageError::StoreError(format!(
                                "Failed to abort multipart upload: {}",
                                e
                            ))
                        })?;

                    return Err(StorageError::StoreError(format!(
                        "Failed to upload part: {}",
                        err
                    )));
                }
            }

            tracing::debug!(
                "Uploaded part {}/{} for key={}",
                part_number,
                total_chunks,
                key
            );
        }

        // Complete multipart upload
        self.client
            .complete_multipart_upload()
            .bucket(self.config.bucket())
            .key(key)
            .upload_id(upload_id)
            .multipart_upload(
                aws_sdk_s3::types::CompletedMultipartUpload::builder()
                    .set_parts(Some(completed_parts))
                    .build(),
            )
            .send()
            .await
            .map_err(|e| {
                StorageError::StoreError(format!(
                    "Failed to complete multipart upload: {}",
                    e
                ))
            })?;

        Ok(())
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn new_for_testing() -> Result<Self, StorageError> {
        dotenvy::dotenv().ok();

        use crate::{StorageEnv, StorageRole};
        let config = S3StorageOpts::new(StorageEnv::Local, StorageRole::Admin)
            .with_random_namespace();

        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(config.endpoint_url())
            .region(config.region())
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                "test", "test", None, None, "static",
            ))
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
            .force_path_style(true)
            .disable_s3_express_session_auth(true)
            .build();

        let client = aws_sdk_s3::Client::from_conf(s3_config);

        // Ensure bucket exists before running tests
        let storage = Self { client, config };
        storage.ensure_bucket().await?;
        Ok(storage)
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub async fn ensure_bucket(&self) -> Result<(), StorageError> {
        // Check if bucket exists
        let exists = self
            .client
            .head_bucket()
            .bucket(self.config.bucket())
            .send()
            .await
            .is_ok();

        // Create bucket if it doesn't exist
        if !exists {
            self.create_bucket().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;
    use crate::storage::Storage;

    #[tokio::test]
    async fn test_basic_operations() {
        let storage = S3Storage::new_for_testing().await.unwrap();

        // Test store and retrieve
        let key = "test-key";
        let content = b"Hello, Storage!".to_vec();

        storage.store(key, content.clone()).await.unwrap();
        let retrieved = storage.retrieve(key).await.unwrap();
        assert_eq!(retrieved, content);

        // Test delete
        storage.delete(key).await.unwrap();
        let result = storage.retrieve(key).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[traced_test]
    async fn test_file_size_threshold() {
        let storage = S3Storage::new_for_testing().await.unwrap();

        // Test small file (under 1MB)
        let small_content = vec![0u8; 500 * 1024];
        storage
            .store("small-file", small_content.clone())
            .await
            .unwrap();
        assert!(logs_contain("put_object"));

        // Verify small file was stored correctly
        let retrieved_small = storage.retrieve("small-file").await.unwrap();
        assert_eq!(retrieved_small, small_content);

        // Test large file (over 1MB)
        let large_content = vec![0u8; 2 * 1024 * 1024];
        storage
            .store("large-file", large_content.clone())
            .await
            .unwrap();
        assert!(logs_contain("multipart_upload"));

        // Verify large file was stored correctly
        let retrieved_large = storage.retrieve("large-file").await.unwrap();
        assert_eq!(retrieved_large, large_content);
    }

    #[tokio::test]
    async fn test_multipart_upload_with_multiple_chunks() {
        let storage = S3Storage::new_for_testing().await.unwrap();

        // Create a file that will require exactly 3 chunks (15MB + 1 byte)
        // Since chunk size is 5MB, this will create 3 chunks:
        // Chunk 1: 5MB
        // Chunk 2: 5MB
        // Chunk 3: 5MB + 1 byte
        let content_size = (5 * 1024 * 1024 * 3) + 1;
        let content: Vec<u8> = (0..content_size)
            .map(|i| (i % 255) as u8) // Create pattern to verify data integrity
            .collect();

        let key = "multiple-chunks";

        // Store the file
        storage.store(key, content.clone()).await.unwrap();

        // Retrieve and verify the file immediately after upload
        let retrieved_after_upload = storage.retrieve(key).await.unwrap();
        assert_eq!(
            retrieved_after_upload.len(),
            content.len(),
            "Retrieved file size should match original"
        );
        assert_eq!(
            retrieved_after_upload, content,
            "Retrieved file content should match original"
        );

        // Wait a moment and retrieve again to verify persistence
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let retrieved_after_wait = storage.retrieve(key).await.unwrap();
        assert_eq!(
            retrieved_after_wait.len(),
            content.len(),
            "Retrieved file size should still match after waiting"
        );
        assert_eq!(
            retrieved_after_wait, content,
            "Retrieved file content should still match after waiting"
        );

        // Clean up
        storage.delete(key).await.unwrap();

        // Verify deletion
        let result = storage.retrieve(key).await;
        assert!(
            result.is_err(),
            "File should no longer exist after deletion"
        );
    }
}
