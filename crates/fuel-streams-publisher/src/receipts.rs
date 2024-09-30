use std::sync::Arc;

use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::{
    prelude::*,
    receipts::*,
    types::IdentifierKind,
    Stream,
};
use tracing::info;

use crate::{
    build_subject_name,
    metrics::PublisherMetrics,
    publish_with_metrics,
};

pub async fn publish(
    receipts_stream: &Stream<Receipt>,
    receipts: Option<Vec<Receipt>>,
    tx_id: &Bytes32,
    chain_id: ChainId,
    metrics: &Arc<PublisherMetrics>,
    block_producer: &Address,
    predicate_tag: &Option<Bytes32>,
) -> anyhow::Result<()> {
    if let Some(receipts) = receipts {
        info!("NATS Publisher: Publishing Receipts for 0x#{tx_id}");

        for (index, receipt) in receipts.iter().enumerate() {
            let (subjects, subjects_wildcard) =
                receipt_subjects(receipt, tx_id.clone(), index);
            for (index, subject) in subjects.iter().enumerate() {
                publish_with_metrics!(
                    receipts_stream.publish_raw(
                        &build_subject_name(predicate_tag, &**subject),
                        receipt
                    ),
                    metrics,
                    chain_id,
                    block_producer,
                    subjects_wildcard
                        .get(index)
                        .expect("Wildcard must be provided")
                );
            }
        }
    }

    Ok(())
}

fn receipt_subjects(
    receipt: &Receipt,
    tx_id: Bytes32,
    index: usize,
) -> (Vec<Box<dyn IntoSubject>>, Vec<&'static str>) {
    match receipt {
        Receipt::Call {
            id: from,
            to,
            asset_id,
            ..
        } => (
            vec![
                ReceiptsCallSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_from(Some(from.into()))
                    .with_to(Some(to.into()))
                    .with_asset_id(Some(asset_id.into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*from).into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*to).into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::AssetID))
                    .with_id_value(Some((*asset_id).into()))
                    .boxed(),
            ],
            vec![
                ReceiptsCallSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
            ],
        ),
        Receipt::Return { id, .. } => (
            vec![
                ReceiptsReturnSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_id(Some(id.into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*id).into()))
                    .boxed(),
            ],
            vec![
                ReceiptsReturnSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
            ],
        ),
        Receipt::ReturnData { id, .. } => (
            vec![
                ReceiptsReturnDataSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_id(Some(id.into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*id).into()))
                    .boxed(),
            ],
            vec![
                ReceiptsReturnDataSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
            ],
        ),
        Receipt::Panic { id, .. } => (
            vec![
                ReceiptsPanicSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_id(Some(id.into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*id).into()))
                    .boxed(),
            ],
            vec![
                ReceiptsPanicSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
            ],
        ),
        Receipt::Revert { id, .. } => (
            vec![
                ReceiptsRevertSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_id(Some(id.into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*id).into()))
                    .boxed(),
            ],
            vec![
                ReceiptsRevertSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
            ],
        ),
        Receipt::Log { id, .. } => (
            vec![
                ReceiptsLogSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_id(Some(id.into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*id).into()))
                    .boxed(),
            ],
            vec![ReceiptsLogSubject::WILDCARD, ReceiptsByIdSubject::WILDCARD],
        ),
        Receipt::LogData { id, .. } => (
            vec![
                ReceiptsLogDataSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_id(Some(id.into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*id).into()))
                    .boxed(),
            ],
            vec![
                ReceiptsLogDataSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
            ],
        ),
        Receipt::Transfer {
            id: from,
            to,
            asset_id,
            ..
        } => (
            vec![
                ReceiptsTransferSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_from(Some(from.into()))
                    .with_to(Some(to.into()))
                    .with_asset_id(Some(asset_id.into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*from).into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*to).into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::AssetID))
                    .with_id_value(Some((*asset_id).into()))
                    .boxed(),
            ],
            vec![
                ReceiptsTransferSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
            ],
        ),

        Receipt::TransferOut {
            id: from,
            to,
            asset_id,
            ..
        } => (
            vec![
                ReceiptsTransferOutSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_from(Some(from.into()))
                    .with_to(Some(to.into()))
                    .with_asset_id(Some(asset_id.into()))
                    .boxed()
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*from).into()))
                    .boxed()
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*to).into()))
                    .boxed()
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::AssetID))
                    .with_id_value(Some((*asset_id).into()))
                    .boxed()
                    .boxed(),
            ],
            vec![
                ReceiptsTransferOutSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
            ],
        ),

        Receipt::ScriptResult { .. } => (
            vec![ReceiptsScriptResultSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .boxed()
                .boxed()],
            vec![ReceiptsScriptResultSubject::WILDCARD],
        ),

        Receipt::MessageOut {
            sender, recipient, ..
        } => (
            vec![
                ReceiptsMessageOutSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_sender(Some((sender).into()))
                    .with_recipient(Some((*recipient).into()))
                    .boxed()
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::Address))
                    .with_id_value(Some((*sender).into()))
                    .boxed()
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::Address))
                    .with_id_value(Some((*recipient).into()))
                    .boxed()
                    .boxed(),
            ],
            vec![
                ReceiptsMessageOutSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
            ],
        ),

        Receipt::Mint {
            contract_id,
            sub_id,
            ..
        } => (
            vec![
                ReceiptsMintSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_contract_id(Some(contract_id.into()))
                    .with_sub_id(Some((*sub_id).into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*contract_id).into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::Address))
                    .with_id_value(Some((*sub_id).into()))
                    .boxed(),
            ],
            vec![
                ReceiptsMintSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
            ],
        ),
        Receipt::Burn {
            sub_id,
            contract_id,
            ..
        } => (
            vec![
                ReceiptsBurnSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_contract_id(Some(contract_id.into()))
                    .with_sub_id(Some((*sub_id).into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*contract_id).into()))
                    .boxed(),
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::Address))
                    .with_id_value(Some((*sub_id).into()))
                    .boxed(),
            ],
            vec![
                ReceiptsBurnSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
                ReceiptsByIdSubject::WILDCARD,
            ],
        ),
    }
}
