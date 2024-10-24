use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::prelude::*;
use rayon::prelude::*;

use crate::{
    identifiers::{Identifier, IdsExtractable, SubjectPayloadBuilder},
    PublishPayload,
    SubjectPayload,
};

pub fn create_publish_payloads(
    stream: &Stream<Receipt>,
    tx: &Transaction,
    receipts: Option<Vec<Receipt>>,
    chain_id: &ChainId,
) -> Vec<PublishPayload<Receipt>> {
    let tx_id = tx.id(chain_id);

    if let Some(receipts) = receipts {
        receipts
            .par_iter()
            .enumerate()
            .map(|(index, receipt)| {
                let subjects = main_subjects(receipt, tx_id.into(), index)
                    .into_iter()
                    .chain(ReceiptsByIdSubject::build_subjects_payload(
                        tx,
                        &[receipt.to_owned()],
                    ))
                    .collect();

                PublishPayload {
                    subjects,
                    stream: stream.to_owned(),
                    payload: receipt.to_owned(),
                }
            })
            .collect()
    } else {
        Vec::new()
    }
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
