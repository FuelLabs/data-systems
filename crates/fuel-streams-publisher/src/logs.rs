use std::sync::Arc;

use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::{
    logs::LogsSubject,
    prelude::*,
    types::{Transaction, UniqueIdentifier},
    Stream,
};
use tracing::info;

use crate::{metrics::PublisherMetrics, publish_with_metrics, FuelCoreLike};

pub async fn publish(
    metrics: &Arc<PublisherMetrics>,
    fuel_core: &dyn FuelCoreLike,
    logs_stream: &Stream<Log>,
    transactions: &[Transaction],
    block_producer: &Address,
    block_height: BlockHeight,
) -> anyhow::Result<()> {
    let chain_id = fuel_core.chain_id();

    for transaction in transactions.iter() {
        let tx_id = transaction.id(chain_id);
        let receipts = fuel_core.get_receipts(&tx_id)?;

        if let Some(receipts) = receipts {
            for (index, receipt) in receipts.iter().enumerate() {
                match receipt {
                    receipt @ (Receipt::Log { id, .. }
                    | Receipt::LogData { id, .. }) => {
                        let subject = LogsSubject::new()
                            .with_block_height(Some(block_height.clone()))
                            .with_tx_id(Some(tx_id.into()))
                            .with_receipt_index(Some(index))
                            .with_log_id(Some((*id).into()));
                        let subject_wildcard = LogsSubject::WILDCARD;

                        info!("NATS Publisher: Publishing Logs for 0x#{tx_id}");
                        publish_with_metrics!(
                            logs_stream
                                .publish(&subject, &(receipt.clone()).into()),
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
    }

    Ok(())
}
