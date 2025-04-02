use async_trait::async_trait;
use fuel_streams_types::{BlockHeight, BlockTimestamp};
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Block, BlockDbItem, BlocksQuery};
use crate::infra::{
    db::Db,
    repository::{Repository, RepositoryError, RepositoryResult},
    QueryOptions,
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
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, BlockDbItem>(
            "WITH upsert AS (
                INSERT INTO blocks (
                    subject,
                    producer_address,
                    block_da_height,
                    block_height,
                    value,
                    version,
                    created_at,
                    published_at,
                    block_propagation_ms,
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
                    consensus_transactions_root
                )
                VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    $21, $22, $23, $24, $25, $26, $27, $28
                )
                ON CONFLICT (subject) DO UPDATE SET
                    producer_address = EXCLUDED.producer_address,
                    block_da_height = EXCLUDED.block_da_height,
                    block_height = EXCLUDED.block_height,
                    value = EXCLUDED.value,
                    version = EXCLUDED.version,
                    created_at = EXCLUDED.created_at,
                    published_at = $8,
                    block_propagation_ms = EXCLUDED.block_propagation_ms,
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
                    consensus_transactions_root = EXCLUDED.consensus_transactions_root
                RETURNING *
            )
            SELECT * FROM upsert"
        )
        .bind(db_item.subject.to_owned())
        .bind(db_item.producer_address.to_owned())
        .bind(db_item.block_da_height)
        .bind(db_item.block_height)
        .bind(db_item.value.to_owned())
        .bind(db_item.version.to_owned())
        .bind(db_item.created_at)
        .bind(published_at)
        .bind(db_item.block_propagation_ms)
        .bind(db_item.header_application_hash.to_owned())
        .bind(db_item.header_consensus_parameters_version)
        .bind(db_item.header_da_height)
        .bind(db_item.header_event_inbox_root.to_owned())
        .bind(db_item.header_message_outbox_root.to_owned())
        .bind(db_item.header_message_receipt_count)
        .bind(db_item.header_prev_root.to_owned())
        .bind(db_item.header_state_transition_bytecode_version)
        .bind(db_item.header_time)
        .bind(db_item.header_transactions_count)
        .bind(db_item.header_transactions_root.to_owned())
        .bind(db_item.header_version.to_owned())
        .bind(db_item.consensus_chain_config_hash.as_ref())
        .bind(db_item.consensus_coins_root.as_ref())
        .bind(db_item.consensus_type)
        .bind(db_item.consensus_contracts_root.as_ref())
        .bind(db_item.consensus_messages_root.as_ref())
        .bind(db_item.consensus_signature.as_ref())
        .bind(db_item.consensus_transactions_root.as_ref())
        .fetch_one(executor)
        .await
        .map_err(RepositoryError::Insert)?;

        Ok(record)
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
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        blocks::packets::DynBlockSubject,
        infra::{DbConnectionOpts, DbItem, OrderBy, QueryParamsBuilder},
        mocks::MockBlock,
    };

    async fn create_test_block(
        height: u32,
        namespace: &str,
    ) -> (BlockDbItem, Block) {
        let block = MockBlock::build(height);
        let timestamp = BlockTimestamp::default();
        let dyn_subject = DynBlockSubject::new(
            block.height,
            block.producer.clone(),
            &block.header.da_height,
        );
        let packet = dyn_subject
            .build_packet(&block, timestamp)
            .with_namespace(namespace);
        let db_item = BlockDbItem::try_from(&packet).unwrap();
        (db_item, block)
    }

    async fn create_blocks(
        namespace: &str,
        db: &Db,
        count: u32,
    ) -> Vec<BlockDbItem> {
        let mut blocks = Vec::with_capacity(count as usize);
        for height in 1..=count {
            let (db_item, _) = create_test_block(height, namespace).await;
            Block::insert(db.pool_ref(), &db_item).await.unwrap();
            blocks.push(db_item);
        }
        blocks
    }

    #[tokio::test]
    async fn test_insert_block() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let (db_item, _) = create_test_block(1, &namespace).await;

        let result = Block::insert(db.pool_ref(), &db_item).await?;
        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.value, db_item.value);
        assert_eq!(result.block_da_height, db_item.block_da_height);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.producer_address, db_item.producer_address);
        Ok(())
    }

    #[tokio::test]
    async fn test_insert_block_with_transaction() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let mut tx = db.pool_ref().begin().await?;

        let (db_item1, _) = create_test_block(1, &namespace).await;
        let (db_item2, _) = create_test_block(2, &namespace).await;
        let result1 =
            Block::insert_with_transaction(&mut tx, &db_item1).await?;
        let result2 =
            Block::insert_with_transaction(&mut tx, &db_item2).await?;
        tx.commit().await?;

        assert_eq!(result1.subject, db_item1.subject);
        assert_eq!(result2.subject, db_item2.subject);
        Ok(())
    }

    #[tokio::test]
    async fn test_find_one_block() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let (db_item, block) = create_test_block(1, &namespace).await;
        let mut query = BlocksQuery::from(&block);
        query.with_namespace(Some(namespace));

        Block::insert(db.pool_ref(), &db_item).await?;
        let result = Block::find_one(db.pool_ref(), &query).await?;
        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.value, db_item.value);
        assert_eq!(result.block_da_height, db_item.block_da_height);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.producer_address, db_item.producer_address);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_blocks_basic_query() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let blocks = create_blocks(&namespace, &db, 3).await;
        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three blocks");
        assert_eq!(results[0].subject, blocks[0].subject);
        assert_eq!(results[1].subject, blocks[1].subject);
        assert_eq!(results[2].subject, blocks[2].subject);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_blocks_with_height_filter() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let blocks = create_blocks(&namespace, &db, 3).await;
        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace));
        query.with_from_block(Some(2.into()));
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
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let blocks = create_blocks(&namespace, &db, 5).await;

        // Test pagination with after cursor and first
        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_after(Some(blocks[1].cursor()));
        query.with_first(Some(2));
        // order_by is not needed for cursor-based pagination

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
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let blocks = create_blocks(&namespace, &db, 5).await;

        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_before(Some(blocks[4].cursor()));
        query.with_last(Some(2));

        let results = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 blocks before cursor"
        );
        assert_eq!(results[0].block_height, blocks[3].block_height); // Block 4
        assert_eq!(results[1].block_height, blocks[2].block_height); // Block 3

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_blocks_with_limit_offset_pagination() -> Result<()>
    {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let blocks = create_blocks(&namespace, &db, 5).await;

        // Test first page with explicit ordering
        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_limit(Some(2));
        query.with_offset(Some(0));
        query.with_order_by(OrderBy::Asc);

        let first_page = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(first_page.len(), 2, "First page should have 2 blocks");
        assert_eq!(first_page[0].block_height, blocks[0].block_height);
        assert_eq!(first_page[1].block_height, blocks[1].block_height);

        // Test second page
        query.with_offset(Some(2));
        let second_page = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(second_page.len(), 2, "Second page should have 2 blocks");
        assert_eq!(second_page[0].block_height, blocks[2].block_height);
        assert_eq!(second_page[1].block_height, blocks[3].block_height);

        // Test last page
        query.with_offset(Some(4));
        let last_page = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(last_page.len(), 1, "Last page should have 1 block");
        assert_eq!(last_page[0].block_height, blocks[4].block_height);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_blocks_with_different_order() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let blocks = create_blocks(&namespace, &db, 3).await;

        // Test ascending order
        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_eq!(asc_results[0].block_height, blocks[0].block_height);
        assert_eq!(asc_results[2].block_height, blocks[2].block_height);

        // Test descending order
        query.with_order_by(OrderBy::Desc);
        let desc_results = Block::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_eq!(desc_results[0].block_height, blocks[2].block_height);
        assert_eq!(desc_results[2].block_height, blocks[0].block_height);

        Ok(())
    }

    #[tokio::test]
    async fn test_cursor_pagination_ignores_order_by() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let blocks = create_blocks(&namespace, &db, 5).await;
        let mut query = BlocksQuery::default();
        query.with_namespace(Some(namespace.clone()));
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
}
