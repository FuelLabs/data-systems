use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::prelude::*;
use rayon::prelude::*;

use crate::{PublishPayload, SubjectPayload};

pub fn create_publish_payloads(
    stream: &Stream<Log>,
    tx: &Transaction,
    receipts: Option<Vec<Receipt>>,
    chain_id: &ChainId,
    block_height: BlockHeight,
) -> Vec<PublishPayload<Log>> {
    let tx_id = tx.id(chain_id);
    if let Some(receipts) = receipts {
        receipts
            .par_iter()
            .enumerate()
            .map(|(index, receipt)| {
                let subjects: Vec<SubjectPayload> = match receipt {
                    Receipt::Log { id, .. } | Receipt::LogData { id, .. } => {
                        vec![(
                            LogsSubject::new()
                                .with_block_height(Some(block_height.clone()))
                                .with_tx_id(Some(tx_id.into()))
                                .with_receipt_index(Some(index))
                                .with_log_id(Some((*id).into()))
                                .boxed(),
                            LogsSubject::WILDCARD,
                        )]
                    }
                    _ => vec![],
                };

                PublishPayload {
                    subjects,
                    stream: stream.to_owned(),
                    payload: receipt.clone().into(),
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}
