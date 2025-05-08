use async_trait::async_trait;
use fuel_data_parser::DataEncoder;
use fuel_streams_types::{BlockHeight, BlockTimestamp};
use sqlx::{Acquire, PgExecutor, Postgres, Row};

use super::{Block, BlockDbItem, BlocksQuery};
use crate::{
    infra::{
        db::Db,
        repository::{Repository, RepositoryResult},
        QueryOptions,
    },
    transactions::{Transaction, TransactionDbItem},
};

#[async_trait]
impl Repository for Block {
    type Item = BlockDbItem;
    type QueryParams = BlocksQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let created_at = BlockTimestamp::now();
        let result = sqlx::query_as::<_, BlockDbItem>(
            r#"
            INSERT INTO blocks (
                subject,
                producer_address,
                block_da_height,
                block_height,
                value,
                version,
                header_application_hash,
                header_consensus_parameters_version,
                header_da_height,
                header_event_inbox_root,
                header_message_outbox_root,
                header_message_receipt_count,
                header_prev_root,
                header_state_transition_bytecode_version,
                header_time,
                header_transactions_count,
                header_transactions_root,
                header_version,
                consensus_chain_config_hash,
                consensus_coins_root,
                consensus_type,
                consensus_contracts_root,
                consensus_messages_root,
                consensus_signature,
                consensus_transactions_root,
                block_time,
                created_at,
                block_propagation_ms
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                $21, $22, $23, $24, $25, $26, $27, $28
            )
            ON CONFLICT (block_height)
            DO UPDATE SET
                subject = EXCLUDED.subject,
                producer_address = EXCLUDED.producer_address,
                block_da_height = EXCLUDED.block_da_height,
                value = EXCLUDED.value,
                version = EXCLUDED.version,
                header_application_hash = EXCLUDED.header_application_hash,
                header_consensus_parameters_version = EXCLUDED.header_consensus_parameters_version,
                header_da_height = EXCLUDED.header_da_height,
                header_event_inbox_root = EXCLUDED.header_event_inbox_root,
                header_message_outbox_root = EXCLUDED.header_message_outbox_root,
                header_message_receipt_count = EXCLUDED.header_message_receipt_count,
                header_prev_root = EXCLUDED.header_prev_root,
                header_state_transition_bytecode_version = EXCLUDED.header_state_transition_bytecode_version,
                header_time = EXCLUDED.header_time,
                header_transactions_count = EXCLUDED.header_transactions_count,
                header_transactions_root = EXCLUDED.header_transactions_root,
                header_version = EXCLUDED.header_version,
                consensus_chain_config_hash = EXCLUDED.consensus_chain_config_hash,
                consensus_coins_root = EXCLUDED.consensus_coins_root,
                consensus_type = EXCLUDED.consensus_type,
                consensus_contracts_root = EXCLUDED.consensus_contracts_root,
                consensus_messages_root = EXCLUDED.consensus_messages_root,
                consensus_signature = EXCLUDED.consensus_signature,
                consensus_transactions_root = EXCLUDED.consensus_transactions_root,
                block_time = EXCLUDED.block_time,
                created_at = EXCLUDED.created_at,
                block_propagation_ms = EXCLUDED.block_propagation_ms
            RETURNING *
            "#,
        )
        .bind(&db_item.subject)
        .bind(&db_item.producer_address)
        .bind(db_item.block_da_height)
        .bind(db_item.block_height.into_inner() as i64)
        .bind(&db_item.value)
        .bind(&db_item.version)
        .bind(&db_item.header_application_hash)
        .bind(db_item.header_consensus_parameters_version)
        .bind(db_item.header_da_height)
        .bind(&db_item.header_event_inbox_root)
        .bind(&db_item.header_message_outbox_root)
        .bind(db_item.header_message_receipt_count)
        .bind(&db_item.header_prev_root)
        .bind(db_item.header_state_transition_bytecode_version)
        .bind(db_item.header_time)
        .bind(db_item.header_transactions_count)
        .bind(&db_item.header_transactions_root)
        .bind(&db_item.header_version)
        .bind(db_item.consensus_chain_config_hash.as_ref())
        .bind(db_item.consensus_coins_root.as_ref())
        .bind(db_item.consensus_type)
        .bind(db_item.consensus_contracts_root.as_ref())
        .bind(db_item.consensus_messages_root.as_ref())
        .bind(db_item.consensus_signature.as_ref())
        .bind(db_item.consensus_transactions_root.as_ref())
        .bind(db_item.block_time)
        .bind(created_at)
        .bind(db_item.block_propagation_ms)
        .fetch_one(executor)
        .await?;

