use std::sync::Arc;

use fuel_core_types::fuel_tx::Receipt;
use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::{
    identifiers::{Identifier, IdsExtractable, PacketIdBuilder},
    packets::{PublishError, PublishOpts, PublishPacket},
    FuelCoreLike,
};

pub fn publish_tasks(
    tx: &Transaction,
    stream: &Stream<Receipt>,
    opts: &Arc<PublishOpts>,
    fuel_core: &dyn FuelCoreLike,
) -> Vec<JoinHandle<Result<(), PublishError>>> {
    let tx_id = tx.id(&opts.chain_id);
    let receipts = fuel_core.get_receipts(&tx_id).unwrap_or_default();
    let packets: Vec<PublishPacket<Receipt>> = receipts
        .unwrap_or_default()
        .par_iter()
        .enumerate()
        .flat_map(|(index, receipt)| {
            let ids = receipt.extract_ids(&opts.chain_id, tx, index as u8);
            let mut packets = receipt.packets_from_ids(ids);
            let packet = packet_from_receipt(tx_id.into(), receipt, index);
            packets.push(packet);
            packets
        })
        .collect();

    packets
        .iter()
        .map(|packet| {
            packet.publish(Arc::new(stream.to_owned()), Arc::clone(opts))
        })
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
            ReceiptsCallSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_from(Some(from.into()))
                .with_to(Some(to.into()))
                .with_asset_id(Some(asset_id.into()))
                .arc(),
        ),
        Receipt::Return { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsReturnSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .arc(),
        ),
        Receipt::ReturnData { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsReturnDataSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .arc(),
        ),
        Receipt::Panic { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsPanicSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .arc(),
        ),
        Receipt::Revert { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsRevertSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .arc(),
        ),
        Receipt::Log { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsLogSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .arc(),
        ),
        Receipt::LogData { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsLogDataSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .arc(),
        ),
        Receipt::Transfer {
            id: from,
            to,
            asset_id,
            ..
        } => PublishPacket::new(
            receipt,
            ReceiptsTransferSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_from(Some(from.into()))
                .with_to(Some(to.into()))
                .with_asset_id(Some(asset_id.into()))
                .arc(),
        ),
        Receipt::TransferOut {
            id: from,
            to,
            asset_id,
            ..
        } => PublishPacket::new(
            receipt,
            ReceiptsTransferOutSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_from(Some(from.into()))
                .with_to(Some(to.into()))
                .with_asset_id(Some(asset_id.into()))
                .arc(),
        ),
        Receipt::ScriptResult { .. } => PublishPacket::new(
            receipt,
            ReceiptsScriptResultSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .arc(),
        ),
        Receipt::MessageOut {
            sender, recipient, ..
        } => PublishPacket::new(
            receipt,
            ReceiptsMessageOutSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_sender(Some(sender.into()))
                .with_recipient(Some(recipient.into()))
                .arc(),
        ),
        Receipt::Mint {
            contract_id,
            sub_id,
            ..
        } => PublishPacket::new(
            receipt,
            ReceiptsMintSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_contract_id(Some(contract_id.into()))
                .with_sub_id(Some((*sub_id).into()))
                .arc(),
        ),
        Receipt::Burn {
            contract_id,
            sub_id,
            ..
        } => PublishPacket::new(
            receipt,
            ReceiptsBurnSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_contract_id(Some(contract_id.into()))
                .with_sub_id(Some((*sub_id).into()))
                .arc(),
        ),
    }
}

impl IdsExtractable for Receipt {
    fn extract_ids(
        &self,
        chain_id: &ChainId,
        tx: &Transaction,
        index: u8,
    ) -> Vec<Identifier> {
        let tx_id = tx.id(chain_id);
        match self {
            Receipt::Call {
                id: from,
                to,
                asset_id,
                ..
            } => {
                vec![
                    Identifier::ContractID(tx_id.into(), index, from.into()),
                    Identifier::ContractID(tx_id.into(), index, to.into()),
                    Identifier::AssetID(tx_id.into(), index, asset_id.into()),
                ]
            }
            Receipt::Return { id, .. }
            | Receipt::ReturnData { id, .. }
            | Receipt::Panic { id, .. }
            | Receipt::Revert { id, .. }
            | Receipt::Log { id, .. }
            | Receipt::LogData { id, .. } => {
                vec![Identifier::ContractID(tx_id.into(), index, id.into())]
            }
            Receipt::Transfer {
                id: from,
                to,
                asset_id,
                ..
            } => {
                vec![
                    Identifier::ContractID(tx_id.into(), index, from.into()),
                    Identifier::ContractID(tx_id.into(), index, to.into()),
                    Identifier::AssetID(tx_id.into(), index, asset_id.into()),
                ]
            }
            Receipt::TransferOut {
                id: from,
                to,
                asset_id,
                ..
            } => {
                vec![
                    Identifier::ContractID(tx_id.into(), index, from.into()),
                    Identifier::ContractID(tx_id.into(), index, to.into()),
                    Identifier::AssetID(tx_id.into(), index, asset_id.into()),
                ]
            }
            Receipt::MessageOut {
                sender, recipient, ..
            } => {
                vec![
                    Identifier::Address(tx_id.into(), index, sender.into()),
                    Identifier::Address(tx_id.into(), index, recipient.into()),
                ]
            }
            Receipt::Mint { contract_id, .. }
            | Receipt::Burn { contract_id, .. } => {
                vec![Identifier::ContractID(
                    tx_id.into(),
                    index,
                    contract_id.into(),
                )]
            }
            _ => Vec::new(),
        }
    }
}
