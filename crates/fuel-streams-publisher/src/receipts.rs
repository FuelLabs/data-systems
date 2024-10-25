use std::sync::Arc;

use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::prelude::*;
use futures::{StreamExt, TryStreamExt};
use rayon::prelude::*;

use crate::{
    identifiers::{Identifier, IdsExtractable, SubjectPayloadBuilder},
    metrics::PublisherMetrics,
    FuelCoreLike,
    PublishError,
    PublishPayload,
    SubjectPayload,
    CONCURRENCY_LIMIT,
};

pub async fn publish_tasks(
    stream: &Stream<Receipt>,
    transactions: &[Transaction],
    chain_id: &ChainId,
    block_producer: &Address,
    metrics: &Arc<PublisherMetrics>,
    fuel_core: &dyn FuelCoreLike,
) -> Result<(), PublishError> {
    futures::stream::iter(transactions.iter().flat_map(|tx| {
        let tx_id = tx.id(chain_id);
        let receipts = fuel_core.get_receipts(&tx_id).unwrap_or_default();
        create_publish_payloads(tx, &receipts, chain_id)
    }))
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
    receipts: &Option<Vec<Receipt>>,
    chain_id: &ChainId,
) -> Vec<PublishPayload<Receipt>> {
    let tx_id = tx.id(chain_id);
    if let Some(receipts) = receipts {
        receipts
            .par_iter()
            .enumerate()
            .flat_map_iter(|(index, receipt)| {
                build_receipt_payloads(tx, tx_id.into(), receipt, index)
            })
            .collect()
    } else {
        Vec::new()
    }
}

fn build_receipt_payloads(
    tx: &Transaction,
    tx_id: Bytes32,
    receipt: &Receipt,
    index: usize,
) -> Vec<PublishPayload<Receipt>> {
    main_subjects(receipt, tx_id, index)
        .into_par_iter()
        .chain(ReceiptsByIdSubject::build_subjects_payload(tx, &[receipt]))
        .map(|subject| PublishPayload {
            subject,
            payload: receipt.to_owned(),
        })
        .collect::<Vec<PublishPayload<Receipt>>>()
}

fn main_subjects(
    receipt: &Receipt,
    tx_id: Bytes32,
    index: usize,
) -> Vec<SubjectPayload> {
    match receipt {
        Receipt::Call {
            id: from,
            to,
            asset_id,
            ..
        } => vec![(
            ReceiptsCallSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_from(Some(from.into()))
                .with_to(Some(to.into()))
                .with_asset_id(Some(asset_id.into()))
                .boxed(),
            ReceiptsCallSubject::WILDCARD,
        )],
        Receipt::Return { id, .. } => vec![(
            ReceiptsReturnSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .boxed(),
            ReceiptsReturnSubject::WILDCARD,
        )],
        Receipt::ReturnData { id, .. } => vec![(
            ReceiptsReturnDataSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .boxed(),
            ReceiptsReturnDataSubject::WILDCARD,
        )],
        Receipt::Panic { id, .. } => vec![(
            ReceiptsPanicSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .boxed(),
            ReceiptsPanicSubject::WILDCARD,
        )],
        Receipt::Revert { id, .. } => vec![(
            ReceiptsRevertSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .boxed(),
            ReceiptsRevertSubject::WILDCARD,
        )],
        Receipt::Log { id, .. } => vec![(
            ReceiptsLogSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .boxed(),
            ReceiptsLogSubject::WILDCARD,
        )],
        Receipt::LogData { id, .. } => vec![(
            ReceiptsLogDataSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .boxed(),
            ReceiptsLogDataSubject::WILDCARD,
        )],

        Receipt::Transfer {
            id: from,
            to,
            asset_id,
            ..
        } => vec![(
            ReceiptsTransferSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_from(Some(from.into()))
                .with_to(Some(to.into()))
                .with_asset_id(Some(asset_id.into()))
                .boxed(),
            ReceiptsTransferSubject::WILDCARD,
        )],
        Receipt::TransferOut {
            id: from,
            to,
            asset_id,
            ..
        } => vec![(
            ReceiptsTransferOutSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_from(Some(from.into()))
                .with_to(Some(to.into()))
                .with_asset_id(Some(asset_id.into()))
                .boxed(),
            ReceiptsTransferOutSubject::WILDCARD,
        )],
        Receipt::ScriptResult { .. } => vec![(
            ReceiptsScriptResultSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .boxed()
                .boxed(),
            ReceiptsScriptResultSubject::WILDCARD,
        )],
        Receipt::MessageOut {
            sender, recipient, ..
        } => vec![(
            ReceiptsMessageOutSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_sender(Some(sender.into()))
                .with_recipient(Some(recipient.into()))
                .boxed(),
            ReceiptsMessageOutSubject::WILDCARD,
        )],
        Receipt::Mint {
            contract_id,
            sub_id,
            ..
        } => vec![(
            ReceiptsMintSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_contract_id(Some(contract_id.into()))
                .with_sub_id(Some((*sub_id).into()))
                .boxed(),
            ReceiptsMintSubject::WILDCARD,
        )],
        Receipt::Burn {
            contract_id,
            sub_id,
            ..
        } => vec![(
            ReceiptsBurnSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_contract_id(Some(contract_id.into()))
                .with_sub_id(Some((*sub_id).into()))
                .boxed(),
            ReceiptsBurnSubject::WILDCARD,
        )],
    }
}

impl IdsExtractable for Receipt {
    fn extract_identifiers(&self, _tx: &Transaction) -> Vec<Identifier> {
        match self {
            Receipt::Call {
                id: from,
                to,
                asset_id,
                ..
            } => {
                vec![
                    Identifier::ContractId(from.into()),
                    Identifier::ContractId(to.into()),
                    Identifier::AssetId(asset_id.into()),
                ]
            }
            Receipt::Return { id, .. }
            | Receipt::ReturnData { id, .. }
            | Receipt::Panic { id, .. }
            | Receipt::Revert { id, .. }
            | Receipt::Log { id, .. }
            | Receipt::LogData { id, .. } => {
                vec![Identifier::ContractId(id.into())]
            }
            Receipt::Transfer {
                id: from,
                to,
                asset_id,
                ..
            } => {
                vec![
                    Identifier::ContractId(from.into()),
                    Identifier::ContractId(to.into()),
                    Identifier::AssetId(asset_id.into()),
                ]
            }
            Receipt::TransferOut {
                id: from,
                to,
                asset_id,
                ..
            } => {
                vec![
                    Identifier::ContractId(from.into()),
                    Identifier::ContractId(to.into()),
                    Identifier::AssetId(asset_id.into()),
                ]
            }
            Receipt::MessageOut {
                sender, recipient, ..
            } => {
                vec![
                    Identifier::Address(sender.into()),
                    Identifier::Address(recipient.into()),
                ]
            }
            Receipt::Mint { contract_id, .. }
            | Receipt::Burn { contract_id, .. } => {
                vec![Identifier::ContractId(contract_id.into())]
            }
            _ => Vec::new(),
        }
    }
}
