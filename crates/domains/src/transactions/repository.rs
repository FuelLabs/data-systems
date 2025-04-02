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
        let created_at = BlockTimestamp::now();
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
                    created_at
                )
                VALUES (
                    $1, $2, $3, $4, $5, $6, $7::transaction_type, $8, $9, $10,
                    $11, $12, $13::transaction_status, $14, $15, $16, $17, $18,
                    $19, $20, $21, $22, $23, $24, $25, $26, $27, $28,
                    $29, $30, $31, $32, $33
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
                    created_at = EXCLUDED.created_at
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
        .bind(created_at)
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
pub mod tests {
    use std::sync::Arc;

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

    async fn setup_db() -> anyhow::Result<(Arc<Db>, String)> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let namespace = QueryOptions::random_namespace();
        Ok((db, namespace))
    }

    fn assert_result(result: &TransactionDbItem, expected: &TransactionDbItem) {
        assert_eq!(result.cursor(), expected.cursor());
        assert_eq!(result.subject, expected.subject);
        assert_eq!(result.value, expected.value);
        assert_eq!(result.block_height, expected.block_height);
        assert_eq!(result.tx_id, expected.tx_id);
        assert_eq!(result.tx_index, expected.tx_index);
        assert_eq!(result.tx_status, expected.tx_status);
        assert_eq!(result.r#type, expected.r#type);
        assert_eq!(result.script_gas_limit, expected.script_gas_limit);
        assert_eq!(result.mint_amount, expected.mint_amount);
        assert_eq!(result.mint_asset_id, expected.mint_asset_id);
        assert_eq!(result.mint_gas_price, expected.mint_gas_price);
        assert_eq!(result.receipts_root, expected.receipts_root);
        assert_eq!(result.script, expected.script);
        assert_eq!(result.script_data, expected.script_data);
        assert_eq!(result.salt, expected.salt);
        assert_eq!(
            result.bytecode_witness_index,
            expected.bytecode_witness_index
        );
        assert_eq!(result.bytecode_root, expected.bytecode_root);
        assert_eq!(result.subsection_index, expected.subsection_index);
        assert_eq!(result.subsections_number, expected.subsections_number);
        assert_eq!(result.upgrade_purpose, expected.upgrade_purpose);
        assert_eq!(result.blob_id, expected.blob_id);
        assert_eq!(result.maturity, expected.maturity);
        assert_eq!(result.policies, expected.policies);
        assert_eq!(result.script_length, expected.script_length);
        assert_eq!(result.script_data_length, expected.script_data_length);
        assert_eq!(result.storage_slots_count, expected.storage_slots_count);
        assert_eq!(result.proof_set_count, expected.proof_set_count);
        assert_eq!(result.witnesses_count, expected.witnesses_count);
        assert_eq!(result.inputs_count, expected.inputs_count);
        assert_eq!(result.outputs_count, expected.outputs_count);
        assert_eq!(result.block_time, expected.block_time);
    }

    pub async fn insert_transaction(
        db: &Arc<Db>,
        tx: Option<Transaction>,
        height: u32,
        namespace: &str,
    ) -> Result<(TransactionDbItem, Transaction, DynTransactionSubject)> {
        let tx = tx
            .unwrap_or_else(|| MockTransaction::script(vec![], vec![], vec![]));
        let subject = DynTransactionSubject::new(&tx, height.into(), 0);
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(&tx, timestamps)
            .with_namespace(namespace);

        let db_item = TransactionDbItem::try_from(&packet)?;
        let result = Transaction::insert(db.pool_ref(), &db_item).await?;
        assert_result(&result, &db_item);

        Ok((db_item, tx, subject))
    }

    async fn create_transactions(
        db: &Arc<Db>,
        namespace: &str,
        count: u32,
    ) -> Result<Vec<TransactionDbItem>> {
        let mut transactions = Vec::with_capacity(count as usize);
        for height in 1..=count {
            let (db_item, _, _) =
                insert_transaction(db, None, height, namespace).await?;
            transactions.push(db_item);
        }
        Ok(transactions)
    }

    #[tokio::test]
    async fn test_inserting_script_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let tx = MockTransaction::script(vec![], vec![], vec![]);
        insert_transaction(&db, Some(tx), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_create_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let tx = MockTransaction::create(vec![], vec![], vec![]);
        insert_transaction(&db, Some(tx), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_mint_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let tx = MockTransaction::mint(vec![], vec![], vec![]);
        insert_transaction(&db, Some(tx), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_upgrade_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let tx = MockTransaction::upgrade(vec![], vec![], vec![]);
        insert_transaction(&db, Some(tx), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_upload_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let tx = MockTransaction::upload(vec![], vec![], vec![]);
        insert_transaction(&db, Some(tx), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_blob_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let tx = MockTransaction::blob(vec![], vec![], vec![]);
        insert_transaction(&db, Some(tx), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_all_transaction_types() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        for tx in MockTransaction::all() {
            insert_transaction(&db, Some(tx), 1, &namespace).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_find_one_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let (db_item, _, subject) =
            insert_transaction(&db, None, 1, &namespace).await?;

        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));

        let result = Transaction::find_one(db.pool_ref(), &query).await?;
        assert_result(&result, &db_item);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_transactions_basic_query() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let transactions = create_transactions(&db, &namespace, 3).await?;

        let mut query = TransactionsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three transactions");
        assert_result(&results[0], &transactions[0]);
        assert_result(&results[1], &transactions[1]);
        assert_result(&results[2], &transactions[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_transactions_with_cursor_based_pagination_after(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let transactions = create_transactions(&db, &namespace, 5).await?;

        let mut query = TransactionsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_after(Some(transactions[1].cursor()));
        query.with_first(Some(2));

        let results = Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 transactions after cursor"
        );
        assert_result(&results[0], &transactions[2]);
        assert_result(&results[1], &transactions[3]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_transactions_with_cursor_based_pagination_before(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let transactions = create_transactions(&db, &namespace, 5).await?;

        let mut query = TransactionsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_before(Some(transactions[4].cursor()));
        query.with_last(Some(2));

        let results = Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 transactions before cursor"
        );
        assert_result(&results[0], &transactions[3]);
        assert_result(&results[1], &transactions[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_transactions_with_limit_offset_pagination(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let transactions = create_transactions(&db, &namespace, 5).await?;

        let mut query = TransactionsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_limit(Some(2));
        query.with_offset(Some(1));
        query.with_order_by(OrderBy::Asc);

        let results = Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 2, "Should return exactly 2 transactions");
        assert_result(&results[0], &transactions[1]);
        assert_result(&results[1], &transactions[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_transactions_with_different_order() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let transactions = create_transactions(&db, &namespace, 3).await?;

        let mut query = TransactionsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_result(&asc_results[0], &transactions[0]);
        assert_result(&asc_results[2], &transactions[2]);

        query.with_order_by(OrderBy::Desc);
        let desc_results =
            Transaction::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_result(&desc_results[0], &transactions[2]);
        assert_result(&desc_results[2], &transactions[0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_cursor_pagination_ignores_order_by() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let transactions = create_transactions(&db, &namespace, 5).await?;

        let mut query = TransactionsQuery::default();
        query.with_namespace(Some(namespace));
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
