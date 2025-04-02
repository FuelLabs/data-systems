use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Transaction, TransactionDbItem, TransactionsQuery};
use crate::infra::{
    repository::{Repository, RepositoryError, RepositoryResult},
    DbItem,
};

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
                    subject, value, cursor, block_height, tx_id, tx_index,
                    tx_status, type, blob_id, created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (subject) DO UPDATE SET
                    tx_id = EXCLUDED.tx_id,
                    value = EXCLUDED.value,
                    cursor = EXCLUDED.cursor,
                    block_height = EXCLUDED.block_height,
                    tx_index = EXCLUDED.tx_index,
                    tx_status = EXCLUDED.tx_status,
                    type = EXCLUDED.type,
                    blob_id = EXCLUDED.blob_id,
                    created_at = EXCLUDED.created_at,
                    published_at = $11
                RETURNING *
            )
            SELECT * FROM upsert",
        )
        .bind(&db_item.subject)
        .bind(&db_item.value)
        .bind(db_item.cursor().to_string())
        .bind(db_item.block_height)
        .bind(&db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(&db_item.tx_status)
        .bind(&db_item.r#type)
        .bind(&db_item.blob_id)
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
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        infra::{
            Db,
            DbConnectionOpts,
            DbItem,
            OrderBy,
            QueryOptions,
            QueryParamsBuilder,
        },
        mocks::MockTransaction,
        transactions::DynTransactionSubject,
    };

    async fn test_transaction(tx: &Transaction) -> anyhow::Result<()> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let namespace = QueryOptions::random_namespace();
        let subject = DynTransactionSubject::new(tx, 1.into(), 0);
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(tx, timestamps)
            .with_namespace(&namespace);

        let db_item = TransactionDbItem::try_from(&packet)?;
        let result = Transaction::insert(db.pool_ref(), &db_item).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.cursor(), db_item.cursor());
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

    async fn create_test_transaction(
        height: u32,
        namespace: &str,
    ) -> (TransactionDbItem, Transaction, DynTransactionSubject) {
        let tx = MockTransaction::script(vec![], vec![], vec![]);
        let subject = DynTransactionSubject::new(&tx, height.into(), 0);
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(&tx, timestamps)
            .with_namespace(namespace);
        let db_item = TransactionDbItem::try_from(&packet).unwrap();
        (db_item, tx, subject)
    }

    async fn create_transactions(
        namespace: &str,
        db: &Db,
        count: u32,
    ) -> Vec<TransactionDbItem> {
        let mut transactions = Vec::with_capacity(count as usize);
        for height in 1..=count {
            let (db_item, _, _) =
                create_test_transaction(height, namespace).await;
            Transaction::insert(db.pool_ref(), &db_item).await.unwrap();
            transactions.push(db_item);
        }
        transactions
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

    #[tokio::test]
    async fn test_find_one_transaction() -> Result<()> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let namespace = QueryOptions::random_namespace();
        let tx = MockTransaction::script(vec![], vec![], vec![]);
        let subject = DynTransactionSubject::new(&tx, 1.into(), 0);
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(&tx, timestamps)
            .with_namespace(&namespace);
        let db_item = TransactionDbItem::try_from(&packet)?;

        Transaction::insert(db.pool_ref(), &db_item).await?;
        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));
        let result = Transaction::find_one(db.pool_ref(), &query).await?;

        assert_eq!(result.subject, db_item.subject);
        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_transactions_basic_query() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let transactions = create_transactions(&namespace, &db, 3).await;
        let mut query = TransactionsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three transactions");
        assert_eq!(results[0].subject, transactions[0].subject);
        assert_eq!(results[1].subject, transactions[1].subject);
        assert_eq!(results[2].subject, transactions[2].subject);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_transactions_with_cursor_based_pagination_after(
    ) -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let transactions = create_transactions(&namespace, &db, 5).await;

        let mut query = TransactionsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_after(Some(transactions[1].cursor()));
        query.with_first(Some(2));

        let results = Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 transactions after cursor"
        );
        assert_eq!(results[0].cursor(), transactions[2].cursor());
        assert_eq!(results[1].cursor(), transactions[3].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_transactions_with_cursor_based_pagination_before(
    ) -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let transactions = create_transactions(&namespace, &db, 5).await;
        let mut query = TransactionsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_before(Some(transactions[4].cursor()));
        query.with_last(Some(2));

        let results = Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 transactions before cursor"
        );
        assert_eq!(results[0].cursor(), transactions[3].cursor());
        assert_eq!(results[1].cursor(), transactions[2].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_transactions_with_limit_offset_pagination(
    ) -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let transactions = create_transactions(&namespace, &db, 5).await;

        // Test first page
        let mut query = TransactionsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_limit(Some(2));
        query.with_offset(Some(0));
        query.with_order_by(OrderBy::Asc);

        let first_page = Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            first_page.len(),
            2,
            "First page should have 2 transactions"
        );
        assert_eq!(first_page[0].cursor(), transactions[0].cursor());
        assert_eq!(first_page[1].cursor(), transactions[1].cursor());

        // Test second page
        query.with_offset(Some(2));
        let second_page = Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            second_page.len(),
            2,
            "Second page should have 2 transactions"
        );
        assert_eq!(second_page[0].cursor(), transactions[2].cursor());
        assert_eq!(second_page[1].cursor(), transactions[3].cursor());

        // Test last page
        query.with_offset(Some(4));
        let last_page = Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(last_page.len(), 1, "Last page should have 1 transaction");
        assert_eq!(last_page[0].cursor(), transactions[4].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_transactions_with_different_order() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let transactions = create_transactions(&namespace, &db, 3).await;

        // Test ascending order
        let mut query = TransactionsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_eq!(asc_results[0].cursor(), transactions[0].cursor());
        assert_eq!(asc_results[2].cursor(), transactions[2].cursor());

        // Test descending order
        query.with_order_by(OrderBy::Desc);
        let desc_results =
            Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_eq!(desc_results[0].cursor(), transactions[2].cursor());
        assert_eq!(desc_results[2].cursor(), transactions[0].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_cursor_pagination_ignores_order_by() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let transactions = create_transactions(&namespace, &db, 5).await;

        let mut query = TransactionsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_after(Some(transactions[1].cursor()));
        query.with_first(Some(2));

        let results_default =
            Transaction::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Asc);

        let results_asc = Transaction::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Desc);

        let results_desc =
            Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results_default, results_asc);
        assert_eq!(results_default, results_desc);

        Ok(())
    }
}
