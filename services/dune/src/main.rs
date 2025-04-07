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
    helpers::{BatchCalculator, Store},
    processor::BlocksProcessor,
    Cli,
    DuneError,
};
use tokio::try_join;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;

    let cli = Cli::parse();
    let db = setup_db(&cli.db_url).await?;
    let store = setup_store(&cli)?;
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

fn setup_store(cli: &Cli) -> Result<Store> {
    Store::new(cli.storage_file_dir.as_deref())
}

async fn process_blocks(
    db: &Arc<Db>,
    store: &Store,
    cli: &Cli,
    last_height_saved: BlockHeight,
    last_height_indexed: BlockHeight,
    shutdown: &Arc<ShutdownController>,
) -> Result<()> {
    let processor = BlocksProcessor::new(&cli.storage_type).await?;
    let first_height = get_initial_height(db, &last_height_saved).await?;
    let total_blocks = *last_height_indexed - *first_height;
    tracing::info!(
        "Processing from block #{first_height} to #{last_height_indexed}"
    );
    tracing::info!("Total of {total_blocks} blocks to process");

    const BATCH_SIZE: u64 = 3600;
    let mut current_start = *first_height;
    while current_start <= *last_height_indexed {
        if shutdown.is_shutdown_initiated() {
            tracing::info!(
                "Shutdown requested, stopping processing gracefully..."
            );
            break;
        }

        // Check if we should continue processing based on max_blocks
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
        let batch_end = std::cmp::min(
            current_start + (BATCH_SIZE - 1) as u32,
            *last_height_indexed,
        );

        tracing::info!(
            "Processing batch from block #{current_start} to #{batch_end}"
        );

        let blocks_and_txs = get_blocks_and_transactions(
            db,
            current_start.into(),
            batch_end.into(),
        )
        .await?;

        process_batch(&processor, &blocks_and_txs, shutdown).await?;
        if let Some((block, _)) = blocks_and_txs.last() {
            store.save_last_block(block).await?;
            store.save_total_blocks(blocks_and_txs.len() as u16).await?;
        }
        tracing::info!("Batch processed in {:?}", start_time.elapsed());
        current_start = batch_end + 1;
    }

    Ok(())
}

async fn process_batch(
    processor: &BlocksProcessor,
    blocks_and_txs: &[(Block, Vec<Transaction>)],
    shutdown: &Arc<ShutdownController>,
) -> Result<()> {
    let calculator = processor.batch_calculator();
    let (block_batches, tx_batches, receipt_batches) =
        calculate_batches(&calculator, blocks_and_txs).await?;

    let blocks = async {
        for (start, end) in block_batches {
            if shutdown.is_shutdown_initiated() {
                break;
            }
            processor
                .process_blocks_range(blocks_and_txs, start, end)
                .await?;
        }
        Ok::<_, DuneError>(())
    };

    let transactions = async {
        for (start, end) in tx_batches {
            if shutdown.is_shutdown_initiated() {
                break;
            }
            processor
                .process_txs_range(blocks_and_txs, start, end)
                .await?;
        }
        Ok::<_, DuneError>(())
    };

    let receipts = async {
        for (start, end) in receipt_batches {
            if shutdown.is_shutdown_initiated() {
                break;
            }
            processor
                .process_receipts_range(blocks_and_txs, start, end)
                .await?;
        }
        Ok::<_, DuneError>(())
    };

    try_join!(blocks, transactions, receipts)?;
    Ok(())
}

async fn calculate_batches(
    calculator: &BatchCalculator,
    blocks_and_txs: &[(Block, Vec<Transaction>)],
) -> Result<(
    Vec<(BlockHeight, BlockHeight)>,
    Vec<(BlockHeight, BlockHeight)>,
    Vec<(BlockHeight, BlockHeight)>,
)> {
    try_join!(
        async {
            Ok::<_, anyhow::Error>(
                calculator.calculate_blocks_batches(blocks_and_txs),
            )
        },
        async {
            Ok::<_, anyhow::Error>(
                calculator.calculate_txs_batches(blocks_and_txs),
            )
        },
        async {
            Ok::<_, anyhow::Error>(
                calculator.calculate_receipts_batches(blocks_and_txs),
            )
        }
    )
}

