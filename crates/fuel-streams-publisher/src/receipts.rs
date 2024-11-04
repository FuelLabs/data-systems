use std::sync::Arc;

use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::{
    identifiers::{Identifier, IdsExtractable, PacketIdBuilder},
    packets::{PublishError, PublishOpts, PublishPacket},
};

pub fn publish_tasks(
    tx: &Transaction,
    tx_id: &Bytes32,
    stream: &Stream<Receipt>,
    opts: &Arc<PublishOpts>,
    receipts: &Vec<Receipt>,
) -> Vec<JoinHandle<Result<(), PublishError>>> {
    let packets: Vec<PublishPacket<Receipt>> = receipts
        .par_iter()
        .enumerate()
        .flat_map(|(index, receipt)| {
            let ids = receipt.extract_ids(tx, tx_id, index as u8);
            let mut packets = receipt.packets_from_ids(ids);
            let packet = packet_from_receipt(tx_id.to_owned(), receipt, index);
            packets.push(packet);
            packets
        })
        .collect();

    packets
        .iter()
        .map(|packet| packet.publish(Arc::new(stream.to_owned()), opts))
        .collect()
}

fn packet_from_receipt(
    tx_id: Bytes32,
    receipt: &Receipt,
    index: usize,
) -> PublishPacket<Receipt> {
    match receipt {
        Receipt::Call {
            id: from,
            to,
            asset_id,
            ..
        } => PublishPacket::new(
            receipt,
            ReceiptsCallSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                from: Some(from.into()),
                to: Some(to.into()),
                asset_id: Some(asset_id.into()),
            }
            .arc(),
        ),
        Receipt::Return { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsReturnSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                id: Some(id.into()),
            }
            .arc(),
        ),
        Receipt::ReturnData { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsReturnDataSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                id: Some(id.into()),
            }
            .arc(),
        ),
        Receipt::Panic { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsPanicSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                id: Some(id.into()),
            }
            .arc(),
        ),
        Receipt::Revert { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsRevertSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                id: Some(id.into()),
            }
            .arc(),
        ),
        Receipt::Log { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsLogSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                id: Some(id.into()),
            }
            .arc(),
        ),
        Receipt::LogData { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsLogDataSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                id: Some(id.into()),
            }
            .arc(),
        ),
        Receipt::Transfer {
            id: from,
            to,
            asset_id,
            ..
        } => PublishPacket::new(
            receipt,
            ReceiptsTransferSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                from: Some(from.into()),
                to: Some(to.into()),
                asset_id: Some(asset_id.into()),
            }
            .arc(),
        ),
        Receipt::TransferOut {
            id: from,
            to,
            asset_id,
            ..
        } => PublishPacket::new(
            receipt,
            ReceiptsTransferOutSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                from: Some(from.into()),
                to: Some(to.into()),
                asset_id: Some(asset_id.into()),
            }
            .arc(),
        ),
        Receipt::ScriptResult { .. } => PublishPacket::new(
            receipt,
            ReceiptsScriptResultSubject {
                tx_id: Some(tx_id),
                index: Some(index),
            }
            .arc(),
        ),
        Receipt::MessageOut {
            sender, recipient, ..
        } => PublishPacket::new(
            receipt,
            ReceiptsMessageOutSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                sender: Some(sender.into()),
                recipient: Some(recipient.into()),
            }
            .arc(),
        ),
        Receipt::Mint {
            contract_id,
            sub_id,
            ..
        } => PublishPacket::new(
            receipt,
            ReceiptsMintSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                contract_id: Some(contract_id.into()),
                sub_id: Some((*sub_id).into()),
            }
            .arc(),
        ),
        Receipt::Burn {
            contract_id,
            sub_id,
            ..
        } => PublishPacket::new(
            receipt,
            ReceiptsBurnSubject {
                tx_id: Some(tx_id),
                index: Some(index),
                contract_id: Some(contract_id.into()),
                sub_id: Some((*sub_id).into()),
            }
            .arc(),
        ),
    }
}

impl IdsExtractable for Receipt {
    fn extract_ids(
        &self,
        _tx: &Transaction,
        tx_id: &Bytes32,
        index: u8,
    ) -> Vec<Identifier> {
        match self {
            Receipt::Call {
                id: from,
                to,
                asset_id,
                ..
            } => {
                vec![
                    Identifier::ContractID(
                        tx_id.to_owned(),
                        index,
                        from.into(),
                    ),
                    Identifier::ContractID(tx_id.to_owned(), index, to.into()),
                    Identifier::AssetID(
                        tx_id.to_owned(),
                        index,
                        asset_id.into(),
                    ),
                ]
            }
            Receipt::Return { id, .. }
            | Receipt::ReturnData { id, .. }
            | Receipt::Panic { id, .. }
            | Receipt::Revert { id, .. }
            | Receipt::Log { id, .. }
            | Receipt::LogData { id, .. } => {
                vec![Identifier::ContractID(tx_id.to_owned(), index, id.into())]
            }
            Receipt::Transfer {
                id: from,
                to,
                asset_id,
                ..
            } => {
                vec![
                    Identifier::ContractID(
                        tx_id.to_owned(),
                        index,
                        from.into(),
                    ),
                    Identifier::ContractID(tx_id.to_owned(), index, to.into()),
                    Identifier::AssetID(
                        tx_id.to_owned(),
                        index,
                        asset_id.into(),
                    ),
                ]
            }
            Receipt::TransferOut {
                id: from,
                to,
                asset_id,
                ..
            } => {
                vec![
                    Identifier::ContractID(
                        tx_id.to_owned(),
                        index,
                        from.into(),
                    ),
                    Identifier::ContractID(tx_id.to_owned(), index, to.into()),
                    Identifier::AssetID(
                        tx_id.to_owned(),
                        index,
                        asset_id.into(),
                    ),
                ]
            }
            Receipt::MessageOut {
                sender, recipient, ..
            } => {
                vec![
                    Identifier::Address(tx_id.to_owned(), index, sender.into()),
                    Identifier::Address(
                        tx_id.to_owned(),
                        index,
                        recipient.into(),
                    ),
                ]
            }
            Receipt::Mint { contract_id, .. }
            | Receipt::Burn { contract_id, .. } => {
                vec![Identifier::ContractID(
                    tx_id.to_owned(),
                    index,
                    contract_id.into(),
                )]
            }
            _ => Vec::new(),
        }
    }
}
