use std::sync::Arc;

use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::fuel_tx::field::ScriptData;
use fuel_streams_core::{prelude::*, transactions::TransactionExt};
use futures::{StreamExt, TryStreamExt};
use rayon::prelude::*;

use crate::{
    identifiers::*,
    metrics::PublisherMetrics,
    sha256,
    FuelCoreLike,
    PublishError,
    PublishPayload,
    SubjectPayload,
    CONCURRENCY_LIMIT,
};

pub async fn publish_tasks(
    stream: &Stream<Transaction>,
    transactions: &[Transaction],
    chain_id: &ChainId,
    block_height: &BlockHeight,
    block_producer: &Address,
    metrics: &Arc<PublisherMetrics>,
    fuel_core: &dyn FuelCoreLike,
) -> Result<(), PublishError> {
    futures::stream::iter(transactions.iter().enumerate().flat_map(
        |(tx_index, tx)| {
            let tx_id = tx.id(chain_id);
            let receipts = fuel_core.get_receipts(&tx_id).unwrap_or_default();
            create_publish_payloads(
                tx,
                tx_index,
                fuel_core,
                chain_id,
                block_height,
                receipts,
            )
        },
    ))
    .map(Ok)
    .try_for_each_concurrent(*CONCURRENCY_LIMIT, |payload| {
        let metrics = metrics.clone();
        let chain_id = chain_id.to_owned();
        let block_producer = block_producer.clone();
        async move {
            payload
                .publish(stream, &metrics, &chain_id, &block_producer)
                .await
        }
    })
    .await
}

fn create_publish_payloads(
    tx: &Transaction,
    tx_index: usize,
    fuel_core: &dyn FuelCoreLike,
    chain_id: &ChainId,
    block_height: &BlockHeight,
    receipts: Option<Vec<Receipt>>,
) -> Vec<PublishPayload<Transaction>> {
    let off_chain_database =
        fuel_core.database().off_chain().latest_view().unwrap();
    let tx_id = tx.id(chain_id);
    let kind = TransactionKind::from(tx.to_owned());
    let inputs: Vec<&Input> = tx.inputs().iter().collect();
    let outputs: Vec<&Output> = tx.outputs().iter().collect();
    let receipts = receipts.unwrap_or_default();
    let receipts: Vec<&Receipt> = receipts.iter().collect();
    let status: TransactionStatus = off_chain_database
        .get_tx_status(&tx_id)
        .unwrap()
        .map(|status| status.into())
        .unwrap_or_default();

    let subjects: Vec<SubjectPayload> = vec![(
        TransactionsSubject::new()
            .with_tx_id(Some(tx_id.into()))
            .with_kind(Some(kind))
            .with_status(Some(status))
            .with_block_height(Some(block_height.to_owned()))
            .with_tx_index(Some(tx_index))
            .boxed(),
        TransactionsSubject::WILDCARD,
    )];

    let tx_subjects =
        TransactionsByIdSubject::build_subjects_payload(tx, &[tx]);
    let tx_inputs_subjects =
        TransactionsByIdSubject::build_subjects_payload(tx, &inputs);
    let tx_outputs_subjects =
        TransactionsByIdSubject::build_subjects_payload(tx, &outputs);
    let tx_receipts_subjects =
        TransactionsByIdSubject::build_subjects_payload(tx, &receipts);

    subjects
        .into_par_iter()
        .chain(tx_subjects)
        .chain(tx_inputs_subjects)
        .chain(tx_outputs_subjects)
        .chain(tx_receipts_subjects)
        .map(|subject| PublishPayload {
            subject,
            payload: tx.to_owned(),
        })
        .collect()
}

impl IdsExtractable for Transaction {
    fn extract_identifiers(&self, _tx: &Transaction) -> Vec<Identifier> {
        match self {
            Transaction::Script(tx) => {
                let script_tag = sha256(tx.script_data());
                vec![Identifier::ScriptId(script_tag)]
            }
            _ => Vec::new(),
        }
    }
}
