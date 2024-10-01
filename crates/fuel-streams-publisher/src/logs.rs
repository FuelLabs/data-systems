use std::sync::Arc;

use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::{logs::LogsSubject, prelude::*, Stream};
use tracing::info;

use crate::{
    build_subject_name,
    metrics::PublisherMetrics,
    publish_with_metrics,
};

#[allow(clippy::too_many_arguments)]
pub async fn publish(
    logs_stream: &Stream<Log>,
    receipts: Option<Vec<Receipt>>,
    tx_id: Bytes32,
    chain_id: &ChainId,
    block_height: BlockHeight,
    metrics: &Arc<PublisherMetrics>,
    block_producer: &Address,
    predicate_tag: Option<Bytes32>,
) -> anyhow::Result<()> {
    if let Some(receipts) = receipts {
        for (index, receipt) in receipts.iter().enumerate() {
            match receipt {
                Receipt::Log { id, .. } | Receipt::LogData { id, .. } => {
                    let subject = LogsSubject::new()
                        .with_block_height(Some(block_height.clone()))
                        .with_tx_id(Some(tx_id.clone()))
                        .with_receipt_index(Some(index))
                        .with_log_id(Some((*id).into()));
                    let subject_wildcard = LogsSubject::WILDCARD;

                    info!("NATS Publisher: Publishing Logs for 0x#{tx_id}");
                    publish_with_metrics!(
                        logs_stream.publish_raw(
                            &build_subject_name(&predicate_tag, &subject),
                            &(receipt.clone()).into(),
                        ),
                        metrics,
                        chain_id,
                        block_producer,
                        subject_wildcard
                    );
                }
                _non_log_receipt => {}
            }
        }
    }

    Ok(())
}