        Ok(result)
    }
}

impl Block {
    pub async fn find_last_block_height(
        db: &Db,
        options: &QueryOptions,
    ) -> RepositoryResult<BlockHeight> {
        let select = "SELECT block_height FROM blocks".to_string();
        let mut query_builder = sqlx::QueryBuilder::new(select);
        if let Some(ns) = options.namespace.as_ref() {
            query_builder
                .push(" WHERE subject LIKE ")
                .push_bind(format!("{}%", ns));
        }
        query_builder.push(" ORDER BY block_height DESC LIMIT 1");
        let query = query_builder.build_query_as::<(i64,)>();
        let record: Option<(i64,)> = query.fetch_optional(&db.pool).await?;
        Ok(record.map(|(height,)| height.into()).unwrap_or_default())
    }

    pub async fn find_first_block_height(
        db: &Db,
        options: &QueryOptions,
    ) -> RepositoryResult<BlockHeight> {
        let select = "SELECT block_height FROM blocks".to_string();
        let mut query_builder = sqlx::QueryBuilder::new(select);
        if let Some(ns) = options.namespace.as_ref() {
            query_builder
                .push(" WHERE subject LIKE ")
                .push_bind(format!("{}%", ns));
        }
        query_builder.push(" ORDER BY block_height ASC LIMIT 1");
        let query = query_builder.build_query_as::<(i64,)>();
        let record: Option<(i64,)> = query.fetch_optional(&db.pool).await?;
        Ok(record.map(|(height,)| height.into()).unwrap_or_default())
    }

