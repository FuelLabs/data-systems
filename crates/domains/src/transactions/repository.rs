use async_trait::async_trait;
use fuel_data_parser::DataEncoder;
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
        let mut conn = executor.acquire().await?;
        let mut db_tx = conn.begin().await?;
        let published_at = BlockTimestamp::now();
        // Insert into transactions table
        let record = sqlx::query_as::<_, TransactionDbItem>(
            "WITH upsert AS (
                INSERT INTO transactions (
                    subject,
                    value,
                    block_height,
                    tx_id,
                    tx_index,
                    cursor,
                    type,
                    script_gas_limit,
                    mint_amount,
                    mint_asset_id,
                    mint_gas_price,
                    receipts_root,
                    tx_status,
                    script,
                    script_data,
                    salt,
                    bytecode_witness_index,
                    bytecode_root,
                    subsection_index,
                    subsections_number,
                    upgrade_purpose,
                    blob_id,
                    maturity,
                    policies,
                    script_length,
                    script_data_length,
                    storage_slots_count,
                    proof_set_count,
                    witnesses_count,
                    inputs_count,
                    outputs_count,
                    block_time,
                    created_at,
                    published_at
                )
                VALUES (
                    $1, $2, $3, $4, $5, $6, $7::transaction_type, $8, $9, $10,
                    $11, $12, $13::transaction_status, $14, $15, $16, $17, $18,
                    $19, $20, $21, $22, $23, $24, $25, $26, $27, $28,
                    $29, $30, $31, $32, $33, $34
                )
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    cursor = EXCLUDED.cursor,
                    type = EXCLUDED.type,
                    script_gas_limit = EXCLUDED.script_gas_limit,
                    mint_amount = EXCLUDED.mint_amount,
                    mint_asset_id = EXCLUDED.mint_asset_id,
                    mint_gas_price = EXCLUDED.mint_gas_price,
                    receipts_root = EXCLUDED.receipts_root,
                    tx_status = EXCLUDED.tx_status,
                    script = EXCLUDED.script,
                    script_data = EXCLUDED.script_data,
                    salt = EXCLUDED.salt,
                    bytecode_witness_index = EXCLUDED.bytecode_witness_index,
                    bytecode_root = EXCLUDED.bytecode_root,
                    subsection_index = EXCLUDED.subsection_index,
                    subsections_number = EXCLUDED.subsections_number,
                    upgrade_purpose = EXCLUDED.upgrade_purpose,
                    blob_id = EXCLUDED.blob_id,
                    maturity = EXCLUDED.maturity,
                    policies = EXCLUDED.policies,
                    script_length = EXCLUDED.script_length,
                    script_data_length = EXCLUDED.script_data_length,
                    storage_slots_count = EXCLUDED.storage_slots_count,
                    proof_set_count = EXCLUDED.proof_set_count,
                    witnesses_count = EXCLUDED.witnesses_count,
                    inputs_count = EXCLUDED.inputs_count,
                    outputs_count = EXCLUDED.outputs_count,
                    block_time = EXCLUDED.block_time,
                    created_at = EXCLUDED.created_at,
                    published_at = $34
                RETURNING *
            )
            SELECT * FROM upsert",
        )
        .bind(&db_item.subject)
        .bind(&db_item.value)
        .bind(db_item.block_height)
        .bind(&db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.cursor().to_string())
        .bind(db_item.r#type)
        .bind(db_item.script_gas_limit)
        .bind(db_item.mint_amount)
        .bind(&db_item.mint_asset_id)
        .bind(db_item.mint_gas_price)
        .bind(&db_item.receipts_root)
        .bind(db_item.tx_status)
        .bind(&db_item.script)
        .bind(&db_item.script_data)
        .bind(&db_item.salt)
        .bind(db_item.bytecode_witness_index)
        .bind(&db_item.bytecode_root)
        .bind(db_item.subsection_index)
        .bind(db_item.subsections_number)
        .bind(&db_item.upgrade_purpose)
        .bind(&db_item.blob_id)
        .bind(db_item.maturity)
        .bind(&db_item.policies)
        .bind(db_item.script_length)
        .bind(db_item.script_data_length)
        .bind(db_item.storage_slots_count)
        .bind(db_item.proof_set_count)
        .bind(db_item.witnesses_count)
        .bind(db_item.inputs_count)
        .bind(db_item.outputs_count)
        .bind(db_item.block_time)
        .bind(db_item.created_at)
        .bind(published_at)
        .fetch_one(&mut *db_tx)
        .await
        .map_err(RepositoryError::Insert)?;

        let tx = Transaction::decode_json(&db_item.value)?;
        for slot in &tx.storage_slots {
            let slot_item = super::db_item::TransactionStorageSlotDbItem {
                tx_id: db_item.tx_id.clone(),
                key: slot.key.to_string(),
                value: slot.value.to_string(),
                created_at: db_item.created_at,
            };
            sqlx::query(
                "INSERT INTO transaction_storage_slots (tx_id, key, value, created_at)
                 VALUES ($1, $2, $3, $4)"
            )
            .bind(&slot_item.tx_id)
            .bind(&slot_item.key)
            .bind(&slot_item.value)
            .bind(slot_item.created_at)
            .execute(&mut *db_tx)
            .await
            .map_err(RepositoryError::Insert)?;
        }

        // Insert witnesses
        for witness in &tx.witnesses {
            let witness_item = super::db_item::TransactionWitnessDbItem {
                tx_id: db_item.tx_id.clone(),
                witness_data: witness.to_string(),
                witness_data_length: witness.as_ref().0.len() as i32,
                created_at: db_item.created_at,
            };
            sqlx::query(
                "INSERT INTO transaction_witnesses (tx_id, witness_data, witness_data_length, created_at)
                 VALUES ($1, $2, $3, $4)"
            )
            .bind(&witness_item.tx_id)
            .bind(&witness_item.witness_data)
            .bind(witness_item.witness_data_length)
            .bind(witness_item.created_at)
            .execute(&mut *db_tx)
            .await
            .map_err(RepositoryError::Insert)?;
        }

        // Insert proof set
        for proof in &tx.proof_set {
            let proof_item = super::db_item::TransactionProofSetDbItem {
                tx_id: db_item.tx_id.clone(),
                proof_hash: proof.to_string(),
                created_at: db_item.created_at,
            };
            sqlx::query(
                "INSERT INTO transaction_proof_set (tx_id, proof_hash, created_at)
                 VALUES ($1, $2, $3)"
            )
            .bind(&proof_item.tx_id)
            .bind(&proof_item.proof_hash)
            .bind(proof_item.created_at)
            .execute(&mut *db_tx)
            .await
            .map_err(RepositoryError::Insert)?;
        }

        db_tx.commit().await?;
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
        assert_eq!(result.script_gas_limit, db_item.script_gas_limit);
        assert_eq!(result.mint_amount, db_item.mint_amount);
        assert_eq!(result.mint_asset_id, db_item.mint_asset_id);
        assert_eq!(result.mint_gas_price, db_item.mint_gas_price);
        assert_eq!(result.receipts_root, db_item.receipts_root);
        assert_eq!(result.script, db_item.script);
        assert_eq!(result.script_data, db_item.script_data);
        assert_eq!(result.salt, db_item.salt);
        assert_eq!(
            result.bytecode_witness_index,
            db_item.bytecode_witness_index
        );
        assert_eq!(result.bytecode_root, db_item.bytecode_root);
        assert_eq!(result.subsection_index, db_item.subsection_index);
        assert_eq!(result.subsections_number, db_item.subsections_number);
        assert_eq!(result.upgrade_purpose, db_item.upgrade_purpose);
        assert_eq!(result.blob_id, db_item.blob_id);
        assert_eq!(result.maturity, db_item.maturity);
        assert_eq!(result.policies, db_item.policies);
        assert_eq!(result.script_length, db_item.script_length);
        assert_eq!(result.script_data_length, db_item.script_data_length);
        assert_eq!(result.storage_slots_count, db_item.storage_slots_count);
        assert_eq!(result.proof_set_count, db_item.proof_set_count);
        assert_eq!(result.witnesses_count, db_item.witnesses_count);
        assert_eq!(result.inputs_count, db_item.inputs_count);
        assert_eq!(result.outputs_count, db_item.outputs_count);
        assert_eq!(result.block_time, db_item.block_time);
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
