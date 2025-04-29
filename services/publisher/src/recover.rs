use std::sync::Arc;

use anyhow::Result;
use fuel_streams_domains::infra::Db;
use fuel_streams_types::TxPointer;
use serde_json;
use sqlx::PgPool;
use tokio::task::JoinSet;
use tracing::info;

const QUERY_BATCH_SIZE: i64 = 500;

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

    let ids: Vec<i32> = transactions.iter().map(|t| t.id).collect();
    let tx_pointers: Vec<Vec<u8>> = transactions
        .iter()
        .map(|t| {
            let tx_pointer = TxPointer {
                block_height: t.block_height.into(),
                tx_index: t.tx_index as u16,
            };
            serde_json::to_vec(&tx_pointer).map_err(|e| e.into())
        })
        .collect::<Result<Vec<_>, anyhow::Error>>()?;

    let updated = sqlx::query(
        r#"
        UPDATE transactions
        SET tx_pointer = data.tx_pointer
        FROM (
            SELECT unnest($1::integer[]) AS id, unnest($2::bytea[]) AS tx_pointer
        ) AS data
        WHERE transactions.id = data.id
        "#,
    )
    .bind(&ids)
    .bind(&tx_pointers)
    .execute(pool)
    .await?
    .rows_affected() as usize;

    Ok(updated)
}

pub async fn recover_tx_pointers(db: &Arc<Db>) -> Result<()> {
    let pool = db.pool_ref();
    let mut join_set = JoinSet::new();
    let mut offset = 0;
    let mut total_updated = 0;
    let mut chunk = 1;

    loop {
        info!(
            "Fetching transaction chunk {} with offset {}",
            chunk, offset
        );

        let transactions = fetch_transaction_chunk(pool, offset).await?;
        if transactions.is_empty() {
            break;
        }

        let pool = pool.clone();
        let chunk_transactions = transactions;
        let current_chunk = chunk;
        join_set.spawn(async move {
            let updated =
                update_transaction_chunk(&pool, chunk_transactions).await?;
            info!(
                "Completed chunk {}, updated {} transactions",
                current_chunk, updated
            );
            Ok::<usize, anyhow::Error>(updated)
        });

        offset += QUERY_BATCH_SIZE;
        chunk += 1;
    }

    while let Some(result) = join_set.join_next().await {
        total_updated += result??;
    }

    info!("Total transactions updated: {}", total_updated);
    Ok(())
}