    pub async fn find_in_height_range<'c, E>(
        executor: E,
        start_height: BlockHeight,
        end_height: BlockHeight,
        options: &QueryOptions,
    ) -> RepositoryResult<Vec<BlockDbItem>>
    where
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let select = "SELECT *".to_string();
        let mut query_builder = sqlx::QueryBuilder::new(select);
        query_builder
            .push(" FROM blocks WHERE block_height >= ")
            .push_bind(start_height.into_inner() as i64)
            .push(" AND block_height <= ")
            .push_bind(end_height.into_inner() as i64);

        if let Some(ns) = options.namespace.as_ref() {
            query_builder
                .push(" AND subject LIKE ")
                .push_bind(format!("{}%", ns));
        }

        query_builder.push(" ORDER BY block_height ASC");
        let query = query_builder.build_query_as::<BlockDbItem>();
        let records = query.fetch_all(executor).await?;

        Ok(records)
    }

    pub async fn transactions_from_block<'c, E>(
        &self,
        executor: E,
    ) -> RepositoryResult<Vec<Transaction>>
    where
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let db_items = sqlx::query_as::<_, TransactionDbItem>(
            r#"
            SELECT *
            FROM transactions
            WHERE block_height = $1
            ORDER BY tx_index ASC
            "#,
        )
        .bind(self.height.into_inner() as i64)
        .fetch_all(executor)
        .await?;

        let mut transactions = Vec::with_capacity(db_items.len());
        for db_item in db_items {
            let transaction = Transaction::decode_json(&db_item.value)?;
            transactions.push(transaction);
        }

        Ok(transactions)
    }

    pub async fn find_blocks_with_transactions<'c, E>(
        executor: E,
        start_height: BlockHeight,
        end_height: BlockHeight,
        options: &QueryOptions,
    ) -> RepositoryResult<Vec<(BlockDbItem, Vec<TransactionDbItem>)>>
    where
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let mut query = String::from(
            r#"
            SELECT
                b.value as b_value, b.subject as b_subject, b.block_height as b_block_height,
                b.block_da_height as b_block_da_height, b.version as b_version, b.producer_address as b_producer_address,
                b.header_application_hash as b_header_application_hash,
                b.header_consensus_parameters_version as b_header_consensus_parameters_version,
                b.header_da_height as b_header_da_height, b.header_event_inbox_root as b_header_event_inbox_root,
                b.header_message_outbox_root as b_header_message_outbox_root,
                b.header_message_receipt_count as b_header_message_receipt_count,
                b.header_prev_root as b_header_prev_root,
                b.header_state_transition_bytecode_version as b_header_state_transition_bytecode_version,
                b.header_time as b_header_time, b.header_transactions_count as b_header_transactions_count,
                b.header_transactions_root as b_header_transactions_root, b.header_version as b_header_version,
                b.consensus_chain_config_hash as b_consensus_chain_config_hash,
                b.consensus_coins_root as b_consensus_coins_root, b.consensus_type as b_consensus_type,
                b.consensus_contracts_root as b_consensus_contracts_root,
                b.consensus_messages_root as b_consensus_messages_root,
                b.consensus_signature as b_consensus_signature,
                b.consensus_transactions_root as b_consensus_transactions_root,
                b.block_time as b_block_time, b.created_at as b_created_at,
                b.block_propagation_ms as b_block_propagation_ms,
                t.value as t_value, t.subject as t_subject, t.block_height as t_block_height,
                t.tx_id as t_tx_id, t.tx_index as t_tx_index, t.type as t_type,
                t.status as t_status, t.created_at as t_created_at,
                t.script_gas_limit as script_gas_limit, t.mint_amount as mint_amount,
                t.mint_asset_id as mint_asset_id, t.mint_gas_price as mint_gas_price,
                t.receipts_root as receipts_root, t.script as script, t.script_data as script_data,
                t.salt as salt, t.bytecode_witness_index as bytecode_witness_index,
                t.bytecode_root as bytecode_root, t.subsection_index as subsection_index,
                t.subsections_number as subsections_number, t.upgrade_purpose as upgrade_purpose,
                t.blob_id as blob_id, t.is_blob as is_blob, t.is_create as is_create,
                t.is_mint as is_mint, t.is_script as is_script, t.is_upgrade as is_upgrade,
                t.is_upload as is_upload, t.raw_payload as raw_payload, t.tx_pointer as tx_pointer,
                t.maturity as maturity, t.script_length as script_length, t.script_data_length as script_data_length,
                t.storage_slots_count as storage_slots_count, t.proof_set_count as proof_set_count,
                t.witnesses_count as witnesses_count, t.inputs_count as inputs_count,
                t.outputs_count as outputs_count, t.block_time as t_block_time
            FROM blocks b
            LEFT JOIN transactions t ON b.block_height = t.block_height
            WHERE b.block_height >= $1 AND b.block_height <= $2
            "#,
        );

        if let Some(ns) = options.namespace.as_ref() {
            query.push_str(&format!(" AND b.subject LIKE '{}%'", ns));
        }

        query.push_str(" ORDER BY b.block_height ASC, t.tx_index ASC");

        let rows = sqlx::query(&query)
            .bind(start_height.into_inner() as i64)
            .bind(end_height.into_inner() as i64)
            .fetch_all(executor)
            .await?;

        // Process rows into blocks with transactions
        let mut result = Vec::new();
        let mut current_block: Option<BlockDbItem> = None;
        let mut current_txs: Vec<TransactionDbItem> = Vec::new();

        for row in rows {
            let block_height: i64 = row.get("b_block_height");
            if current_block.as_ref().is_none_or(|b| {
                b.block_height.into_inner() as i64 != block_height
            }) {
                if let Some(block) = current_block {
                    result.push((block, current_txs));
                }
                current_block = Some(BlockDbItem {
                    value: row.get("b_value"),
                    subject: row.get("b_subject"),
                    block_height: (row.get::<i64, _>("b_block_height") as u64)
                        .into(),
                    block_da_height: row.get("b_block_da_height"),
                    version: row.get("b_version"),
                    producer_address: row.get("b_producer_address"),
                    header_application_hash: row
                        .get("b_header_application_hash"),
                    header_consensus_parameters_version: row
                        .get("b_header_consensus_parameters_version"),
                    header_da_height: row.get("b_header_da_height"),
                    header_event_inbox_root: row
                        .get("b_header_event_inbox_root"),
                    header_message_outbox_root: row
                        .get("b_header_message_outbox_root"),
                    header_message_receipt_count: row
                        .get("b_header_message_receipt_count"),
                    header_prev_root: row.get("b_header_prev_root"),
                    header_state_transition_bytecode_version: row
                        .get("b_header_state_transition_bytecode_version"),
                    header_time: row.get("b_header_time"),
                    header_transactions_count: row
                        .get("b_header_transactions_count"),
                    header_transactions_root: row
                        .get("b_header_transactions_root"),
                    header_version: row.get("b_header_version"),
                    consensus_chain_config_hash: row
                        .get("b_consensus_chain_config_hash"),
                    consensus_coins_root: row.get("b_consensus_coins_root"),
                    consensus_type: row.get("b_consensus_type"),
                    consensus_contracts_root: row
                        .get("b_consensus_contracts_root"),
                    consensus_messages_root: row
                        .get("b_consensus_messages_root"),
                    consensus_signature: row.get("b_consensus_signature"),
                    consensus_transactions_root: row
                        .get("b_consensus_transactions_root"),
                    block_time: row.get("b_block_time"),
                    created_at: row.get("b_created_at"),
                    block_propagation_ms: row.get("b_block_propagation_ms"),
                });
                current_txs = Vec::new();
            }

            let tx_value: Option<Vec<u8>> = row.get("t_value");
            if tx_value.is_some() {
                current_txs.push(TransactionDbItem {
                    value: row.get("t_value"),
                    subject: row.get("t_subject"),
                    block_height: (row.get::<i64, _>("t_block_height") as u64)
                        .into(),
                    tx_id: row.get("t_tx_id"),
                    tx_index: row.get("t_tx_index"),
                    r#type: row.get("t_type"),
                    status: row.get("t_status"),
                    created_at: row.get("t_created_at"),
                    script_gas_limit: row.get("script_gas_limit"),
                    mint_amount: row.get("mint_amount"),
                    mint_asset_id: row.get("mint_asset_id"),
                    mint_gas_price: row.get("mint_gas_price"),
                    receipts_root: row.get("receipts_root"),
                    script: row.get("script"),
                    script_data: row.get("script_data"),
                    salt: row.get("salt"),
                    bytecode_witness_index: row.get("bytecode_witness_index"),
                    bytecode_root: row.get("bytecode_root"),
                    subsection_index: row.get("subsection_index"),
                    subsections_number: row.get("subsections_number"),
                    upgrade_purpose: row.get("upgrade_purpose"),
                    blob_id: row.get("blob_id"),
                    is_blob: row.get("is_blob"),
                    is_create: row.get("is_create"),
                    is_mint: row.get("is_mint"),
                    is_script: row.get("is_script"),
                    is_upgrade: row.get("is_upgrade"),
                    is_upload: row.get("is_upload"),
                    raw_payload: row.get("raw_payload"),
                    tx_pointer: row.get("tx_pointer"),
                    maturity: row.get("maturity"),
                    script_length: row.get("script_length"),
                    script_data_length: row.get("script_data_length"),
                    storage_slots_count: row.get("storage_slots_count"),
                    proof_set_count: row.get("proof_set_count"),
                    witnesses_count: row.get("witnesses_count"),
                    inputs_count: row.get("inputs_count"),
                    outputs_count: row.get("outputs_count"),
                    block_time: row.get("t_block_time"),
                });
            }
        }

        if let Some(block) = current_block {
            result.push((block, current_txs));
        }

        Ok(result)
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use anyhow::Result;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        blocks::packets::DynBlockSubject,
        infra::{
            Db,
            DbConnectionOpts,
            DbItem,
            OrderBy,
            QueryOptions,
            QueryParamsBuilder,
        },
        mocks::MockBlock,
    };

    async fn setup_db() -> anyhow::Result<(Arc<Db>, String)> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let namespace = QueryOptions::random_namespace();
        Ok((db, namespace))
    }

    fn assert_result(result: &BlockDbItem, expected: &BlockDbItem) {
        assert_eq!(result.subject, expected.subject);
        assert_eq!(result.value, expected.value);
        assert_eq!(result.block_height, expected.block_height);
        assert_eq!(result.block_da_height, expected.block_da_height);
        assert_eq!(result.producer_address, expected.producer_address);
        assert_eq!(result.version, expected.version);
        assert_eq!(result.block_propagation_ms, expected.block_propagation_ms);
        assert_eq!(
            result.header_application_hash,
            expected.header_application_hash
        );
        assert_eq!(
            result.header_consensus_parameters_version,
            expected.header_consensus_parameters_version
        );
        assert_eq!(result.header_da_height, expected.header_da_height);
        assert_eq!(
            result.header_event_inbox_root,
            expected.header_event_inbox_root
        );
        assert_eq!(
            result.header_message_outbox_root,
            expected.header_message_outbox_root
        );
        assert_eq!(
            result.header_message_receipt_count,
            expected.header_message_receipt_count
        );
        assert_eq!(result.header_prev_root, expected.header_prev_root);
        assert_eq!(
            result.header_state_transition_bytecode_version,
            expected.header_state_transition_bytecode_version
        );
        assert_eq!(
            result.header_transactions_count,
            expected.header_transactions_count
        );
        assert_eq!(
            result.header_transactions_root,
            expected.header_transactions_root
        );
        assert_eq!(result.header_version, expected.header_version);
        assert_eq!(
            result.consensus_chain_config_hash,
            expected.consensus_chain_config_hash
        );
        assert_eq!(result.consensus_coins_root, expected.consensus_coins_root);
        assert_eq!(result.consensus_type, expected.consensus_type);
        assert_eq!(
            result.consensus_contracts_root,
            expected.consensus_contracts_root
        );
        assert_eq!(
            result.consensus_messages_root,
            expected.consensus_messages_root
        );
        assert_eq!(result.consensus_signature, expected.consensus_signature);
        assert_eq!(
            result.consensus_transactions_root,
            expected.consensus_transactions_root
        );
    }

    pub async fn insert_block(
        db: &Arc<Db>,
        height: BlockHeight,
        namespace: &str,
    ) -> Result<(BlockDbItem, Block, DynBlockSubject)> {
        let block = MockBlock::build(height);
        let subject = DynBlockSubject::new(
            block.height,
            block.producer.clone(),
            &block.header.da_height,
        );
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(&block, timestamps)
            .with_namespace(namespace);

        let db_item = BlockDbItem::try_from(&packet)?;
        let result = Block::insert(db.pool_ref(), &db_item).await?;
        assert_result(&result, &db_item);

        Ok((db_item, block, subject))
    }

    async fn create_blocks(
        db: &Arc<Db>,
        namespace: &str,
        count: u32,
    ) -> Result<Vec<BlockDbItem>> {
        let mut blocks = Vec::with_capacity(count as usize);
        for _ in 1..=count {
            let random_height = BlockHeight::random();
            let (db_item, _, _) =
                insert_block(db, random_height, namespace).await?;
            blocks.push(db_item);
        }
        blocks.sort_by_key(|b| b.block_height);
        Ok(blocks)
    }

    #[tokio::test]
    async fn test_insert_block() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let random_height = BlockHeight::random();
        insert_block(&db, random_height, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_insert_block_with_transaction() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let (db_item1, _, _) =
            insert_block(&db, BlockHeight::random(), &namespace).await?;
        let (db_item2, _, _) =
            insert_block(&db, BlockHeight::random(), &namespace).await?;

        let mut tx = db.pool_ref().begin().await?;
        let result1 =
            Block::insert_with_transaction(&mut tx, &db_item1).await?;
        let result2 =
            Block::insert_with_transaction(&mut tx, &db_item2).await?;
        tx.commit().await?;

        assert_result(&result1, &db_item1);
        assert_result(&result2, &db_item2);
        Ok(())
    }

    #[tokio::test]
    async fn test_find_one_block() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let (db_item, _, subject) =
            insert_block(&db, BlockHeight::random(), &namespace).await?;

        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));

        let result = Block::find_one(db.pool_ref(), &query).await?;
        assert_result(&result, &db_item);
        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_blocks_basic_query() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let blocks = create_blocks(&db, &namespace, 3).await?;

        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three blocks");
        assert_result(&results[0], &blocks[0]);
        assert_result(&results[1], &blocks[1]);
        assert_result(&results[2], &blocks[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_blocks_with_height_filter() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let blocks = create_blocks(&db, &namespace, 3).await?;

        let second_height = blocks[1].block_height;
        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace));
        query.with_from_block(Some(second_height));
        query.with_order_by(OrderBy::Asc);

        let results = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 2, "Should find blocks with height >= 2");
        assert_eq!(results[0].block_height, blocks[1].block_height);
        assert_eq!(results[1].block_height, blocks[2].block_height);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_blocks_with_cursor_based_pagination_after(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let blocks = create_blocks(&db, &namespace, 5).await?;

        let second_height = blocks[1].cursor();
        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace));
        query.with_after(Some(second_height));
        query.with_first(Some(2));

        let results = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 blocks after cursor"
        );
        assert_eq!(results[0].block_height, blocks[2].block_height);
        assert_eq!(results[1].block_height, blocks[3].block_height);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_blocks_with_cursor_based_pagination_before(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let blocks = create_blocks(&db, &namespace, 5).await?;

        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace));
        query.with_before(Some(blocks[4].cursor()));
        query.with_last(Some(2));

        let results = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 blocks before cursor"
        );
        assert_eq!(results[0].block_height, blocks[3].block_height);
        assert_eq!(results[1].block_height, blocks[2].block_height);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_blocks_with_limit_offset_pagination() -> Result<()>
    {
        let (db, namespace) = setup_db().await?;
        let blocks = create_blocks(&db, &namespace, 5).await?;

        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace));
        query.with_limit(Some(2));
        query.with_offset(Some(1));
        query.with_order_by(OrderBy::Asc);

        let results = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 2, "Should return exactly 2 blocks");
        assert_result(&results[0], &blocks[1]);
        assert_result(&results[1], &blocks[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_blocks_with_different_order() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let mut blocks = create_blocks(&db, &namespace, 3).await?;
        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_result(&asc_results[0], &blocks[0]);
        assert_result(&asc_results[2], &blocks[2]);

        query.with_order_by(OrderBy::Desc);
        blocks.sort_by(|b1, b2| b1.block_height.cmp(&b2.block_height));
        let desc_results = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_result(&desc_results[0], &blocks[2]);
        assert_result(&desc_results[2], &blocks[0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_cursor_pagination_ignores_order_by() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let blocks = create_blocks(&db, &namespace, 5).await?;

        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace));
        query.with_after(Some(blocks[1].cursor()));
        query.with_first(Some(2));

        let results_default = Block::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Asc);
        let results_asc = Block::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Desc);
        let results_desc = Block::find_many(db.pool_ref(), &query).await?;

        assert_eq!(results_default, results_asc);
        assert_eq!(results_default, results_desc);
        Ok(())
    }

    #[tokio::test]
    async fn test_find_last_block_height() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let blocks = create_blocks(&db, &namespace, 3).await?;

        let mut options = QueryOptions::default();
        options.with_namespace(Some(namespace));
        let last_height = Block::find_last_block_height(&db, &options).await?;

        assert_eq!(last_height, blocks.last().unwrap().block_height);
        Ok(())
    }

    #[tokio::test]
    async fn test_find_blocks_in_height_range() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let blocks = create_blocks(&db, &namespace, 5).await?;
        let start_height = blocks[1].block_height;
        let end_height = blocks[3].block_height;
        let mut options = QueryOptions::default();
        options.with_namespace(Some(namespace));

        let results = Block::find_in_height_range(
            db.pool_ref(),
            start_height,
            end_height,
            &options,
        )
        .await?;

        assert_eq!(results.len(), 3, "Should return exactly 3 blocks in range");
        assert_result(&results[0], &blocks[1]);
        assert_result(&results[1], &blocks[2]);
        assert_result(&results[2], &blocks[3]);
        Ok(())
    }
}
