use std::sync::Arc;

use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::{publish, PublishOpts};

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
            let identifier_subjects =
                identifier_subjects(receipt, tx_id, index as u8);

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

pub fn identifier_subjects(
    receipt: &FuelCoreReceipt,
    tx_id: &Bytes32,
    index: u8,
) -> Vec<Arc<dyn IntoSubject>> {
    match receipt {
        FuelCoreReceipt::Call {
            id: from,
            to,
            asset_id,
            ..
        } => {
            vec![
                Identifier::ContractID(tx_id.to_owned(), index, from.into())
                    .into(),
                Identifier::ContractID(tx_id.to_owned(), index, to.into())
                    .into(),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into())
                    .into(),
            ]
        }
        FuelCoreReceipt::Return { id, .. }
        | FuelCoreReceipt::ReturnData { id, .. }
        | FuelCoreReceipt::Panic { id, .. }
        | FuelCoreReceipt::Revert { id, .. }
        | FuelCoreReceipt::Log { id, .. }
        | FuelCoreReceipt::LogData { id, .. } => {
            vec![Identifier::ContractID(tx_id.to_owned(), index, id.into())
                .into()]
        }
        FuelCoreReceipt::Transfer {
            id: from,
            to,
            asset_id,
            ..
        } => {
            vec![
                Identifier::ContractID(tx_id.to_owned(), index, from.into())
                    .into(),
                Identifier::ContractID(tx_id.to_owned(), index, to.into())
                    .into(),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into())
                    .into(),
            ]
        }
        FuelCoreReceipt::TransferOut {
            id: from,
            to,
            asset_id,
            ..
        } => {
            vec![
                Identifier::ContractID(tx_id.to_owned(), index, from.into())
                    .into(),
                Identifier::ContractID(tx_id.to_owned(), index, to.into())
                    .into(),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into())
                    .into(),
            ]
        }
        FuelCoreReceipt::MessageOut {
            sender, recipient, ..
        } => {
            vec![
                Identifier::Address(tx_id.to_owned(), index, sender.into())
                    .into(),
                Identifier::Address(tx_id.to_owned(), index, recipient.into())
                    .into(),
            ]
        }
        FuelCoreReceipt::Mint { contract_id, .. }
        | FuelCoreReceipt::Burn { contract_id, .. } => {
            vec![Identifier::ContractID(
                tx_id.to_owned(),
                index,
                contract_id.into(),
            )
            .into()]
        }
        _ => Vec::new(),
    }
}
