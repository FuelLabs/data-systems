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
            let ids = receipt.extract_ids(Some(tx));
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
            ReceiptsCallSubject::WILDCARD,
        ),
        Receipt::Return { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsReturnSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .arc(),
            ReceiptsReturnSubject::WILDCARD,
        ),
        Receipt::ReturnData { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsReturnDataSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .arc(),
            ReceiptsReturnDataSubject::WILDCARD,
        ),
        Receipt::Panic { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsPanicSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .arc(),
            ReceiptsPanicSubject::WILDCARD,
        ),
        Receipt::Revert { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsRevertSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .arc(),
            ReceiptsRevertSubject::WILDCARD,
        ),
        Receipt::Log { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsLogSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
                .arc(),
            ReceiptsLogSubject::WILDCARD,
        ),
        Receipt::LogData { id, .. } => PublishPacket::new(
            receipt,
            ReceiptsLogDataSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_id(Some(id.into()))
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
            ReceiptsTransferSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_from(Some(from.into()))
                .with_to(Some(to.into()))
                .with_asset_id(Some(asset_id.into()))
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
            ReceiptsTransferOutSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .with_from(Some(from.into()))
                .with_to(Some(to.into()))
                .with_asset_id(Some(asset_id.into()))
                .arc(),
            ReceiptsTransferOutSubject::WILDCARD,
        ),
        Receipt::ScriptResult { .. } => PublishPacket::new(
            receipt,
            ReceiptsScriptResultSubject::new()
                .with_tx_id(Some(tx_id))
                .with_index(Some(index))
                .arc(),
            ReceiptsScriptResultSubject::WILDCARD,
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
            ReceiptsMessageOutSubject::WILDCARD,
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
            ReceiptsMintSubject::WILDCARD,
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
            ReceiptsBurnSubject::WILDCARD,
        ),
    }
}

impl IdsExtractable for Receipt {
    fn extract_ids(&self, _tx: Option<&Transaction>) -> Vec<Identifier> {
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
