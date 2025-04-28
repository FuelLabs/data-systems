use async_trait::async_trait;
use fuel_data_parser::DataEncoder;
use fuel_streams_types::{BlockHeight, BlockTimestamp};
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{
    db_relations::*,
    Transaction,
    TransactionDbItem,
    TransactionsQuery,
};
use crate::infra::{
    repository::{Repository, RepositoryError, RepositoryResult},
    Db,
    DbItem,
    QueryOptions,
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
            r#"
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
                status,
                script,
                script_data,
                salt,
                bytecode_witness_index,
                bytecode_root,
                subsection_index,
                subsections_number,
                upgrade_purpose,
                blob_id,
                is_blob,
                is_create,
                is_mint,
                is_script,
                is_upgrade,
                is_upload,
                raw_payload,
                maturity,
                tx_pointer,
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
                $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30,
                $31, $32, $33, $34, $35, $36, $37, $38, $39, $40
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
                status = EXCLUDED.status,
                script = EXCLUDED.script,
                script_data = EXCLUDED.script_data,
                salt = EXCLUDED.salt,
                bytecode_witness_index = EXCLUDED.bytecode_witness_index,
                bytecode_root = EXCLUDED.bytecode_root,
                subsection_index = EXCLUDED.subsection_index,
                subsections_number = EXCLUDED.subsections_number,
                upgrade_purpose = EXCLUDED.upgrade_purpose,
                blob_id = EXCLUDED.blob_id,
                is_blob = EXCLUDED.is_blob,
                is_create = EXCLUDED.is_create,
                is_mint = EXCLUDED.is_mint,
                is_script = EXCLUDED.is_script,
                is_upgrade = EXCLUDED.is_upgrade,
                is_upload = EXCLUDED.is_upload,
                raw_payload = EXCLUDED.raw_payload,
                maturity = EXCLUDED.maturity,
                tx_pointer = EXCLUDED.tx_pointer,
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
            "#,
        )
        .bind(&db_item.subject)
        .bind(&db_item.value)
        .bind(db_item.block_height.into_inner() as i64)
        .bind(&db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.cursor().to_string())
        .bind(db_item.r#type)
        .bind(db_item.script_gas_limit)
        .bind(db_item.mint_amount)
        .bind(&db_item.mint_asset_id)
        .bind(db_item.mint_gas_price)
        .bind(&db_item.receipts_root)
        .bind(db_item.status)
        .bind(&db_item.script)
        .bind(&db_item.script_data)
        .bind(&db_item.salt)
        .bind(db_item.bytecode_witness_index)
        .bind(&db_item.bytecode_root)
        .bind(db_item.subsection_index)
        .bind(db_item.subsections_number)
        .bind(&db_item.upgrade_purpose)
        .bind(&db_item.blob_id)
        .bind(db_item.is_blob)
        .bind(db_item.is_create)
        .bind(db_item.is_mint)
        .bind(db_item.is_script)
        .bind(db_item.is_upgrade)
        .bind(db_item.is_upload)
        .bind(&db_item.raw_payload)
        .bind(db_item.maturity)
        .bind(&db_item.tx_pointer)
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
        if let Some(storage_slots) = &tx.storage_slots {
            for slot in storage_slots {
                let slot_item = TransactionStorageSlotDbItem {
                    tx_id: db_item.tx_id.clone(),
                    block_height: db_item.block_height,
                    key: slot.key.to_string(),
                    value: slot.value.to_string(),
                    block_time: db_item.block_time,
                    created_at,
                };
                sqlx::query(
                    "INSERT INTO transaction_storage_slots (
                        tx_id, block_height, key, value, block_time, created_at
                    ) VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(&slot_item.tx_id)
                .bind(slot_item.block_height.into_inner() as i64)
                .bind(&slot_item.key)
                .bind(&slot_item.value)
                .bind(slot_item.block_time)
                .bind(slot_item.created_at)
                .execute(&mut *db_tx)
                .await
                .map_err(RepositoryError::Insert)?;
            }
        }

        if let Some(witnesses) = &tx.witnesses {
            for witness in witnesses {
                let witness_item = TransactionWitnessDbItem {
                    tx_id: db_item.tx_id.clone(),
                    block_height: db_item.block_height,
                    witness_data: witness.to_string(),
                    witness_data_length: witness.as_ref().0.len() as i32,
                    block_time: db_item.block_time,
                    created_at,
                };
                sqlx::query(
                    "INSERT INTO transaction_witnesses (
                        tx_id, block_height, witness_data, witness_data_length,
                        block_time, created_at
                    ) VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(&witness_item.tx_id)
                .bind(witness_item.block_height.into_inner() as i64)
                .bind(&witness_item.witness_data)
                .bind(witness_item.witness_data_length)
                .bind(witness_item.block_time)
                .bind(witness_item.created_at)
                .execute(&mut *db_tx)
                .await
                .map_err(RepositoryError::Insert)?;
            }
        }

        if let Some(proof_set) = &tx.proof_set {
            for proof in proof_set {
                let proof_item = TransactionProofSetDbItem {
                    tx_id: db_item.tx_id.clone(),
                    block_height: db_item.block_height,
                    proof_hash: proof.to_string(),
                    block_time: db_item.block_time,
                    created_at,
                };
                sqlx::query(
                    "INSERT INTO transaction_proof_set (
                        tx_id, block_height, proof_hash, block_time, created_at
                    ) VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(&proof_item.tx_id)
                .bind(proof_item.block_height.into_inner() as i64)
                .bind(&proof_item.proof_hash)
                .bind(proof_item.block_time)
                .bind(proof_item.created_at)
                .execute(&mut *db_tx)
                .await
                .map_err(RepositoryError::Insert)?;
            }
        }

        if let Some(policies) = &tx.policies {
            let policy_item = TransactionPolicyDbItem {
                tx_id: db_item.tx_id.clone(),
                block_height: db_item.block_height,
                tip: policies.tip.map(|t| t.into()),
                maturity: policies.maturity.map(|m| m.into_inner() as i32),
                witness_limit: policies.witness_limit.map(|w| w.into()),
                max_fee: policies.max_fee.map(|f| f.into()),
                block_time: db_item.block_time,
                created_at,
            };
            sqlx::query(
                "INSERT INTO transaction_policies (
                    tx_id, block_height, tip, maturity, witness_limit, max_fee,
                    block_time, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(&policy_item.tx_id)
            .bind(policy_item.block_height.into_inner() as i64)
            .bind(policy_item.tip)
            .bind(policy_item.maturity)
            .bind(policy_item.witness_limit)
            .bind(policy_item.max_fee)
            .bind(policy_item.block_time)
            .bind(policy_item.created_at)
            .execute(&mut *db_tx)
            .await
            .map_err(RepositoryError::Insert)?;
        }

        if let Some(input_contracts) = &tx.input_contracts {
            for contract_id in input_contracts {
                let contract_item = TransactionInputContractDbItem {
                    tx_id: db_item.tx_id.clone(),
                    block_height: db_item.block_height,
                    contract_id: contract_id.to_string(),
                    block_time: db_item.block_time,
                    created_at,
                };
                sqlx::query(
                    "INSERT INTO transaction_input_contracts (
                        tx_id, block_height, contract_id, block_time, created_at
                    ) VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(&contract_item.tx_id)
                .bind(contract_item.block_height.into_inner() as i64)
                .bind(&contract_item.contract_id)
                .bind(contract_item.block_time)
                .bind(contract_item.created_at)
                .execute(&mut *db_tx)
                .await
                .map_err(RepositoryError::Insert)?;
            }
        }

        if let Some(input_contract) = &tx.input_contract {
            let tx_pointer = serde_json::to_vec(&input_contract.tx_pointer)?;
            let contract_item = TransactionInputContractSingleDbItem {
                tx_id: db_item.tx_id.clone(),
                block_height: db_item.block_height,
                balance_root: input_contract.balance_root.to_string(),
                contract_id: input_contract.contract_id.to_string(),
                state_root: input_contract.state_root.to_string(),
                tx_pointer,
                utxo_id: input_contract.utxo_id.to_string(),
                block_time: db_item.block_time,
                created_at,
            };
            sqlx::query(
                "INSERT INTO transaction_input_contract (
                    tx_id, block_height, balance_root, contract_id, state_root,
                    tx_pointer, utxo_id, block_time, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(&contract_item.tx_id)
            .bind(contract_item.block_height.into_inner() as i64)
            .bind(&contract_item.balance_root)
            .bind(&contract_item.contract_id)
            .bind(&contract_item.state_root)
            .bind(&contract_item.tx_pointer)
            .bind(&contract_item.utxo_id)
            .bind(contract_item.block_time)
            .bind(contract_item.created_at)
            .execute(&mut *db_tx)
            .await
            .map_err(RepositoryError::Insert)?;
        }

        if let Some(output_contract) = &tx.output_contract {
            let contract_item = TransactionOutputContractDbItem {
                tx_id: db_item.tx_id.clone(),
                block_height: db_item.block_height,
                balance_root: output_contract.balance_root.to_string(),
                input_index: output_contract.input_index as i32,
                state_root: output_contract.state_root.to_string(),
                block_time: db_item.block_time,
                created_at,
            };
            sqlx::query(
                "INSERT INTO transaction_output_contract (
                    tx_id, block_height, balance_root, input_index, state_root,
                    block_time, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(&contract_item.tx_id)
            .bind(contract_item.block_height.into_inner() as i64)
            .bind(&contract_item.balance_root)
            .bind(contract_item.input_index)
            .bind(&contract_item.state_root)
            .bind(contract_item.block_time)
            .bind(contract_item.created_at)
            .execute(&mut *db_tx)
            .await
            .map_err(RepositoryError::Insert)?;
        }

        db_tx.commit().await?;
        Ok(record)
    }
}

impl Transaction {
    pub async fn find_in_height_range(
        db: &Db,
        start_height: BlockHeight,
        end_height: BlockHeight,
        options: &QueryOptions,
    ) -> RepositoryResult<Vec<TransactionDbItem>> {
        let select = "SELECT * FROM transactions".to_string();
        let mut query_builder = sqlx::QueryBuilder::new(select);
        query_builder
            .push(" WHERE block_height >= ")
            .push_bind(start_height.into_inner() as i64)
            .push(" AND block_height <= ")
            .push_bind(end_height.into_inner() as i64);

        if let Some(ns) = options.namespace.as_ref() {
            query_builder
                .push(" AND subject LIKE ")
                .push_bind(format!("{}%", ns));
        }

        query_builder.push(" ORDER BY block_height ASC");
        let query = query_builder.build_query_as::<TransactionDbItem>();
        let records = query.fetch_all(&db.pool).await?;

        Ok(records)
    }
}

#[cfg(test)]
pub mod tests {
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
            DbItem,
            OrderBy,
            QueryOptions,
            QueryParamsBuilder,
            RecordPointer,
        },
        mocks::{MockInput, MockOutput, MockTransaction},
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
        assert_eq!(result.status, expected.status);
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
        assert_eq!(result.is_blob, expected.is_blob);
        assert_eq!(result.is_create, expected.is_create);
        assert_eq!(result.is_mint, expected.is_mint);
        assert_eq!(result.is_script, expected.is_script);
        assert_eq!(result.is_upgrade, expected.is_upgrade);
        assert_eq!(result.is_upload, expected.is_upload);
        assert_eq!(result.raw_payload, expected.raw_payload);
        assert_eq!(result.maturity, expected.maturity);
        assert_eq!(result.script_length, expected.script_length);
        assert_eq!(result.script_data_length, expected.script_data_length);
        assert_eq!(result.storage_slots_count, expected.storage_slots_count);
        assert_eq!(result.proof_set_count, expected.proof_set_count);
        assert_eq!(result.witnesses_count, expected.witnesses_count);
        assert_eq!(result.inputs_count, expected.inputs_count);
        assert_eq!(result.outputs_count, expected.outputs_count);
    }

    async fn verify_related_tables(
        db: &Arc<Db>,
        tx: &Transaction,
        db_item: &TransactionDbItem,
    ) -> Result<()> {
        // Verify storage slots
        if let Some(storage_slots) = &tx.storage_slots {
            let slots: Vec<TransactionStorageSlotDbItem> = sqlx::query_as(
                "SELECT * FROM transaction_storage_slots WHERE tx_id = $1",
            )
            .bind(&db_item.tx_id)
            .fetch_all(db.pool_ref())
            .await?;
            assert_eq!(slots.len(), storage_slots.len());
        }

        // Verify witnesses
        if let Some(witnesses) = &tx.witnesses {
            let db_witnesses: Vec<TransactionWitnessDbItem> = sqlx::query_as(
                "SELECT * FROM transaction_witnesses WHERE tx_id = $1",
            )
            .bind(&db_item.tx_id)
            .fetch_all(db.pool_ref())
            .await?;
            assert_eq!(db_witnesses.len(), witnesses.len());
        }

        // Verify proof set
        if let Some(proof_set) = &tx.proof_set {
            let db_proofs: Vec<TransactionProofSetDbItem> = sqlx::query_as(
                "SELECT * FROM transaction_proof_set WHERE tx_id = $1",
            )
            .bind(&db_item.tx_id)
            .fetch_all(db.pool_ref())
            .await?;
            assert_eq!(db_proofs.len(), proof_set.len());
        }

        // Verify policies
        if tx.policies.is_some() {
            let policies: Vec<TransactionPolicyDbItem> = sqlx::query_as(
                "SELECT * FROM transaction_policies WHERE tx_id = $1",
            )
            .bind(&db_item.tx_id)
            .fetch_all(db.pool_ref())
            .await?;
            assert_eq!(policies.len(), 1);
        }

        // Verify input contracts
        if let Some(input_contracts) = &tx.input_contracts {
            let db_contracts: Vec<TransactionInputContractDbItem> = sqlx::query_as(
                "SELECT * FROM transaction_input_contracts WHERE tx_id = $1"
            )
            .bind(&db_item.tx_id)
            .fetch_all(db.pool_ref())
            .await?;
            assert_eq!(db_contracts.len(), input_contracts.len());
        }

        // Verify single input contract
        if tx.input_contract.is_some() {
            let contracts: Vec<TransactionInputContractSingleDbItem> =
                sqlx::query_as(
                    "SELECT * FROM transaction_input_contract WHERE tx_id = $1",
                )
                .bind(&db_item.tx_id)
                .fetch_all(db.pool_ref())
                .await?;
            assert_eq!(contracts.len(), 1);
        }

        // Verify output contract
        if tx.output_contract.is_some() {
            let contracts: Vec<TransactionOutputContractDbItem> = sqlx::query_as(
                "SELECT * FROM transaction_output_contract WHERE tx_id = $1"
            )
            .bind(&db_item.tx_id)
            .fetch_all(db.pool_ref())
            .await?;
            assert_eq!(contracts.len(), 1);
        }

        Ok(())
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

    pub async fn insert_transaction(
        db: &Arc<Db>,
        tx: Option<Transaction>,
        height: BlockHeight,
        namespace: &str,
    ) -> Result<(TransactionDbItem, Transaction, DynTransactionSubject)> {
        let _ = insert_random_block(db, height, namespace).await?;
        let tx = tx
            .unwrap_or_else(|| MockTransaction::script(vec![], vec![], vec![]));
        let subject = DynTransactionSubject::new(&tx, height, 0);
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(&tx, timestamps, RecordPointer {
                block_height: height,
                tx_id: Some(tx.id.to_owned()),
                tx_index: Some(0_u32),
                ..Default::default()
            })
            .with_namespace(namespace);

        let db_item = TransactionDbItem::try_from(&packet)?;
        let result = Transaction::insert(db.pool_ref(), &db_item).await?;
        assert_result(&result, &db_item);
        verify_related_tables(db, &tx, &db_item).await?;

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
                insert_transaction(db, None, height.into(), namespace).await?;
            transactions.push(db_item);
        }
        Ok(transactions)
    }

    #[tokio::test]
    async fn test_inserting_script_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let inputs = MockInput::all();
        let outputs = MockOutput::all();
        let tx = MockTransaction::script(inputs, outputs, vec![]);
        let (db_item, tx, _) =
            insert_transaction(&db, Some(tx), 1.into(), &namespace).await?;
        assert_result(&db_item, &db_item); // Compare with itself to verify insertion
        verify_related_tables(&db, &tx, &db_item).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_create_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let inputs = MockInput::all();
        let outputs = MockOutput::all();
        let tx = MockTransaction::create(inputs, outputs, vec![]);
        let (db_item, tx, _) =
            insert_transaction(&db, Some(tx), 1.into(), &namespace).await?;
        assert_result(&db_item, &db_item);
        verify_related_tables(&db, &tx, &db_item).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_mint_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let inputs = MockInput::all();
        let outputs = MockOutput::all();
        let tx = MockTransaction::mint(inputs, outputs, vec![]);
        let (db_item, tx, _) =
            insert_transaction(&db, Some(tx), 1.into(), &namespace).await?;
        assert_result(&db_item, &db_item);
        verify_related_tables(&db, &tx, &db_item).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_upgrade_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let inputs = MockInput::all();
        let outputs = MockOutput::all();
        let tx = MockTransaction::upgrade(inputs, outputs, vec![]);
        let (db_item, tx, _) =
            insert_transaction(&db, Some(tx), 1.into(), &namespace).await?;
        assert_result(&db_item, &db_item);
        verify_related_tables(&db, &tx, &db_item).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_upload_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let inputs = MockInput::all();
        let outputs = MockOutput::all();
        let tx = MockTransaction::upload(inputs, outputs, vec![]);
        let (db_item, tx, _) =
            insert_transaction(&db, Some(tx), 1.into(), &namespace).await?;
        assert_result(&db_item, &db_item);
        verify_related_tables(&db, &tx, &db_item).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_blob_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let inputs = MockInput::all();
        let outputs = MockOutput::all();
        let tx = MockTransaction::blob(inputs, outputs, vec![]);
        let (db_item, tx, _) =
            insert_transaction(&db, Some(tx), 1.into(), &namespace).await?;
        assert_result(&db_item, &db_item);
        verify_related_tables(&db, &tx, &db_item).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_all_transaction_types() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        for tx in MockTransaction::all() {
            let (db_item, tx, _) =
                insert_transaction(&db, Some(tx), 1.into(), &namespace).await?;
            assert_result(&db_item, &db_item);
            verify_related_tables(&db, &tx, &db_item).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_find_one_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let (db_item, tx, subject) =
            insert_transaction(&db, None, 1.into(), &namespace).await?;

        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));

        let result = Transaction::find_one(db.pool_ref(), &query).await?;
        assert_result(&result, &db_item);
        verify_related_tables(&db, &tx, &result).await?;

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

    #[tokio::test]
    async fn test_find_transactions_in_height_range() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let transactions = create_transactions(&db, &namespace, 10).await?;
        let start_height = transactions[1].block_height;
        let end_height = transactions[5].block_height;
        let mut options = QueryOptions::default();
        options.with_namespace(Some(namespace));

        let results = Transaction::find_in_height_range(
            &db,
            start_height,
            end_height,
            &options,
        )
        .await?;

        assert_eq!(
            results.len(),
            5,
            "Should return exactly 5 transactions in range"
        );
        assert_result(&results[0], &transactions[1]);
        assert_result(&results[1], &transactions[2]);
        assert_result(&results[2], &transactions[3]);
        assert_result(&results[3], &transactions[4]);
        assert_result(&results[4], &transactions[5]);
        Ok(())
    }
}
