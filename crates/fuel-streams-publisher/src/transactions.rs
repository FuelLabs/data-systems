use std::sync::Arc;

use fuel_core_storage::transactional::AtomicView;
use fuel_streams::types::Address;
use fuel_streams_core::{
    prelude::*,
    transactions::TransactionsSubject,
    types::{
        BlockHeight, Transaction, TransactionKind, TransactionStatus,
        UniqueIdentifier,
    },
    Stream,
};
use tracing::info;

use crate::{
    build_subject_name, metrics::PublisherMetrics, publish_with_metrics,
    FuelCoreLike,
};

#[allow(clippy::too_many_arguments)]
pub async fn publish(
    transactions_stream: &Stream<Transaction>,
    (transaction_index, transaction): (usize, &Transaction),
    fuel_core: &dyn FuelCoreLike,
    block_height: BlockHeight,
    metrics: &Arc<PublisherMetrics>,
    block_producer: &Address,
    predicate_tag: Option<Bytes32>,
) -> anyhow::Result<()> {
    let chain_id = fuel_core.chain_id();
    let off_chain_database = fuel_core.database().off_chain().latest_view()?;

    let tx_id = transaction.id(chain_id);
    let kind = TransactionKind::from(transaction.to_owned());
    let status: TransactionStatus = off_chain_database
        .get_tx_status(&tx_id)?
        .map(|status| status.into())
        .unwrap_or_default();

    let transactions_subject = TransactionsSubject::new()
        .with_tx_id(Some(tx_id.into()))
        .with_kind(Some(kind))
        .with_status(Some(status))
        .with_height(Some(block_height))
        .with_tx_index(Some(transaction_index));

    info!("NATS Publisher: Publishing Transaction 0x#{tx_id}");

    publish_with_metrics!(
        transactions_stream.publish_raw(
            &build_subject_name(&predicate_tag, &transactions_subject),
            transaction,
        ),
        metrics,
        chain_id,
        block_producer,
        TransactionsSubject::WILDCARD
    );

    Ok(())
}
