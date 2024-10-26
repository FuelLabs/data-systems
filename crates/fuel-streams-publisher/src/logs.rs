use std::sync::Arc;

use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::prelude::*;
use tracing::info;

use crate::{
    elastic::ElasticSearch,
    log_all,
    maybe_include_predicate_and_script_subjects,
    metrics::PublisherMetrics,
    publish_all,
};

#[allow(clippy::too_many_arguments)]
pub async fn publish(
    elastic_logger: &Option<Arc<ElasticSearch>>,
    stream: &Stream<Log>,
    receipts: Option<Vec<Receipt>>,
    tx_id: Bytes32,
    chain_id: &ChainId,
    block_height: BlockHeight,
    metrics: &Arc<PublisherMetrics>,
    block_producer: &Address,
    predicate_tag: Option<Bytes32>,
    script_tag: Option<Bytes32>,
) -> anyhow::Result<()> {
    if let Some(receipts) = receipts {
        for (index, receipt) in receipts.iter().enumerate() {
            match receipt {
                Receipt::Log { id, .. } | Receipt::LogData { id, .. } => {
                    let mut subjects: Vec<(
                        Box<dyn IntoSubject>,
                        &'static str,
                    )> = vec![(
                        LogsSubject::new()
                            .with_block_height(Some(block_height.clone()))
                            .with_tx_id(Some(tx_id.clone()))
                            .with_receipt_index(Some(index))
                            .with_log_id(Some((*id).into()))
                            .boxed(),
                        LogsSubject::WILDCARD,
                    )];

                    maybe_include_predicate_and_script_subjects(
                        &mut subjects,
                        &predicate_tag,
                        &script_tag,
                    );

                    info!("NATS Publisher: Publishing Logs for 0x#{tx_id}");

                    publish_all(
                        stream,
                        &subjects,
                        &(receipt.clone()).into(),
                        metrics,
                        chain_id,
                        block_producer,
                    )
                    .await;

                    log_all(elastic_logger, &subjects, &(receipt.clone()))
                        .await;
                }
                _non_log_receipt => {}
            }
        }
    }

    Ok(())
}
