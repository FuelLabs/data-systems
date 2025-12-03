#![deny(warnings)]

use anyhow::Result;
use clap::Parser;
use fuel_core_services::Service;
use fuel_web_utils::{
    shutdown::ShutdownController,
    tracing::init_tracing,
};
use std::sync::Arc;
use sv_dune::{
    Cli,
    service::{
        Config,
        new_service,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;

    let cli = Cli::parse();
    let shutdown = Arc::new(ShutdownController::new());
    shutdown.clone().spawn_signal_handler();

    let config = Config {
        url: cli.url,
        starting_height: cli.starting_block.into(),
        storage_type: cli.storage_type,
        registry_blocks_request_batch_size: cli.registry_blocks_request_batch_size,
        registry_blocks_request_concurrency: cli.registry_blocks_request_concurrency,
    };

    let service = new_service(config)?;

    service.start_and_await().await?;

    tokio::select! {
        state = service.await_stop() => {
            tracing::error!("Service stopped working: {:?}", state);
        }
        _ = shutdown.wait_for_shutdown() => {
            tracing::info!("Shutdown signal received, waiting for processing to complete...");
        }
    }

    service.stop_and_await().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use fuel_streams_domains::mocks::{
        MockBlock,
        MockTransaction,
    };
    use sv_dune::{
        helpers::AvroParser,
        processor::Processor,
        s3::{
            S3Storage,
            S3StorageOpts,
            S3TableName,
            Storage,
        },
        schemas::AvroBlock,
    };

    mod file_storage {
        use super::*;
        use pretty_assertions::assert_eq;
        use sv_dune::processor::StorageTypeConfig;

        #[tokio::test]
        async fn test_process_blocks_range_serialization() -> Result<()> {
            let processor = Processor::new(StorageTypeConfig::File).await?;
            let mut blocks_and_txs = Vec::new();

            // Create 10 blocks with transactions
            for i in 0..3 {
                let mut block = MockBlock::random();
                block.height = (i + 1).into();
                let txs = vec![MockTransaction::script(vec![], vec![], vec![])];
                blocks_and_txs.push((block, txs));
            }

            // Sort blocks by height like in the repository
            blocks_and_txs.sort_by_key(|(block, _)| block.height);

            // Calculate batches
            let batches = processor
                .calculate_blocks_batches(&blocks_and_txs)
                .expect("Failed to calculate batches");

            // Process the batches for the Blocks table
            let created_files = processor
                .process_range(batches, S3TableName::Blocks)
                .await?;

            assert_eq!(
                created_files.len(),
                1,
                "Expected one file for small data set"
            );
            let created_file_path = &created_files[0];

            // Deserialize using Avro
            let file_contents = std::fs::read(created_file_path)?;
            let parser = AvroParser::default();
            let deserialized = parser
                .reader_with_schema::<AvroBlock>()?
                .deserialize(&file_contents)?;

            // Verify the deserialized data
            assert_eq!(
                deserialized.len(),
                blocks_and_txs.len(),
                "Should have same number of blocks"
            );

            // Check each block's data is in order
            for (i, deserialized_block) in deserialized.iter().enumerate() {
                let (original_block, _) = &blocks_and_txs[i];

                // Verify block metadata
                assert_eq!(
                    deserialized_block.height,
                    Some(original_block.height.0 as i64),
                    "Block height should match"
                );
                assert_eq!(
                    deserialized_block.version,
                    Some(original_block.version.to_string()),
                    "Block version should match"
                );
                assert_eq!(
                    deserialized_block.producer,
                    Some(original_block.producer.as_ref().to_vec().into()),
                    "Block producer should match"
                );
            }

            // Clean up the test file
            let _ = std::fs::remove_file(created_file_path);

            Ok(())
        }
    }

    mod s3_storage {
        use pretty_assertions::assert_eq;
        use sv_dune::{
            processor::StorageTypeConfig,
            s3::StorageConfig,
        };

        use super::*;

        #[tokio::test]
        async fn test_process_blocks_range_serialization() -> Result<()> {
            let processor = Processor::new(StorageTypeConfig::S3).await?;
            let mut blocks_and_txs = Vec::new();

            // Create a few blocks with transactions (keep it small)
            for i in 0..3 {
                let mut block = MockBlock::random();
                block.height = (i + 1).into();
                let txs = vec![MockTransaction::script(vec![], vec![], vec![])];
                blocks_and_txs.push((block, txs));
            }

            // Sort blocks by height
            blocks_and_txs.sort_by_key(|(block, _)| block.height);

            // Calculate batches
            let batches = processor
                .calculate_blocks_batches(&blocks_and_txs)
                .expect("Failed to calculate batches");

            // Process the batches
            let s3_keys = processor
                .process_range(batches, S3TableName::Blocks)
                .await?;

            assert_eq!(s3_keys.len(), 1, "Expected one S3 key");
            let s3_key = &s3_keys[0];

            // Get the S3 storage client to retrieve and verify the data
            let s3_storage_opts = S3StorageOpts::admin_opts();
            let s3_storage = S3Storage::new(s3_storage_opts).await?;

            // Retrieve the data from S3
            let data = s3_storage.retrieve(s3_key).await?.unwrap();

            // Deserialize using Avro
            let parser = AvroParser::default();
            let deserialized = parser
                .reader_with_schema::<AvroBlock>()?
                .deserialize(&data)?;

            // Verify the deserialized data
            assert_eq!(
                deserialized.len(),
                blocks_and_txs.len(),
                "Should have same number of blocks"
            );

            // Check each block's data is in order
            for (i, deserialized_block) in deserialized.iter().enumerate() {
                let (original_block, _) = &blocks_and_txs[i];

                // Verify block metadata
                assert_eq!(
                    deserialized_block.height,
                    Some(original_block.height.0 as i64),
                    "Block height should match"
                );
                assert_eq!(
                    deserialized_block.version,
                    Some(original_block.version.to_string()),
                    "Block version should match"
                );
                assert_eq!(
                    deserialized_block.producer,
                    Some(original_block.producer.as_ref().to_vec().into()),
                    "Block producer should match"
                );
            }

            // Clean up - delete the test object from S3
            s3_storage.delete(s3_key).await?;

            Ok(())
        }
    }
}
