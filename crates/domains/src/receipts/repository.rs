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
        let created_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, ReceiptDbItem>(
            r#"
            INSERT INTO receipts (
                subject,
                value,
                block_height,
                tx_id,
                tx_index,
                receipt_index,
                cursor,
                type,
                from_contract_id,
                to_contract_id,
                amount,
                asset_id,
                gas,
                param1,
                param2,
                contract_id,
                pc,
                "is",
                val,
                ptr,
                len,
                digest,
                data,
                ra,
                rb,
                rc,
                rd,
                to_address,
                panic_reason,
                panic_instruction,
                result,
                gas_used,
                sender_address,
                recipient_address,
                nonce,
                sub_id,
                block_time,
                created_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8::receipt_type, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                $21, $22, $23, $24, $25, $26, $27, $28, $29, $30,
                $31, $32, $33, $34, $35, $36, $37, $38
            )
            ON CONFLICT (subject) DO UPDATE SET
                value = EXCLUDED.value,
                block_height = EXCLUDED.block_height,
                tx_id = EXCLUDED.tx_id,
                tx_index = EXCLUDED.tx_index,
                receipt_index = EXCLUDED.receipt_index,
                cursor = EXCLUDED.cursor,
                type = EXCLUDED.type,
                from_contract_id = EXCLUDED.from_contract_id,
                to_contract_id = EXCLUDED.to_contract_id,
                amount = EXCLUDED.amount,
                asset_id = EXCLUDED.asset_id,
                gas = EXCLUDED.gas,
                param1 = EXCLUDED.param1,
                param2 = EXCLUDED.param2,
                contract_id = EXCLUDED.contract_id,
                pc = EXCLUDED.pc,
                "is" = EXCLUDED."is",
                val = EXCLUDED.val,
                ptr = EXCLUDED.ptr,
                len = EXCLUDED.len,
                digest = EXCLUDED.digest,
                data = EXCLUDED.data,
                ra = EXCLUDED.ra,
                rb = EXCLUDED.rb,
                rc = EXCLUDED.rc,
                rd = EXCLUDED.rd,
                to_address = EXCLUDED.to_address,
                panic_reason = EXCLUDED.panic_reason,
                panic_instruction = EXCLUDED.panic_instruction,
                result = EXCLUDED.result,
                gas_used = EXCLUDED.gas_used,
                sender_address = EXCLUDED.sender_address,
                recipient_address = EXCLUDED.recipient_address,
                nonce = EXCLUDED.nonce,
                sub_id = EXCLUDED.sub_id,
                block_time = EXCLUDED.block_time,
                created_at = EXCLUDED.created_at
            RETURNING *
            "#,
        )
        .bind(&db_item.subject)
        .bind(&db_item.value)
        .bind(db_item.block_height.into_inner() as i64)
        .bind(&db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.receipt_index)
        .bind(db_item.cursor().to_string())
        .bind(db_item.r#type)
        .bind(&db_item.from_contract_id)
        .bind(&db_item.to_contract_id)
        .bind(db_item.amount)
        .bind(&db_item.asset_id)
        .bind(db_item.gas)
        .bind(db_item.param1)
        .bind(db_item.param2)
        .bind(&db_item.contract_id)
        .bind(db_item.pc)
        .bind(db_item.is)
        .bind(db_item.val)
        .bind(db_item.ptr)
        .bind(db_item.len)
        .bind(&db_item.digest)
        .bind(&db_item.data)
        .bind(db_item.ra)
        .bind(db_item.rb)
        .bind(db_item.rc)
        .bind(db_item.rd)
        .bind(&db_item.to_address)
        .bind(&db_item.panic_reason)
        .bind(db_item.panic_instruction)
        .bind(&db_item.result)
        .bind(db_item.gas_used)
        .bind(&db_item.sender_address)
        .bind(&db_item.recipient_address)
        .bind(&db_item.nonce)
        .bind(&db_item.sub_id)
        .bind(db_item.block_time)
        .bind(created_at)
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
    use fuel_streams_types::BlockHeight;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        blocks::{
            packets::DynBlockSubject,
            repository::tests::insert_block,
            Block,
            BlockDbItem,
        },
        infra::{
            Db,
            DbConnectionOpts,
            OrderBy,
            QueryOptions,
            QueryParamsBuilder,
            RecordPointer,
        },
        mocks::{MockReceipt, MockTransaction},
        receipts::DynReceiptSubject,
        transactions::{
            repository::tests::insert_transaction,
            DynTransactionSubject,
            Transaction,
            TransactionDbItem,
        },
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
        assert_eq!(result.r#type, expected.r#type);
        assert_eq!(result.from_contract_id, expected.from_contract_id);
        assert_eq!(result.to_contract_id, expected.to_contract_id);
        assert_eq!(result.amount, expected.amount);
        assert_eq!(result.asset_id, expected.asset_id);
        assert_eq!(result.gas, expected.gas);
        assert_eq!(result.param1, expected.param1);
        assert_eq!(result.param2, expected.param2);
        assert_eq!(result.contract_id, expected.contract_id);
        assert_eq!(result.pc, expected.pc);
        assert_eq!(result.is, expected.is);
        assert_eq!(result.val, expected.val);
        assert_eq!(result.ptr, expected.ptr);
        assert_eq!(result.len, expected.len);
        assert_eq!(result.digest, expected.digest);
        assert_eq!(result.data, expected.data);
        assert_eq!(result.ra, expected.ra);
        assert_eq!(result.rb, expected.rb);
        assert_eq!(result.rc, expected.rc);
        assert_eq!(result.rd, expected.rd);
        assert_eq!(result.to_address, expected.to_address);
        assert_eq!(result.panic_reason, expected.panic_reason);
        assert_eq!(result.panic_instruction, expected.panic_instruction);
        assert_eq!(result.result, expected.result);
        assert_eq!(result.gas_used, expected.gas_used);
        assert_eq!(result.sender_address, expected.sender_address);
        assert_eq!(result.recipient_address, expected.recipient_address);
        assert_eq!(result.nonce, expected.nonce);
        assert_eq!(result.sub_id, expected.sub_id);
    }

    async fn insert_random_block(
        db: &Arc<Db>,
        height: BlockHeight,
        namespace: &str,
    ) -> Result<(BlockDbItem, Block, DynBlockSubject)> {
        let (db_item, block, subject) =
            insert_block(db, height, namespace).await?;
        Ok((db_item, block, subject))
    }

    async fn insert_tx(
        db: &Arc<Db>,
        tx: &Transaction,
        height: BlockHeight,
        namespace: &str,
    ) -> Result<(TransactionDbItem, Transaction, DynTransactionSubject)> {
        let _ = insert_random_block(db, height, namespace).await?;
        insert_transaction(db, Some(tx.clone()), height, namespace).await
    }

    async fn insert_receipt(
        db: &Arc<Db>,
        tx: &Transaction,
        receipt: &Receipt,
        height: BlockHeight,
        namespace: &str,
        (tx_index, receipt_index): (i32, i32),
    ) -> Result<(ReceiptDbItem, Receipt, DynReceiptSubject)> {
        let subject = DynReceiptSubject::new(
            receipt,
            height,
            tx.id.to_owned(),
            tx_index,
            receipt_index,
        );
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(receipt, timestamps, RecordPointer {
                block_height: height,
                tx_id: Some(tx.id.to_owned()),
                tx_index: Some(tx_index as u32),
                receipt_index: Some(receipt_index as u32),
                ..Default::default()
            })
            .with_namespace(namespace);

        let db_item = ReceiptDbItem::try_from(&packet)?;
        let result = Receipt::insert(db.pool_ref(), &db_item).await?;
        assert_result(&result, &db_item);

        Ok((db_item, receipt.clone(), subject))
    }

    async fn create_receipts(
        db: &Arc<Db>,
        namespace: &str,
        count: u32,
    ) -> Result<Vec<ReceiptDbItem>> {
        let mut receipts = Vec::with_capacity(count as usize);
        for _ in 0..count {
            receipts.push(MockReceipt::call())
        }

        let height = BlockHeight::random();
        let tx = MockTransaction::script(vec![], vec![], receipts.clone());
        insert_tx(db, &tx, height, namespace).await?;

        let mut db_items = Vec::with_capacity(count as usize);
        for (index, receipt) in receipts.iter().enumerate() {
            let (db_item, _, _) = insert_receipt(
                db,
                &tx,
                receipt,
                height,
                namespace,
                (0, index as i32),
            )
            .await?;
            db_items.push(db_item);
        }
        db_items.sort_by_key(|i| i.cursor());
        Ok(db_items)
    }

    #[tokio::test]
    async fn test_inserting_receipt_call() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt1 = MockReceipt::call();
        let receipt2 = MockReceipt::call();
        let tx = MockTransaction::script(vec![], vec![], vec![
            receipt1.clone(),
            receipt2.clone(),
        ]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_receipt(&db, &tx, &receipt1, height, &namespace, (0, 0)).await?;
        insert_receipt(&db, &tx, &receipt2, height, &namespace, (0, 1)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_return() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt = MockReceipt::return_receipt();
        let tx = MockTransaction::script(vec![], vec![], vec![receipt.clone()]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_receipt(&db, &tx, &receipt, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_return_data() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt = MockReceipt::return_data();
        let tx = MockTransaction::script(vec![], vec![], vec![receipt.clone()]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_receipt(&db, &tx, &receipt, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_panic() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt = MockReceipt::panic();
        let tx = MockTransaction::script(vec![], vec![], vec![receipt.clone()]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_receipt(&db, &tx, &receipt, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_revert() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt = MockReceipt::revert();
        let tx = MockTransaction::script(vec![], vec![], vec![receipt.clone()]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_receipt(&db, &tx, &receipt, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_log() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt = MockReceipt::log();
        let tx = MockTransaction::script(vec![], vec![], vec![receipt.clone()]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_receipt(&db, &tx, &receipt, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_log_data() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt = MockReceipt::log_data();
        let tx = MockTransaction::script(vec![], vec![], vec![receipt.clone()]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_receipt(&db, &tx, &receipt, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_transfer() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt = MockReceipt::transfer();
        let tx = MockTransaction::script(vec![], vec![], vec![receipt.clone()]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_receipt(&db, &tx, &receipt, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_transfer_out() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt = MockReceipt::transfer_out();
        let tx = MockTransaction::script(vec![], vec![], vec![receipt.clone()]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_receipt(&db, &tx, &receipt, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_script_result() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt = MockReceipt::script_result();
        let tx = MockTransaction::script(vec![], vec![], vec![receipt.clone()]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_receipt(&db, &tx, &receipt, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_message_out() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt = MockReceipt::message_out();
        let tx = MockTransaction::script(vec![], vec![], vec![receipt.clone()]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_receipt(&db, &tx, &receipt, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_mint() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt = MockReceipt::mint();
        let tx = MockTransaction::script(vec![], vec![], vec![receipt.clone()]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_receipt(&db, &tx, &receipt, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_burn() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt = MockReceipt::burn();
        let tx = MockTransaction::script(vec![], vec![], vec![receipt.clone()]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_receipt(&db, &tx, &receipt, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_one_receipt() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let receipt = MockReceipt::call();
        let tx = MockTransaction::script(vec![], vec![], vec![receipt.clone()]);
        insert_tx(&db, &tx, height, &namespace).await?;

        let (db_item, _, subject) =
            insert_receipt(&db, &tx, &receipt, height, &namespace, (0, 0))
                .await?;
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
