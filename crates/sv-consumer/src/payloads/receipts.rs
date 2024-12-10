use std::sync::Arc;

use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::publisher::{publish, PublishOpts};

pub fn publish_tasks(
    tx_id: &Bytes32,
    stream: &Stream<Receipt>,
    opts: &Arc<PublishOpts>,
    receipts: &Vec<FuelCoreReceipt>,
) -> Vec<JoinHandle<anyhow::Result<()>>> {
    let packets: Vec<PublishPacket<Receipt>> = receipts
        .par_iter()
        .enumerate()
        .flat_map(|(index, receipt)| {
            let main_subject = main_subject(receipt, tx_id, index);
            let identifier_subjects = identifiers(receipt, tx_id, index as u8)
                .into_par_iter()
                .map(|identifier| identifier.into())
                .map(|subject: ReceiptsByIdSubject| subject.arc())
                .collect::<Vec<_>>();

            let receipt: Receipt = receipt.into();

            let mut packets = vec![receipt.to_packet(main_subject)];
            packets.extend(
                identifier_subjects
                    .into_iter()
                    .map(|subject| receipt.to_packet(subject)),
            );

            packets
        })
        .collect();

    packets
        .iter()
        .map(|packet| publish(packet, Arc::new(stream.to_owned()), opts))
        .collect()
}

fn main_subject(
    receipt: &FuelCoreReceipt,
    tx_id: &Bytes32,
    index: usize,
) -> Arc<dyn IntoSubject> {
    match receipt {
        FuelCoreReceipt::Call {
            id: from,
            to,
            asset_id,
            ..
        } => ReceiptsCallSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            from: Some(from.into()),
            to: Some(to.into()),
            asset_id: Some(asset_id.into()),
        }
        .arc(),
        FuelCoreReceipt::Return { id, .. } => ReceiptsReturnSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            id: Some(id.into()),
        }
        .arc(),
        FuelCoreReceipt::ReturnData { id, .. } => ReceiptsReturnDataSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            id: Some(id.into()),
        }
        .arc(),
        FuelCoreReceipt::Panic { id, .. } => ReceiptsPanicSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            id: Some(id.into()),
        }
        .arc(),
        FuelCoreReceipt::Revert { id, .. } => ReceiptsRevertSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            id: Some(id.into()),
        }
        .arc(),
        FuelCoreReceipt::Log { id, .. } => ReceiptsLogSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            id: Some(id.into()),
        }
        .arc(),
        FuelCoreReceipt::LogData { id, .. } => ReceiptsLogDataSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            id: Some(id.into()),
        }
        .arc(),
        FuelCoreReceipt::Transfer {
            id: from,
            to,
            asset_id,
            ..
        } => ReceiptsTransferSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            from: Some(from.into()),
            to: Some(to.into()),
            asset_id: Some(asset_id.into()),
        }
        .arc(),

        FuelCoreReceipt::TransferOut {
            id: from,
            to,
            asset_id,
            ..
        } => ReceiptsTransferOutSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            from: Some(from.into()),
            to: Some(to.into()),
            asset_id: Some(asset_id.into()),
        }
        .arc(),

        FuelCoreReceipt::ScriptResult { .. } => ReceiptsScriptResultSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
        }
        .arc(),
        FuelCoreReceipt::MessageOut {
            sender, recipient, ..
        } => ReceiptsMessageOutSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            sender: Some(sender.into()),
            recipient: Some(recipient.into()),
        }
        .arc(),
        FuelCoreReceipt::Mint {
            contract_id,
            sub_id,
            ..
        } => ReceiptsMintSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            contract_id: Some(contract_id.into()),
            sub_id: Some((*sub_id).into()),
        }
        .arc(),
        FuelCoreReceipt::Burn {
            contract_id,
            sub_id,
            ..
        } => ReceiptsBurnSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            contract_id: Some(contract_id.into()),
            sub_id: Some((*sub_id).into()),
        }
        .arc(),
    }
}

pub fn identifiers(
    receipt: &FuelCoreReceipt,
    tx_id: &Bytes32,
    index: u8,
) -> Vec<Identifier> {
    match receipt {
        FuelCoreReceipt::Call {
            id: from,
            to,
            asset_id,
            ..
        } => {
            vec![
                Identifier::ContractID(tx_id.to_owned(), index, from.into()),
                Identifier::ContractID(tx_id.to_owned(), index, to.into()),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
            ]
        }
        FuelCoreReceipt::Return { id, .. }
        | FuelCoreReceipt::ReturnData { id, .. }
        | FuelCoreReceipt::Panic { id, .. }
        | FuelCoreReceipt::Revert { id, .. }
        | FuelCoreReceipt::Log { id, .. }
        | FuelCoreReceipt::LogData { id, .. } => {
            vec![Identifier::ContractID(tx_id.to_owned(), index, id.into())]
        }
        FuelCoreReceipt::Transfer {
            id: from,
            to,
            asset_id,
            ..
        } => {
            vec![
                Identifier::ContractID(tx_id.to_owned(), index, from.into()),
                Identifier::ContractID(tx_id.to_owned(), index, to.into()),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
            ]
        }
        FuelCoreReceipt::TransferOut {
            id: from,
            to,
            asset_id,
            ..
        } => {
            vec![
                Identifier::ContractID(tx_id.to_owned(), index, from.into()),
                Identifier::ContractID(tx_id.to_owned(), index, to.into()),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
            ]
        }
        FuelCoreReceipt::MessageOut {
            sender, recipient, ..
        } => {
            vec![
                Identifier::Address(tx_id.to_owned(), index, sender.into()),
                Identifier::Address(tx_id.to_owned(), index, recipient.into()),
            ]
        }
        FuelCoreReceipt::Mint { contract_id, .. }
        | FuelCoreReceipt::Burn { contract_id, .. } => {
            vec![Identifier::ContractID(
                tx_id.to_owned(),
                index,
                contract_id.into(),
            )]
        }
        _ => Vec::new(),
    }
}
