use std::sync::Arc;

use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::prelude::*;
use futures::{StreamExt, TryStreamExt};
use rayon::prelude::*;

use crate::{
    metrics::PublisherMetrics,
    FuelCoreLike,
    PublishError,
    PublishPayload,
    SubjectPayload,
    CONCURRENCY_LIMIT,
};

pub async fn publish_tasks(
    stream: &Stream<Log>,
    transactions: &[Transaction],
    chain_id: &ChainId,
    block_producer: &Address,
    block_height: &BlockHeight,
    metrics: &Arc<PublisherMetrics>,
    fuel_core: &dyn FuelCoreLike,
) -> Result<(), PublishError> {
    futures::stream::iter(
        transactions
            .iter()
            .flat_map(|tx| {
                let tx_id = tx.id(chain_id);
                let receipts= fuel_core.get_receipts(&tx_id).unwrap_or_default();
                create_publish_payloads(stream, tx, &receipts, chain_id, block_height)
            }),
    )
    .map(Ok)
    .try_for_each_concurrent(*CONCURRENCY_LIMIT, |payload| {
        let metrics = metrics.clone();
        let chain_id = chain_id.to_owned();
        let block_producer = block_producer.clone();
        async move {
            payload.publish(&metrics, &chain_id, &block_producer).await
        }
    })
    .await
}

fn create_publish_payloads(
    stream: &Stream<Log>,
    tx: &Transaction,
    receipts: &Option<Vec<Receipt>>,
    chain_id: &ChainId,
    block_height: &BlockHeight,
) -> Vec<PublishPayload<Log>> {
    let tx_id = tx.id(chain_id);
    if let Some(receipts) = receipts {
        receipts
            .par_iter()
            .enumerate()
            .flat_map_iter(|(index, receipt)| {
                build_log_payloads(
                    stream,
                    block_height,
                    tx_id.into(),
                    receipt,
                    index,
                )
            })
            .collect()
    } else {
        Vec::new()
    }
}

fn build_log_payloads(
    stream: &Stream<Log>,
    block_height: &BlockHeight,
    tx_id: Bytes32,
    receipt: &Receipt,
    index: usize,
) -> Vec<PublishPayload<Log>> {
    let subjects: Vec<SubjectPayload> = match receipt {
        Receipt::Log { id, .. } | Receipt::LogData { id, .. } => {
            vec![(
                LogsSubject::new()
                    .with_block_height(Some(block_height.clone()))
                    .with_tx_id(Some(tx_id))
                    .with_receipt_index(Some(index))
                    .with_log_id(Some((*id).into()))
                    .boxed(),
                LogsSubject::WILDCARD,
            )]
        }
        _ => vec![],
    };

    subjects
        .into_par_iter()
        .map(|subject| PublishPayload {
            subject,
            stream: stream.to_owned(),
            payload: receipt.clone().into(),
        })
        .collect()
}
