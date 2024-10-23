use std::sync::Arc;

use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::prelude::*;
use tracing::info;

use crate::{
    identifiers::{add_predicate_subjects, add_script_subjects},
    metrics::PublisherMetrics,
    publish_all,
    PublishPayload,
};

#[allow(clippy::too_many_arguments)]
pub async fn publish(
    stream: &Stream<Receipt>,
    receipts: Option<Vec<Receipt>>,
    tx_id: Bytes32,
    chain_id: &ChainId,
    metrics: &Arc<PublisherMetrics>,
    block_producer: &Address,
    predicate_tag: Option<Bytes32>,
    script_tag: Option<Bytes32>,
) -> anyhow::Result<()> {
    if let Some(receipts) = receipts {
        info!("NATS Publisher: Publishing Receipts for 0x#{tx_id}");

        for (index, receipt) in receipts.iter().enumerate() {
            let mut subjects = receipt_subjects(receipt, tx_id.clone(), index);

            let predicate_tag = predicate_tag.clone();
            let script_tag = script_tag.clone();
            add_predicate_subjects::<Receipt>(&mut subjects, predicate_tag);
            add_script_subjects::<Receipt>(&mut subjects, script_tag);

            publish_all(PublishPayload {
                stream,
                subjects,
                payload: receipt,
                metrics,
                chain_id,
                block_producer,
            })
            .await;
        }
    }

    Ok(())
}

fn receipt_subjects(
    receipt: &Receipt,
    tx_id: Bytes32,
    index: usize,
) -> Vec<(Box<dyn IntoSubject>, &'static str)> {
    match receipt {
        Receipt::Call {
            id: from,
            to,
            asset_id,
            ..
        } => vec![
            (
                ReceiptsCallSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_from(Some(from.into()))
                    .with_to(Some(to.into()))
                    .with_asset_id(Some(asset_id.into()))
                    .boxed(),
                ReceiptsCallSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*from).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*to).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::AssetID))
                    .with_id_value(Some((*asset_id).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
        ],
        Receipt::Return { id, .. } => vec![
            (
                ReceiptsReturnSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_id(Some(id.into()))
                    .boxed(),
                ReceiptsReturnSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*id).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
        ],
        Receipt::ReturnData { id, .. } => vec![
            (
                ReceiptsReturnDataSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_id(Some(id.into()))
                    .boxed(),
                ReceiptsReturnDataSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*id).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
        ],

        Receipt::Panic { id, .. } => vec![
            (
                ReceiptsPanicSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_id(Some(id.into()))
                    .boxed(),
                ReceiptsPanicSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*id).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
        ],
        Receipt::Revert { id, .. } => vec![
            (
                ReceiptsRevertSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_id(Some(id.into()))
                    .boxed(),
                ReceiptsRevertSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*id).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
        ],

        Receipt::Log { id, .. } => vec![
            (
                ReceiptsLogSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_id(Some(id.into()))
                    .boxed(),
                ReceiptsLogSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*id).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
        ],
        Receipt::LogData { id, .. } => vec![
            (
                ReceiptsLogDataSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_id(Some(id.into()))
                    .boxed(),
                ReceiptsLogDataSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*id).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
        ],

        Receipt::Transfer {
            id: from,
            to,
            asset_id,
            ..
        } => vec![
            (
                ReceiptsTransferSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_from(Some(from.into()))
                    .with_to(Some(to.into()))
                    .with_asset_id(Some(asset_id.into()))
                    .boxed(),
                ReceiptsTransferSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*from).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*to).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::AssetID))
                    .with_id_value(Some((*asset_id).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
        ],

        Receipt::TransferOut {
            id: from,
            to,
            asset_id,
            ..
        } => vec![
            (
                ReceiptsTransferOutSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_from(Some(from.into()))
                    .with_to(Some(to.into()))
                    .with_asset_id(Some(asset_id.into()))
                    .boxed(),
                ReceiptsTransferOutSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*from).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*to).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::AssetID))
                    .with_id_value(Some((*asset_id).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
        ],

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
        } => vec![
            (
                ReceiptsMessageOutSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_sender(Some(sender.into()))
                    .with_recipient(Some(recipient.into()))
                    .boxed(),
                ReceiptsMessageOutSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::Address))
                    .with_id_value(Some((*sender).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::Address))
                    .with_id_value(Some((*recipient).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
        ],

        Receipt::Mint {
            contract_id,
            sub_id,
            ..
        } => vec![
            (
                ReceiptsMintSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_contract_id(Some(contract_id.into()))
                    .with_sub_id(Some((*sub_id).into()))
                    .boxed(),
                ReceiptsMintSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*contract_id).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::Address))
                    .with_id_value(Some((*sub_id).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
        ],
        Receipt::Burn {
            sub_id,
            contract_id,
            ..
        } => vec![
            (
                ReceiptsBurnSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_contract_id(Some(contract_id.into()))
                    .with_sub_id(Some((*sub_id).into()))
                    .boxed(),
                ReceiptsBurnSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some((*contract_id).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
            (
                ReceiptsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::Address))
                    .with_id_value(Some((*sub_id).into()))
                    .boxed(),
                ReceiptsByIdSubject::WILDCARD,
            ),
        ],
    }
}
