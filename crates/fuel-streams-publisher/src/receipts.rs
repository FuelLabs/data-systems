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
            ReceiptsCallSubject::build(
                Some(tx_id),
                Some(index),
                Some(from.into()),
                Some(to.into()),
                Some(asset_id.into()),
            )
            .arc(),
            ReceiptsCallSubject::WILDCARD,
        ),
        Receipt::Return { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsReturnSubject::build(
                Some(tx_id),
                Some(index),
                Some(id.into()),
            )
            .arc(),
            ReceiptsReturnSubject::WILDCARD,
        ),
        Receipt::ReturnData { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsReturnDataSubject::build(
                Some(tx_id),
                Some(index),
                Some(id.into()),
            )
            .arc(),
            ReceiptsReturnDataSubject::WILDCARD,
        ),
        Receipt::Panic { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsPanicSubject::build(
                Some(tx_id),
                Some(index),
                Some(id.into()),
            )
            .arc(),
            ReceiptsPanicSubject::WILDCARD,
        ),
        Receipt::Revert { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsRevertSubject::build(
                Some(tx_id),
                Some(index),
                Some(id.into()),
            )
            .arc(),
            ReceiptsRevertSubject::WILDCARD,
        ),
        Receipt::Log { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsLogSubject::build(
                Some(tx_id),
                Some(index),
                Some(id.into()),
            )
            .arc(),
            ReceiptsLogSubject::WILDCARD,
        ),
        Receipt::LogData { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsLogDataSubject::build(
                Some(tx_id),
                Some(index),
                Some(id.into()),
            )
            .arc(),
            ReceiptsLogDataSubject::WILDCARD,
        ),
        Receipt::Transfer {
            id: from,
            to,
            asset_id,
            ..
        } => PublishPacket::new(
            receipt,
            ReceiptsTransferSubject::build(
                Some(tx_id),
                Some(index),
                Some(from.into()),
                Some(to.into()),
                Some(asset_id.into()),
            )
            .arc(),
            ReceiptsTransferSubject::WILDCARD,
        ),
        Receipt::TransferOut {
            id: from,
            to,
            asset_id,
            ..
        } => PublishPacket::new(
            receipt,
            ReceiptsTransferOutSubject::build(
                Some(tx_id),
                Some(index),
                Some(from.into()),
                Some(to.into()),
                Some(asset_id.into()),
            )
            .arc(),
            ReceiptsTransferOutSubject::WILDCARD,
        ),
        Receipt::ScriptResult { .. } => PublishPacket::new(
            receipt,
            ReceiptsScriptResultSubject::build(Some(tx_id), Some(index)).arc(),
            ReceiptsScriptResultSubject::WILDCARD,
        ),
        Receipt::MessageOut {
            sender, recipient, ..
        } => PublishPacket::new(
            receipt,
            ReceiptsMessageOutSubject::build(
                Some(tx_id),
                Some(index),
                Some(sender.into()),
                Some(recipient.into()),
            )
            .arc(),
            ReceiptsMessageOutSubject::WILDCARD,
        ),
        Receipt::Mint {
            contract_id,
            sub_id,
            ..
        } => PublishPacket::new(
            receipt,
            ReceiptsMintSubject::build(
                Some(tx_id),
                Some(index),
                Some(contract_id.into()),
                Some((*sub_id).into()),
            )
            .arc(),
            ReceiptsMintSubject::WILDCARD,
        ),
        Receipt::Burn {
            contract_id,
            sub_id,
            ..
        } => PublishPacket::new(
            receipt,
            ReceiptsBurnSubject::build(
                Some(tx_id),
                Some(index),
                Some(contract_id.into()),
                Some((*sub_id).into()),
            )
            .arc(),
            ReceiptsBurnSubject::WILDCARD,
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
