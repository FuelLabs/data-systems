use std::sync::Arc;

use async_trait::async_trait;
use fuel_data_parser::DataEncoder;
use fuel_streams_types::{BlockHeight, BlockTimestamp};
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Block, BlockDbItem, BlocksQuery};
use crate::{
    infra::{
        db::Db,
        repository::{Repository, RepositoryResult},
        QueryOptions,
    },
    transactions::{Transaction, TransactionsQuery},
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

    pub async fn find_in_height_range(
        db: &Db,
        start_height: BlockHeight,
        end_height: BlockHeight,
        options: &QueryOptions,
    ) -> RepositoryResult<Vec<BlockDbItem>> {
        let select = "SELECT * FROM blocks".to_string();
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
        let query = query_builder.build_query_as::<BlockDbItem>();
        let records = query.fetch_all(&db.pool).await?;

        Ok(records)
    }

    pub async fn transactions_from_db(
        &self,
        db: &Arc<Db>,
    ) -> RepositoryResult<Vec<Transaction>> {
        let mut all_transactions =
            Vec::with_capacity(self.transaction_ids.len());
        for chunk in self.transaction_ids.chunks(50) {
            let mut db_tx = db.pool_ref().begin().await?;
            for tx_id in chunk {
                let txs_query = TransactionsQuery {
                    block_height: Some(self.height),
                    tx_id: Some(tx_id.to_owned()),
                    ..Default::default()
                };
                let db_item =
                    Transaction::find_one_with_db_tx(&mut db_tx, &txs_query)
                        .await?;
                let transaction = Transaction::decode_json(&db_item.value)?;
                all_transactions.push(transaction);
            }
            db_tx.commit().await?;
        }
        Ok(all_transactions)
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
            &db,
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
