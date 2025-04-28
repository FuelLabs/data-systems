use std::sync::Arc;

use anyhow::Result;
use fuel_streams_domains::infra::Db;
use fuel_streams_types::TxPointer;
use serde_json;
use sqlx::PgPool;
use tokio::{sync::Semaphore, task::JoinSet};

const QUERY_BATCH_SIZE: i64 = 50;

#[derive(Clone, Debug, sqlx::FromRow)]
struct TransactionRecord {
    id: i32,
    block_height: i64,
    tx_index: i32,
}

async fn fetch_transaction_chunk(
    pool: &PgPool,
    offset: i64,
) -> Result<Vec<TransactionRecord>> {
    println!("Fetching transaction chunk with offset {}", offset);
    sqlx::query_as::<_, TransactionRecord>(
        "SELECT id, block_height, tx_index
         FROM transactions
         WHERE tx_pointer IS NULL
         ORDER BY block_height ASC
         LIMIT $1
         OFFSET $2",
    )
    .bind(QUERY_BATCH_SIZE)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

async fn update_transaction_chunk(
    pool: &PgPool,
    transactions: Vec<TransactionRecord>,
) -> Result<usize> {
    if transactions.is_empty() {
        return Ok(0);
    }

    let mut tx = pool.begin().await?;
    for record in transactions.clone() {
        let tx_pointer = TxPointer {
            block_height: record.block_height.try_into()?,
            tx_index: record.tx_index as u16,
        };
        let tx_pointer = serde_json::to_vec(&tx_pointer)?;
        sqlx::query(
            "UPDATE transactions
             SET tx_pointer = $1
             WHERE id = $2",
        )
        .bind(&tx_pointer)
        .bind(record.id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(transactions.len())
}

pub async fn recover_tx_pointers(db: &Arc<Db>) -> Result<()> {
    let pool = db.pool_ref();
    let semaphore = Arc::new(Semaphore::new(30));
    let mut join_set = JoinSet::new();
    let mut offset = 0;
    let mut total_updated = 0;

    loop {
        let transactions = fetch_transaction_chunk(&pool, offset).await?;
        if transactions.is_empty() {
            break;
        }

        let pool = pool.clone();
        let permit = semaphore.clone().acquire_owned().await?;
        let chunk_transactions = transactions;
        join_set.spawn(async move {
            let updated =
                update_transaction_chunk(&pool, chunk_transactions).await?;
            println!(
                "Completed chunk at offset {}, updated {} transactions",
                offset, updated
            );
            drop(permit);
            Ok::<usize, anyhow::Error>(updated)
        });

        offset += QUERY_BATCH_SIZE;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    while let Some(result) = join_set.join_next().await {
        total_updated += result??;
    }

    println!(
        "Successfully completed transaction updates. Total transactions updated: {}",
        total_updated
    );
    Ok(())
}
