use std::sync::Arc;

use anyhow::Result;
use fuel_streams_domains::blocks::{Block, BlockTimestamp};
use fuel_streams_store::record::DataEncoder;
use rayon::prelude::*;
use sqlx::PgPool;
use tokio::{sync::Semaphore, task::JoinSet};

#[derive(Debug, sqlx::FromRow)]
struct BlockRecord {
    subject: String,
    value: Vec<u8>,
}

#[derive(Debug, sqlx::FromRow)]
struct HeightRange {
    min_height: i64,
    max_height: i64,
}

const QUERY_BATCH_SIZE: i64 = 10000;
const UPDATE_BATCH_SIZE: usize = 10000;

async fn fetch_block_chunk(
    pool: &PgPool,
    start_height: i64,
    end_height: i64,
) -> Result<Vec<BlockRecord>> {
    println!(
        "Fetching block chunk from height {} to {}",
        start_height, end_height
    );
    sqlx::query_as::<_, BlockRecord>(
        "SELECT subject, block_height, value
         FROM blocks
         WHERE block_height >= $1 AND block_height < $2
         AND timestamp IS NULL
         ORDER BY block_height",
    )
    .bind(start_height)
    .bind(end_height)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

async fn update_block_range(
    pool: &PgPool,
    start_height: i64,
    end_height: i64,
) -> Result<usize> {
    // Single query for this chunk
    let blocks = fetch_block_chunk(pool, start_height, end_height).await?;
    if blocks.is_empty() {
        return Ok(0);
    }

    // Process blocks in parallel using rayon
    let updates: Vec<(String, f64)> = blocks
        .par_iter()
        .map(|block_record| {
            let block = Block::decode_json(&block_record.value)?;
            let timestamp = BlockTimestamp::from(&block);
            println!("Processed block height {}", block.height);
            Ok((block_record.subject.clone(), timestamp.to_seconds() as f64))
        })
        .collect::<Result<Vec<_>>>()?;

    // Single transaction for this chunk
    let mut tx = pool.begin().await?;
    for chunk in updates.chunks(UPDATE_BATCH_SIZE) {
        let query = format!(
            "UPDATE blocks SET timestamp = to_timestamp(u.ts)
             FROM (VALUES {}) AS u(subject, ts)
             WHERE blocks.subject = u.subject AND blocks.timestamp IS NULL",
            (0..chunk.len())
                .map(|i| format!("(${},${})", i * 2 + 1, i * 2 + 2))
                .collect::<Vec<_>>()
                .join(",")
        );

        let mut query_builder = sqlx::query(&query);
        for (subject, timestamp) in chunk {
            query_builder = query_builder.bind(subject).bind(timestamp);
        }

        query_builder.execute(&mut *tx).await?;
    }

    tx.commit().await?;
    Ok(updates.len())
}

#[tokio::main]
async fn main() -> Result<()> {
    let database_url =
        dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Get the minimum height with null timestamp and maximum height overall
    let HeightRange { min_height, max_height } = sqlx::query_as::<_, HeightRange>(
        "SELECT
            (SELECT MIN(block_height) FROM blocks WHERE timestamp IS NULL) as min_height,
            (SELECT MAX(block_height) FROM blocks) as max_height"
    )
    .fetch_one(&pool)
    .await?;

    println!(
        "Processing blocks from height {} to {}",
        min_height, max_height
    );

    let semaphore = Arc::new(Semaphore::new(30));
    let mut join_set = JoinSet::new();

    // Calculate total chunks needed starting from min_height
    let total_chunks = ((max_height - min_height + 1) + QUERY_BATCH_SIZE - 1)
        / QUERY_BATCH_SIZE;

    // Spawn tasks for all chunks, starting from min_height
    for chunk_idx in 0..total_chunks {
        let chunk_start = min_height + (chunk_idx * QUERY_BATCH_SIZE);
        let chunk_end = (chunk_start + QUERY_BATCH_SIZE).min(max_height + 1);

        let pool = pool.clone();
        let permit = semaphore.clone().acquire_owned().await?;
        join_set.spawn(async move {
            let updated =
                update_block_range(&pool, chunk_start, chunk_end).await?;
            println!(
                "Completed chunk {}-{}, updated {} blocks",
                chunk_start,
                chunk_end - 1,
                updated
            );
            drop(permit);
            Ok::<usize, anyhow::Error>(updated)
        });

        // Small delay to prevent overwhelming the database with initial connections
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Wait for remaining tasks to complete
    let mut total_updated = 0;
    while let Some(result) = join_set.join_next().await {
        total_updated += result??;
    }

    println!("Successfully completed block timestamp updates. Total blocks updated: {}", total_updated);
    Ok(())
}
