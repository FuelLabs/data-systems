use std::{sync::Arc, time::Instant};

use anyhow::Result;
use clap::Parser;
use fuel_data_parser::DataEncoder;
use fuel_streams_domains::{
    blocks::Block,
    infra::{Db, DbConnectionOpts, QueryOptions},
    transactions::Transaction,
};
use fuel_streams_types::BlockHeight;
use fuel_web_utils::{shutdown::ShutdownController, tracing::init_tracing};
use sv_dune::{
    helpers::Store,
    processor::Processor,
    s3::S3TableName,
    Cli,
    DuneError,
};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;

    let cli = Cli::parse();
    let db = setup_db(&cli.db_url).await?;
    let store = Store::new()?;
    let shutdown = Arc::new(ShutdownController::new());
    shutdown.clone().spawn_signal_handler();

    let opts = QueryOptions::default();
    let last_height_indexed = Block::find_last_block_height(&db, &opts).await?;
    let last_height_saved = store.get_last_block_saved().await?;
    if *last_height_saved >= *last_height_indexed {
        tracing::info!("Last block saved is up to date");
        return Ok(());
    }

    tokio::select! {
        result = process_blocks(
            &db,
            &store,
            &cli,
            last_height_saved,
            last_height_indexed,
            &shutdown,
        ) => {
            if let Err(e) = result {
                tracing::error!("Error processing blocks: {:?}", e);
                return Err(e);
            }
        }
        _ = shutdown.wait_for_shutdown() => {
            tracing::info!("Shutdown signal received, waiting for processing to complete...");
        }
    }

    tracing::info!("Shutdown complete");
    Ok(())
}

async fn setup_db(db_url: &str) -> Result<Arc<Db>, DuneError> {
    let db = Db::new(DbConnectionOpts {
        connection_str: db_url.to_string(),
        ..Default::default()
    })
    .await?;
    Ok(db)
}

async fn process_blocks(
    db: &Arc<Db>,
    store: &Store,
    cli: &Cli,
    last_height_saved: BlockHeight,
    last_height_indexed: BlockHeight,
    shutdown: &Arc<ShutdownController>,
) -> Result<()> {
    let processor = Processor::new(&cli.storage_type).await?;
    let first_height = get_initial_height(db, cli, &last_height_saved).await?;
    let total_blocks = *last_height_indexed - *first_height;
    tracing::info!(
        "Processing from block #{first_height} to #{last_height_indexed}"
    );
    tracing::info!("Total of {total_blocks} blocks to process");

    let batches = Processor::calculate_height_batches(
        first_height,
        last_height_indexed,
        cli.batch_size,
    )?;

    for (current_start, batch_end) in batches {
        if shutdown.is_shutdown_initiated() {
            tracing::info!(
                "Shutdown requested, stopping processing gracefully..."
            );
            break;
        }

        if !store
            .should_continue_processing(cli.max_blocks_to_store)
            .await?
        {
            tracing::info!(
                "Reached maximum blocks to store ({}), stopping processing",
                cli.max_blocks_to_store.unwrap()
            );
            break;
        }

        let start_time = Instant::now();
        tracing::info!(
            "Processing batch from block #{current_start} to #{batch_end}"
        );

        let blocks_and_txs =
            get_blocks_and_transactions(db, current_start, batch_end).await?;

        process_batch(&processor, &blocks_and_txs, shutdown).await?;
        if let Some((block, _)) = blocks_and_txs.last() {
            store.save_last_block(block).await?;
            store.save_total_blocks(blocks_and_txs.len()).await?;
        }
        tracing::info!("Batch processed in {:?}", start_time.elapsed());
    }

    Ok(())
}

async fn process_batch(
    processor: &Processor,
    blocks_and_txs: &[(Block, Vec<Transaction>)],
    shutdown: &Arc<ShutdownController>,
) -> Result<()> {
    let blocks_task = async {
        let batches = processor.calculate_blocks_batches(blocks_and_txs)?;
        processor
            .process_range(batches, S3TableName::Blocks)
            .await?;
        Ok::<_, DuneError>(())
    };

    let tx_task = async {
        let batches = processor.calculate_txs_batches(blocks_and_txs)?;
        processor
            .process_range(batches, S3TableName::Transactions)
            .await?;
        Ok::<_, DuneError>(())
    };

    let receipts_task = async {
        let batches = processor.calculate_receipts_batches(blocks_and_txs)?;
        processor
            .process_range(batches, S3TableName::Receipts)
            .await?;
        Ok::<_, DuneError>(())
    };

    tokio::select! {
        result = async {
            tokio::try_join!(blocks_task, tx_task, receipts_task)?;
            Ok::<_, DuneError>(())
        } => {
            result?;
        }
        _ = shutdown.wait_for_shutdown() => {
            tracing::info!("Shutdown signal received, stopping batch processing...");
        }
    }

    Ok(())
}

async fn get_initial_height(
    db: &Arc<Db>,
    cli: &Cli,
    last_height_saved: &BlockHeight,
) -> Result<BlockHeight> {
    if let Some(height) = cli.from_block {
        return Ok(height);
    }
    let opts = QueryOptions::default();
    let first_height_indexed =
        Block::find_first_block_height(db, &opts).await?;
    if *first_height_indexed > **last_height_saved {
        Ok(first_height_indexed)
    } else {
        Ok(*last_height_saved)
    }
}

async fn get_blocks_and_transactions(
    db: &Arc<Db>,
    start_height: BlockHeight,
    end_height: BlockHeight,
) -> Result<Vec<(Block, Vec<Transaction>)>> {
    let mut data = Vec::new();
    let blocks = Block::find_in_height_range(
        db,
        start_height,
        end_height,
        &QueryOptions::default(),
    )
    .await?;

    for db_item in blocks {
        let block = Block::decode_json(&db_item.value)?;
        let transactions = block.transactions_from_db(db).await?;
        data.push((block, transactions));
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use fuel_streams_domains::mocks::{MockBlock, MockTransaction};
    use sv_dune::{
        helpers::AvroParser,
        processor::Processor,
        s3::{S3Storage, S3StorageOpts, S3TableName, Storage},
        schemas::AvroBlock,
    };

    mod file_storage {
        use pretty_assertions::assert_eq;

        use super::*;

        #[tokio::test]
        async fn test_process_blocks_range_serialization() -> Result<()> {
            let processor = Processor::new("File").await?;
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
        use sv_dune::s3::StorageConfig;

        use super::*;

        #[tokio::test]
        async fn test_process_blocks_range_serialization() -> Result<()> {
            let processor = Processor::new("S3").await?;
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
            let data = s3_storage.retrieve(s3_key).await?;

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
