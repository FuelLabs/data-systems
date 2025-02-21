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

const QUERY_BATCH_SIZE: i64 = 10000;
const UPDATE_BATCH_SIZE: usize = 1000;

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

#[derive(Debug, sqlx::FromRow)]
struct BlockHeightRange {
    min_height: Option<i64>,
    max_height: Option<i64>,
}

async fn get_block_height_range(pool: &PgPool) -> Result<(i64, i64)> {
    let range = sqlx::query_as::<_, BlockHeightRange>(
        "SELECT MIN(block_height) as min_height, MAX(block_height) as max_height
         FROM blocks
         WHERE timestamp IS NULL"
    )
    .fetch_one(pool)
    .await?;

    let min_height = range.min_height.unwrap_or(0);
    let max_height = range.max_height.unwrap_or(0);
    Ok((min_height, max_height))
}

#[tokio::main]
async fn main() -> Result<()> {
    let database_url =
        dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let (min_height, max_height) = get_block_height_range(&pool).await?;
    println!(
        "Processing blocks from height {} to {}",
        min_height, max_height
    );

    let semaphore = Arc::new(Semaphore::new(100));
    let mut join_set = JoinSet::new();
    let mut current_height = min_height;
    while current_height <= max_height {
        let chunk_end = (current_height + QUERY_BATCH_SIZE).min(max_height + 1);
        let pool = pool.clone();
        let permit = semaphore.clone().acquire_owned().await?;
        join_set.spawn(async move {
            let updated =
                update_block_range(&pool, current_height, chunk_end).await?;
            println!(
                "Completed chunk {}-{}, updated {} blocks",
                current_height,
                chunk_end - 1,
                updated
            );
            drop(permit); // Release the permit when done
            Ok::<usize, anyhow::Error>(updated)
        });
        current_height = chunk_end;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Wait for remaining tasks to complete
    let mut total_updated = 0;
    while let Some(result) = join_set.join_next().await {
        total_updated += result??;
    }

    println!("Successfully completed block timestamp updates. Total blocks updated: {}", total_updated);
    Ok(())
}