async fn get_initial_height(
    db: &Arc<Db>,
    last_height_saved: &BlockHeight,
) -> Result<BlockHeight> {
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
    use std::fs;

    use anyhow::Result;
    use fuel_streams_domains::mocks::{MockBlock, MockTransaction};
    use pretty_assertions::assert_eq;
    use sv_dune::{helpers::AvroParser, schemas::AvroBlock};

    use super::*;

    #[tokio::test]
    async fn test_process_blocks_range_serialization() -> Result<()> {
        let processor = BlocksProcessor::new("File").await?;
        let mut blocks_and_txs = Vec::new();

        // Create 10 blocks with transactions
        for _ in 0..10 {
            let block = MockBlock::random();
            let txs = MockTransaction::all();
            blocks_and_txs.push((block, txs.clone()));
        }

        // Sort blocks by height like in the repository
        blocks_and_txs.sort_by_key(|(block, _)| block.height);

        let first_block = &blocks_and_txs.first().unwrap().0;
        let last_block = &blocks_and_txs.last().unwrap().0;
        let first_height = first_block.height;
        let last_height = last_block.height;

        // Process the blocks
        let created = processor
            .process_blocks_range(&blocks_and_txs, first_height, last_height)
            .await?;

        // Deserialize using Avro
        let file_contents = fs::read(&created)?;
        let parser = AvroParser::default();
        let deserialized = parser
            .reader_with_schema::<AvroBlock>()
            .unwrap()
            .deserialize(&file_contents)
            .unwrap();

        // Verify the deserialized data
        assert_eq!(
            deserialized.len(),
            blocks_and_txs.len(),
            "Should have same number of blocks"
        );

        // Check each block's data is in order
        for (i, deserialized_block) in deserialized.iter().enumerate() {
            let (original_block, original_txs) = &blocks_and_txs[i];

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
                Some(original_block.producer.as_ref().to_vec()),
                "Block producer should match"
            );

            // Verify transactions
            assert_eq!(
                deserialized_block.transactions.len(),
                original_txs.len(),
                "Number of transactions should match"
            );

            // Verify transaction data
            for (j, deserialized_tx) in
                deserialized_block.transactions.iter().enumerate()
            {
                let original_tx = &original_txs[j];
                assert_eq!(
                    deserialized_tx.id,
                    Some(original_tx.id.as_ref().to_vec()),
                    "Transaction ID should match"
                );
                assert_eq!(
                    deserialized_tx.block_height,
                    Some(original_block.height.0 as i64),
                    "Transaction block height should match"
                );
            }
        }

        // Clean up the test file
        let _ = fs::remove_file(&created);

        Ok(())
    }

    #[tokio::test]
    async fn test_process_empty_blocks_range() -> Result<()> {
        let processor = BlocksProcessor::new("File").await?;

        // Create a single empty block for testing
        let block = MockBlock::random();
        let height = block.height;
        let blocks_and_txs = vec![(block.clone(), vec![])];

        // Process the empty block
        let created = processor
            .process_blocks_range(&blocks_and_txs, height, height)
            .await?;

        // Deserialize and verify
        let file_contents = fs::read(&created)?;
        let parser = AvroParser::default();
        let deserialized = parser
            .reader_with_schema::<AvroBlock>()
            .unwrap()
            .deserialize(&file_contents)
            .unwrap();

        assert_eq!(deserialized.len(), 1, "Should have one block");
        let deserialized_block = &deserialized[0];
        assert_eq!(
            deserialized_block.height,
            Some(block.height.0 as i64),
            "Block height should match"
        );
        assert_eq!(
            deserialized_block.transactions.len(),
            0,
            "Should have no transactions"
        );

        // Clean up
        let _ = fs::remove_file(&created);

        Ok(())
    }
}
