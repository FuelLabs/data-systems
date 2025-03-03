use std::sync::Arc;

use async_trait::async_trait;
use fuel_streams_store::record::{PacketBuilder, Record, RecordPacket};
use fuel_streams_subject::subject::IntoSubject;
use fuel_streams_types::TxId;
use rayon::prelude::*;

use super::{subjects::*, types::*};
use crate::{blocks::BlockHeight, transactions::Transaction, MsgPayload};

#[async_trait]
impl PacketBuilder for Receipt {
    type Opts = (MsgPayload, usize, Transaction);
    fn build_packets(
        (msg_payload, tx_index, tx): &Self::Opts,
    ) -> Vec<RecordPacket> {
        let tx_id = tx.id.clone();
        let receipts = tx.receipts.clone();
        receipts
            .par_iter()
            .enumerate()
            .map(|(receipt_index, receipt)| {
                let subject = DynReceiptSubject::from((
                    receipt,
                    msg_payload.block_height(),
                    tx_id.clone(),
                    *tx_index as u32,
                    receipt_index as u32,
                ));
                let timestamps = msg_payload.timestamp();
                let packet = receipt.to_packet(&subject.into(), timestamps);
                match msg_payload.namespace.clone() {
                    Some(ns) => packet.with_namespace(&ns),
                    _ => packet,
                }
            })
            .collect()
    }
}

pub struct DynReceiptSubject(Arc<dyn IntoSubject>);
impl From<(&Receipt, BlockHeight, TxId, u32, u32)> for DynReceiptSubject {
    fn from(
        (receipt, block_height, tx_id, tx_index, receipt_index): (
            &Receipt,
            BlockHeight,
            TxId,
            u32,
            u32,
        ),
    ) -> Self {
        DynReceiptSubject(match receipt {
            Receipt::Call(CallReceipt {
                id: from,
                to,
                asset_id,
                ..
            }) => ReceiptsCallSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                from: Some(from.to_owned()),
                to: Some(to.to_owned()),
                asset: Some(asset_id.to_owned()),
            }
            .arc(),
            Receipt::Return(ReturnReceipt { id, .. }) => {
                ReceiptsReturnSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    receipt_index: Some(receipt_index),
                    contract: Some(id.to_owned()),
                }
                .arc()
            }
            Receipt::ReturnData(ReturnDataReceipt { id, .. }) => {
                ReceiptsReturnDataSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    receipt_index: Some(receipt_index),
                    contract: Some(id.to_owned()),
                }
                .arc()
            }
            Receipt::Panic(PanicReceipt { id, .. }) => ReceiptsPanicSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                contract: Some(id.to_owned()),
            }
            .arc(),
            Receipt::Revert(RevertReceipt { id, .. }) => {
                ReceiptsRevertSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    receipt_index: Some(receipt_index),
                    contract: Some(id.to_owned()),
                }
                .arc()
            }
            Receipt::Log(LogReceipt { id, .. }) => ReceiptsLogSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                contract: Some(id.to_owned()),
            }
            .arc(),
            Receipt::LogData(LogDataReceipt { id, .. }) => {
                ReceiptsLogDataSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    receipt_index: Some(receipt_index),
                    contract: Some(id.to_owned()),
                }
                .arc()
            }
            Receipt::Transfer(TransferReceipt {
                id: from,
                to,
                asset_id,
                ..
            }) => ReceiptsTransferSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                from: Some(from.to_owned()),
                to: Some(to.to_owned()),
                asset: Some(asset_id.to_owned()),
            }
            .arc(),
            Receipt::TransferOut(TransferOutReceipt {
                id: from,
                to,
                asset_id,
                ..
            }) => ReceiptsTransferOutSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                from: Some(from.to_owned()),
                to_address: Some(to.to_owned()),
                asset: Some(asset_id.to_owned()),
            }
            .arc(),
            Receipt::ScriptResult(ScriptResultReceipt { .. }) => {
                ReceiptsScriptResultSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    receipt_index: Some(receipt_index),
                }
                .arc()
            }
            Receipt::MessageOut(MessageOutReceipt {
                sender,
                recipient,
                ..
            }) => ReceiptsMessageOutSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                sender: Some(sender.to_owned()),
                recipient: Some(recipient.to_owned()),
            }
            .arc(),
            Receipt::Mint(MintReceipt {
                contract_id,
                sub_id,
                ..
            }) => ReceiptsMintSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                contract: Some(contract_id.to_owned()),
                sub_id: Some((*sub_id).to_owned()),
            }
            .arc(),
            Receipt::Burn(BurnReceipt {
                contract_id,
                sub_id,
                ..
            }) => ReceiptsBurnSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                contract: Some(contract_id.to_owned()),
                sub_id: Some((*sub_id).to_owned()),
            }
            .arc(),
        })
    }
}

impl From<DynReceiptSubject> for Arc<dyn IntoSubject> {
    fn from(subject: DynReceiptSubject) -> Self {
        subject.0
    }
}
