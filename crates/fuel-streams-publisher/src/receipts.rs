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
                let subjects: &[&dyn IntoSubject] = match receipt {
                    Receipt::Call {
                        id: from,
                        to,
                        asset_id,
                        ..
                    } => &[
                        &ReceiptsCallSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index))
                            .with_from(Some(from.into()))
                            .with_to(Some(to.into()))
                            .with_asset_id(Some(asset_id.into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*from).into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*to).into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::AssetID))
                            .with_id_value(Some((*asset_id).into())),
                    ],
                    Receipt::Return { id, .. } => &[
                        &ReceiptsReturnSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index))
                            .with_id(Some(id.into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*id).into())),
                    ],
                    Receipt::ReturnData { id, .. } => &[
                        &ReceiptsReturnDataSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index))
                            .with_id(Some(id.into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*id).into())),
                    ],
                    Receipt::Panic { id, .. } => &[
                        &ReceiptsPanicSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index))
                            .with_id(Some(id.into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*id).into())),
                    ],
                    Receipt::Revert { id, .. } => &[
                        &ReceiptsRevertSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index))
                            .with_id(Some(id.into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*id).into())),
                    ],
                    Receipt::Log { id, .. } => &[
                        &ReceiptsLogSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index))
                            .with_id(Some(id.into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*id).into())),
                    ],
                    Receipt::LogData { id, .. } => &[
                        &ReceiptsLogDataSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index))
                            .with_id(Some(id.into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*id).into())),
                    ],
                    Receipt::Transfer {
                        id: from,
                        to,
                        asset_id,
                        ..
                    } => &[
                        &ReceiptsTransferSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index))
                            .with_from(Some(from.into()))
                            .with_to(Some(to.into()))
                            .with_asset_id(Some(asset_id.into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*from).into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*to).into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::AssetID))
                            .with_id_value(Some((*asset_id).into())),
                    ],

                    Receipt::TransferOut {
                        id: from,
                        to,
                        asset_id,
                        ..
                    } => &[
                        &ReceiptsTransferOutSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index))
                            .with_from(Some(from.into()))
                            .with_to(Some(to.into()))
                            .with_asset_id(Some(asset_id.into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*from).into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*to).into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::AssetID))
                            .with_id_value(Some((*asset_id).into())),
                    ],

                    Receipt::ScriptResult { .. } => {
                        &[&ReceiptsScriptResultSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index))]
                    }
                    Receipt::MessageOut {
                        sender, recipient, ..
                    } => &[
                        &ReceiptsMessageOutSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index))
                            .with_sender(Some((sender).into()))
                            .with_recipient(Some((*recipient).into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::Address))
                            .with_id_value(Some((*sender).into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::Address))
                            .with_id_value(Some((*recipient).into())),
                    ],

                    Receipt::Mint {
                        contract_id,
                        sub_id,
                        ..
                    } => &[
                        &ReceiptsMintSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index))
                            .with_contract_id(Some(contract_id.into()))
                            .with_sub_id(Some((*sub_id).into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*contract_id).into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::Address))
                            .with_id_value(Some((*sub_id).into())),
                    ],
                    Receipt::Burn {
                        sub_id,
                        contract_id,
                        ..
                    } => &[
                        &ReceiptsBurnSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index))
                            .with_contract_id(Some(contract_id.into()))
                            .with_sub_id(Some((*sub_id).into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some((*contract_id).into())),
                        &ReceiptsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::Address))
                            .with_id_value(Some((*sub_id).into())),
                    ],
                };

                receipts_stream.publish_many(subjects, receipt).await?;
            }
        }
    }

    Ok(())
}
