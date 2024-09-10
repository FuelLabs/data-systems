use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::{
    prelude::*,
    receipts::*,
    types::{IdentifierKind, Transaction, UniqueIdentifier},
    Stream,
};
use tracing::info;

use crate::FuelCoreLike;

pub async fn publish(
    fuel_core: &dyn FuelCoreLike,
    receipts_stream: &Stream<Receipt>,
    transactions: &[Transaction],
) -> anyhow::Result<()> {
    let chain_id = fuel_core.chain_id();

    for transaction in transactions.iter() {
        let tx_id = transaction.id(chain_id);
        let receipts = fuel_core.get_receipts(&tx_id)?;

        if let Some(receipts) = receipts {
            info!("NATS Publisher: Publishing Receipts for 0x#{tx_id}");

            for (index, receipt) in receipts.iter().enumerate() {
                let subjects = receipt_subjects(receipt, tx_id.into(), index);
                receipts_stream.publish_many(&subjects, receipt).await?;
            }
        }
    }

    Ok(())
}

fn receipt_subjects(
    receipt: &Receipt,
    tx_id: Bytes32,
    index: usize,
) -> Vec<Box<dyn IntoSubject>> {
    match receipt {
        Receipt::Call {
            id: from,
            to,
            asset_id,
            ..
        } => vec![
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
        Receipt::Return { id, .. } => vec![
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
        Receipt::ReturnData { id, .. } => vec![
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
        Receipt::Panic { id, .. } => vec![
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
        Receipt::Revert { id, .. } => vec![
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
        Receipt::Log { id, .. } => vec![
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
        Receipt::LogData { id, .. } => vec![
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
        Receipt::Transfer {
            id: from,
            to,
            asset_id,
            ..
        } => vec![
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

        Receipt::TransferOut {
            id: from,
            to,
            asset_id,
            ..
        } => vec![
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

        Receipt::ScriptResult { .. } => {
            vec![ReceiptsScriptResultSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .boxed()
                .boxed()]
        }
        Receipt::MessageOut {
            sender, recipient, ..
        } => vec![
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

        Receipt::Mint {
            contract_id,
            sub_id,
            ..
        } => vec![
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
        Receipt::Burn {
            sub_id,
            contract_id,
            ..
        } => vec![
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
    }
}
