use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Receipt, ReceiptDbItem, ReceiptsQuery};
use crate::infra::{
    repository::{Repository, RepositoryError, RepositoryResult},
    DbItem,
};

#[async_trait]
impl Repository for Receipt {
    type Item = ReceiptDbItem;
    type QueryParams = ReceiptsQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, ReceiptDbItem>(
            "WITH upsert AS (
                INSERT INTO receipts (
                    subject, value, cursor, block_height, tx_id, tx_index, receipt_index,
                    receipt_type, from_contract_id, to_contract_id, to_address,
                    asset_id, contract_id, sub_id, sender_address, recipient_address,
                    created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    cursor = EXCLUDED.cursor,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    receipt_index = EXCLUDED.receipt_index,
                    receipt_type = EXCLUDED.receipt_type,
                    from_contract_id = EXCLUDED.from_contract_id,
                    to_contract_id = EXCLUDED.to_contract_id,
                    to_address = EXCLUDED.to_address,
                    asset_id = EXCLUDED.asset_id,
                    contract_id = EXCLUDED.contract_id,
                    sub_id = EXCLUDED.sub_id,
                    sender_address = EXCLUDED.sender_address,
                    recipient_address = EXCLUDED.recipient_address,
                    created_at = EXCLUDED.created_at,
                    published_at = $18
                RETURNING *
            )
            SELECT * FROM upsert"
        )
        .bind(db_item.subject.clone())
        .bind(&db_item.value)
        .bind(db_item.cursor().to_string())
        .bind(db_item.block_height)
        .bind(&db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.receipt_index)
        .bind(&db_item.receipt_type)
        .bind(&db_item.from_contract_id)
        .bind(&db_item.to_contract_id)
        .bind(&db_item.to_address)
        .bind(&db_item.asset_id)
        .bind(&db_item.contract_id)
        .bind(&db_item.sub_id)
        .bind(&db_item.sender_address)
        .bind(&db_item.recipient_address)
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
            OrderBy,
            QueryOptions,
            QueryParamsBuilder,
        },
        mocks::{MockReceipt, MockTransaction},
        receipts::DynReceiptSubject,
    };

    async fn test_receipt(receipt: &Receipt) -> Result<()> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let tx =
            MockTransaction::script(vec![], vec![], vec![receipt.to_owned()]);
        let namespace = QueryOptions::random_namespace();
        let subject = DynReceiptSubject::new(receipt, 1.into(), tx.id, 0, 0);
        let timestamps = BlockTimestamp::default();

        let packet = subject
            .build_packet(receipt, timestamps)
            .with_namespace(&namespace);

        let db_item = ReceiptDbItem::try_from(&packet)?;
        let result = Receipt::insert(db.pool_ref(), &db_item).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(db_item.cursor(), result.cursor());
        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.value, db_item.value);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.tx_id, db_item.tx_id);
        assert_eq!(result.tx_index, db_item.tx_index);
        assert_eq!(result.receipt_index, db_item.receipt_index);
        assert_eq!(result.receipt_type, db_item.receipt_type);
        assert_eq!(result.from_contract_id, db_item.from_contract_id);
        assert_eq!(result.to_contract_id, db_item.to_contract_id);
        assert_eq!(result.to_address, db_item.to_address);
        assert_eq!(result.asset_id, db_item.asset_id);
        assert_eq!(result.contract_id, db_item.contract_id);
        assert_eq!(result.sub_id, db_item.sub_id);
        assert_eq!(result.sender_address, db_item.sender_address);
        assert_eq!(result.recipient_address, db_item.recipient_address);
        assert_eq!(result.created_at, db_item.created_at);

        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_call() -> Result<()> {
        test_receipt(&MockReceipt::call()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_return() -> Result<()> {
        test_receipt(&MockReceipt::return_receipt()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_return_data() -> Result<()> {
        test_receipt(&MockReceipt::return_data()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_panic() -> Result<()> {
        test_receipt(&MockReceipt::panic()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_revert() -> Result<()> {
        test_receipt(&MockReceipt::revert()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_log() -> Result<()> {
        test_receipt(&MockReceipt::log()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_log_data() -> Result<()> {
        test_receipt(&MockReceipt::log_data()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_transfer() -> Result<()> {
        test_receipt(&MockReceipt::transfer()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_transfer_out() -> Result<()> {
        test_receipt(&MockReceipt::transfer_out()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_script_result() -> Result<()> {
        test_receipt(&MockReceipt::script_result()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_message_out() -> Result<()> {
        test_receipt(&MockReceipt::message_out()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_mint() -> Result<()> {
        test_receipt(&MockReceipt::mint()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_burn() -> Result<()> {
        test_receipt(&MockReceipt::burn()).await?;
        Ok(())
    }

    async fn create_test_receipt(
        height: u32,
        namespace: &str,
    ) -> (ReceiptDbItem, Receipt, DynReceiptSubject) {
        let receipt = MockReceipt::call();
        let tx =
            MockTransaction::script(vec![], vec![], vec![receipt.to_owned()]);
        let subject =
            DynReceiptSubject::new(&receipt, height.into(), tx.id, 0, 0);
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(&receipt, timestamps)
            .with_namespace(namespace);
        let db_item = ReceiptDbItem::try_from(&packet).unwrap();
        (db_item, receipt, subject)
    }

    async fn create_receipts(
        namespace: &str,
        db: &Db,
        count: u32,
    ) -> Vec<ReceiptDbItem> {
        let mut receipts = Vec::with_capacity(count as usize);
        for height in 1..=count {
            let (db_item, _, _) = create_test_receipt(height, namespace).await;
            Receipt::insert(db.pool_ref(), &db_item).await.unwrap();
            receipts.push(db_item);
        }
        receipts
    }

    #[tokio::test]
    async fn test_find_one_receipt() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let (db_item, _, subject) = create_test_receipt(1, &namespace).await;
        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));

        Receipt::insert(db.pool_ref(), &db_item).await?;
        let result = Receipt::find_one(db.pool_ref(), &query).await?;
        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.value, db_item.value);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.tx_id, db_item.tx_id);
        assert_eq!(result.receipt_type, db_item.receipt_type);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_receipts_basic_query() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let receipts = create_receipts(&namespace, &db, 3).await;
        let mut query = ReceiptsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three receipts");
        assert_eq!(results[0].subject, receipts[0].subject);
        assert_eq!(results[1].subject, receipts[1].subject);
        assert_eq!(results[2].subject, receipts[2].subject);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_receipts_with_cursor_based_pagination_after(
    ) -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let receipts = create_receipts(&namespace, &db, 5).await;

        let mut query = ReceiptsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_after(Some(receipts[1].cursor()));
        query.with_first(Some(2));

        let results = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 receipts after cursor"
        );
        assert_eq!(results[0].cursor(), receipts[2].cursor());
        assert_eq!(results[1].cursor(), receipts[3].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_receipts_with_cursor_based_pagination_before(
    ) -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let receipts = create_receipts(&namespace, &db, 5).await;
        let mut query = ReceiptsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_before(Some(receipts[4].cursor()));
        query.with_last(Some(2));

        let results = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 receipts before cursor"
        );
        assert_eq!(results[0].cursor(), receipts[3].cursor());
        assert_eq!(results[1].cursor(), receipts[2].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_receipts_with_limit_offset_pagination() -> Result<()>
    {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let receipts = create_receipts(&namespace, &db, 5).await;

        // Test first page
        let mut query = ReceiptsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_limit(Some(2));
        query.with_offset(Some(0));
        query.with_order_by(OrderBy::Asc);

        let first_page = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(first_page.len(), 2, "First page should have 2 receipts");
        assert_eq!(first_page[0].cursor(), receipts[0].cursor());
        assert_eq!(first_page[1].cursor(), receipts[1].cursor());

        // Test second page
        query.with_offset(Some(2));
        let second_page = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(second_page.len(), 2, "Second page should have 2 receipts");
        assert_eq!(second_page[0].cursor(), receipts[2].cursor());
        assert_eq!(second_page[1].cursor(), receipts[3].cursor());

        // Test last page
        query.with_offset(Some(4));
        let last_page = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(last_page.len(), 1, "Last page should have 1 receipt");
        assert_eq!(last_page[0].cursor(), receipts[4].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_receipts_with_different_order() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let receipts = create_receipts(&namespace, &db, 3).await;

        // Test ascending order
        let mut query = ReceiptsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_eq!(asc_results[0].cursor(), receipts[0].cursor());
        assert_eq!(asc_results[2].cursor(), receipts[2].cursor());

        // Test descending order
        query.with_order_by(OrderBy::Desc);
        let desc_results = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_eq!(desc_results[0].cursor(), receipts[2].cursor());
        assert_eq!(desc_results[2].cursor(), receipts[0].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_cursor_pagination_ignores_order_by() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let receipts = create_receipts(&namespace, &db, 5).await;

        let mut query = ReceiptsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_after(Some(receipts[1].cursor()));
        query.with_first(Some(2));

        let results_default = Receipt::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Asc);

        let results_asc = Receipt::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Desc);

        let results_desc = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results_default, results_asc);
        assert_eq!(results_default, results_desc);

        Ok(())
    }
}
