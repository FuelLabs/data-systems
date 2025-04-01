use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Transaction, TransactionDbItem, TransactionsQuery};
use crate::infra::repository::{Repository, RepositoryError, RepositoryResult};

#[async_trait]
impl Repository for Transaction {
    type Item = TransactionDbItem;
    type QueryParams = TransactionsQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, TransactionDbItem>(
            "WITH upsert AS (
                INSERT INTO transactions (
                    subject, value, block_height, tx_id, tx_index,
                    tx_status, type, blob_id, created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (subject) DO UPDATE SET
                    tx_id = EXCLUDED.tx_id,
                    value = EXCLUDED.value,
                    block_height = EXCLUDED.block_height,
                    tx_index = EXCLUDED.tx_index,
                    tx_status = EXCLUDED.tx_status,
                    type = EXCLUDED.type,
                    blob_id = EXCLUDED.blob_id,
                    created_at = EXCLUDED.created_at,
                    published_at = $10
                RETURNING *
            )
            SELECT * FROM upsert",
        )
        .bind(db_item.subject.clone())
        .bind(db_item.value.to_owned())
        .bind(db_item.block_height)
        .bind(db_item.tx_id.to_owned())
        .bind(db_item.tx_index)
        .bind(db_item.tx_status.to_owned())
        .bind(db_item.r#type.to_owned())
        .bind(db_item.blob_id.to_owned())
        .bind(db_item.created_at)
        .bind(published_at)
        .fetch_one(executor)
        .await
        .map_err(RepositoryError::Insert)?;

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        infra::{Db, DbConnectionOpts, QueryOptions, ToPacket},
        mocks::MockTransaction,
        transactions::subjects::TransactionsSubject,
    };

    async fn test_transaction(tx: &Transaction) -> anyhow::Result<()> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let namespace = QueryOptions::random_namespace();

        let subject = TransactionsSubject {
            block_height: Some(1.into()),
            tx_id: Some(tx.id.clone()),
            tx_index: Some(0),
            tx_status: Some(tx.status.clone()),
            tx_type: Some(tx.tx_type.clone()),
        }
        .dyn_arc();

        let timestamps = BlockTimestamp::default();
        let packet = tx
            .to_packet(&subject, timestamps)
            .with_namespace(&namespace);

        let db_item = TransactionDbItem::try_from(&packet)?;
        let result = Transaction::insert(db.pool_ref(), &db_item).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.value, db_item.value);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.tx_id, db_item.tx_id);
        assert_eq!(result.tx_index, db_item.tx_index);
        assert_eq!(result.tx_status, db_item.tx_status);
        assert_eq!(result.r#type, db_item.r#type);
        assert_eq!(result.blob_id, db_item.blob_id);
        assert_eq!(result.created_at, db_item.created_at);

        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_script_transaction() -> anyhow::Result<()> {
        let tx = MockTransaction::script(vec![], vec![], vec![]);
        test_transaction(&tx).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_create_transaction() -> anyhow::Result<()> {
        let tx = MockTransaction::create(vec![], vec![], vec![]);
        test_transaction(&tx).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_mint_transaction() -> anyhow::Result<()> {
        let tx = MockTransaction::mint(vec![], vec![], vec![]);
        test_transaction(&tx).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_upgrade_transaction() -> anyhow::Result<()> {
        let tx = MockTransaction::upgrade(vec![], vec![], vec![]);
        test_transaction(&tx).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_upload_transaction() -> anyhow::Result<()> {
        let tx = MockTransaction::upload(vec![], vec![], vec![]);
        test_transaction(&tx).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_blob_transaction() -> anyhow::Result<()> {
        let tx = MockTransaction::blob(vec![], vec![], vec![]);
        test_transaction(&tx).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_all_transaction_types() -> anyhow::Result<()> {
        for tx in MockTransaction::all() {
            test_transaction(&tx).await?;
        }
        Ok(())
    }
}
