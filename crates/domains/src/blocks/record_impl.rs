use async_trait::async_trait;
use fuel_streams_store::{
    db::{DbError, DbResult},
    record::{DataEncoder, Record, RecordEntity},
};
use fuel_streams_types::BlockTimestamp;
use sqlx::PgExecutor;

use super::{Block, BlockDbItem};

impl DataEncoder for Block {
    type Err = DbError;
}

#[async_trait]
impl Record for Block {
    type DbItem = BlockDbItem;

    const ENTITY: RecordEntity = RecordEntity::Block;
    const ORDER_PROPS: &'static [&'static str] = &["block_height"];

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: Self::DbItem,
    ) -> DbResult<Self::DbItem>
    where
        'c: 'e,
        E: PgExecutor<'c>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, BlockDbItem>(
            "WITH upsert AS (
                INSERT INTO blocks (
                    subject,
                    block_height,
                    block_da_height,
                    value,
                    version,
                    producer_address,
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
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28)
                ON CONFLICT (subject) DO UPDATE SET
                    block_height = EXCLUDED.block_height,
                    block_da_height = EXCLUDED.block_da_height,
                    value = EXCLUDED.value,
                    version = EXCLUDED.version,
                    producer_address = EXCLUDED.producer_address,
                    created_at = EXCLUDED.created_at,
                    published_at = $8,
                    block_propagation_ms = $9,
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
            SELECT * FROM upsert",
        )
        .bind(db_item.subject)
        .bind(db_item.block_height)
        .bind(db_item.block_da_height)
        .bind(db_item.value)
        .bind(db_item.version)
        .bind(db_item.producer_address)
        .bind(db_item.created_at)
        .bind(published_at)
        .bind(db_item.block_propagation_ms)
        .bind(db_item.header_application_hash)
        .bind(db_item.header_consensus_parameters_version)
        .bind(db_item.header_da_height)
        .bind(db_item.header_event_inbox_root)
        .bind(db_item.header_message_outbox_root)
        .bind(db_item.header_message_receipt_count)
        .bind(db_item.header_prev_root)
        .bind(db_item.header_state_transition_bytecode_version)
        .bind(db_item.header_time)
        .bind(db_item.header_transactions_count)
        .bind(db_item.header_transactions_root)
        .bind(db_item.header_version)
        .bind(db_item.consensus_chain_config_hash)
        .bind(db_item.consensus_coins_root)
        .bind(db_item.consensus_type)
        .bind(db_item.consensus_contracts_root)
        .bind(db_item.consensus_messages_root)
        .bind(db_item.consensus_signature)
        .bind(db_item.consensus_transactions_root)
        .fetch_one(executor)
        .await
        .map_err(DbError::Insert)?;

        Ok(record)
    }
}
