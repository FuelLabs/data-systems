use std::sync::Arc;

use fuel_core_storage::transactional::AtomicView;
use fuel_streams::types::Address;
use fuel_streams_core::prelude::*;
use tracing::info;

use crate::{
    identifiers::{
        add_predicate_subjects,
        add_script_subjects,
        IdSubjectsMutator,
    },
    metrics::PublisherMetrics,
    publish_all,
    FuelCoreLike,
    PublishPayload,
};

#[allow(clippy::too_many_arguments)]
pub async fn publish(
    stream: &Stream<Transaction>,
    (transaction_index, transaction): (usize, &Transaction),
    fuel_core: &dyn FuelCoreLike,
    block_height: BlockHeight,
    metrics: &Arc<PublisherMetrics>,
    block_producer: &Address,
    predicate_tag: Option<Bytes32>,
    script_tag: Option<Bytes32>,
) -> anyhow::Result<()> {
    let chain_id = fuel_core.chain_id();
    let off_chain_database = fuel_core.database().off_chain().latest_view()?;
    let tx_id = transaction.id(chain_id);
    let kind = TransactionKind::from(transaction.to_owned());
    let status: TransactionStatus = off_chain_database
        .get_tx_status(&tx_id)?
        .map(|status| status.into())
        .unwrap_or_default();

    let mut subjects: Vec<(Box<dyn IntoSubject>, &'static str)> = vec![(
        TransactionsSubject::new()
            .with_tx_id(Some(tx_id.into()))
            .with_kind(Some(kind))
            .with_status(Some(status))
            .with_block_height(Some(block_height))
            .with_tx_index(Some(transaction_index))
            .boxed(),
        TransactionsSubject::WILDCARD,
    )];

    add_predicate_subjects::<Transaction>(&mut subjects, predicate_tag);
    add_script_subjects::<Transaction>(&mut subjects, script_tag);

    info!("NATS Publisher: Publishing Transaction 0x#{tx_id}");

    publish_all(PublishPayload {
        stream,
        subjects,
        payload: transaction,
        metrics,
        chain_id,
        block_producer,
    })
    .await;

    Ok(())
}
