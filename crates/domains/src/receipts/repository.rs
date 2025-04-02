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
    use std::sync::Arc;

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

    async fn setup_db() -> anyhow::Result<(Arc<Db>, String)> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let namespace = QueryOptions::random_namespace();
        Ok((db, namespace))
    }

    fn assert_result(result: &ReceiptDbItem, expected: &ReceiptDbItem) {
        assert_eq!(result.cursor(), expected.cursor());
        assert_eq!(result.subject, expected.subject);
        assert_eq!(result.value, expected.value);
        assert_eq!(result.block_height, expected.block_height);
        assert_eq!(result.tx_id, expected.tx_id);
        assert_eq!(result.tx_index, expected.tx_index);
        assert_eq!(result.receipt_index, expected.receipt_index);
        assert_eq!(result.receipt_type, expected.receipt_type);
        assert_eq!(result.from_contract_id, expected.from_contract_id);
        assert_eq!(result.to_contract_id, expected.to_contract_id);
        assert_eq!(result.to_address, expected.to_address);
        assert_eq!(result.asset_id, expected.asset_id);
        assert_eq!(result.contract_id, expected.contract_id);
        assert_eq!(result.sub_id, expected.sub_id);
        assert_eq!(result.sender_address, expected.sender_address);
        assert_eq!(result.recipient_address, expected.recipient_address);
        assert_eq!(result.created_at, expected.created_at);
    }

    async fn insert_receipt(
        db: &Arc<Db>,
        receipt: Option<Receipt>,
        height: u32,
        namespace: &str,
    ) -> Result<(ReceiptDbItem, Receipt, DynReceiptSubject)> {
        let receipt = receipt.unwrap_or_else(MockReceipt::call);
        let tx =
            MockTransaction::script(vec![], vec![], vec![receipt.to_owned()]);

        let subject =
            DynReceiptSubject::new(&receipt, height.into(), tx.id, 0, 0);
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(&receipt, timestamps)
            .with_namespace(namespace);

        let db_item = ReceiptDbItem::try_from(&packet)?;
        let result = Receipt::insert(db.pool_ref(), &db_item).await?;
        assert_result(&result, &db_item);

        Ok((db_item, receipt, subject))
    }

    async fn create_receipts(
        db: &Arc<Db>,
        namespace: &str,
        count: u32,
    ) -> Result<Vec<ReceiptDbItem>> {
        let mut receipts = Vec::with_capacity(count as usize);
        for height in 1..=count {
            let (db_item, _, _) =
                insert_receipt(db, None, height, namespace).await?;
            receipts.push(db_item);
        }
        Ok(receipts)
    }

    #[tokio::test]
    async fn test_inserting_receipt_call() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_receipt(&db, Some(MockReceipt::call()), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_return() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_receipt(&db, Some(MockReceipt::return_receipt()), 1, &namespace)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_return_data() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_receipt(&db, Some(MockReceipt::return_data()), 1, &namespace)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_panic() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_receipt(&db, Some(MockReceipt::panic()), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_revert() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_receipt(&db, Some(MockReceipt::revert()), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_log() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_receipt(&db, Some(MockReceipt::log()), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_log_data() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_receipt(&db, Some(MockReceipt::log_data()), 1, &namespace)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_transfer() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_receipt(&db, Some(MockReceipt::transfer()), 1, &namespace)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_transfer_out() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_receipt(&db, Some(MockReceipt::transfer_out()), 1, &namespace)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_script_result() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_receipt(&db, Some(MockReceipt::script_result()), 1, &namespace)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_message_out() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_receipt(&db, Some(MockReceipt::message_out()), 1, &namespace)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_mint() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_receipt(&db, Some(MockReceipt::mint()), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_burn() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_receipt(&db, Some(MockReceipt::burn()), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_one_receipt() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let (db_item, _, subject) =
            insert_receipt(&db, None, 1, &namespace).await?;

        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));

        let result = Receipt::find_one(db.pool_ref(), &query).await?;
        assert_result(&result, &db_item);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_receipts_basic_query() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let receipts = create_receipts(&db, &namespace, 3).await?;

        let mut query = ReceiptsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three receipts");
        assert_result(&results[0], &receipts[0]);
        assert_result(&results[1], &receipts[1]);
        assert_result(&results[2], &receipts[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_receipts_with_cursor_based_pagination_after(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let receipts = create_receipts(&db, &namespace, 5).await?;

        let mut query = ReceiptsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_after(Some(receipts[1].cursor()));
        query.with_first(Some(2));

        let results = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 receipts after cursor"
        );
        assert_result(&results[0], &receipts[2]);
        assert_result(&results[1], &receipts[3]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_receipts_with_cursor_based_pagination_before(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let receipts = create_receipts(&db, &namespace, 5).await?;

        let mut query = ReceiptsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_before(Some(receipts[4].cursor()));
        query.with_last(Some(2));

        let results = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 receipts before cursor"
        );
        assert_result(&results[0], &receipts[3]);
        assert_result(&results[1], &receipts[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_receipts_with_limit_offset_pagination() -> Result<()>
    {
        let (db, namespace) = setup_db().await?;
        let receipts = create_receipts(&db, &namespace, 5).await?;

        let mut query = ReceiptsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_limit(Some(2));
        query.with_offset(Some(1));
        query.with_order_by(OrderBy::Asc);

        let results = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 2, "Should return exactly 2 receipts");
        assert_result(&results[0], &receipts[1]);
        assert_result(&results[1], &receipts[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_receipts_with_different_order() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let receipts = create_receipts(&db, &namespace, 3).await?;

        let mut query = ReceiptsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_result(&asc_results[0], &receipts[0]);
        assert_result(&asc_results[2], &receipts[2]);

        query.with_order_by(OrderBy::Desc);
        let desc_results = Receipt::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_result(&desc_results[0], &receipts[2]);
        assert_result(&desc_results[2], &receipts[0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_cursor_pagination_ignores_order_by() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let receipts = create_receipts(&db, &namespace, 5).await?;

        let mut query = ReceiptsQuery::default();
        query.with_namespace(Some(namespace));
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
