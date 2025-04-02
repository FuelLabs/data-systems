use std::sync::Arc;

use async_trait::async_trait;
use fuel_streams_types::{BlockTimestamp, TxId};
use rayon::prelude::*;

use super::{subjects::*, types::*, ReceiptsQuery};
use crate::{
    blocks::BlockHeight,
    infra::record::{PacketBuilder, RecordPacket, ToPacket},
    transactions::Transaction,
    MsgPayload,
};

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
                let subject = DynReceiptSubject::new(
                    receipt,
                    msg_payload.block_height(),
                    tx_id.clone(),
                    *tx_index as i32,
                    receipt_index as i32,
                );
                let timestamps = msg_payload.timestamp();
                let packet = subject.build_packet(receipt, timestamps);
                match msg_payload.namespace.clone() {
                    Some(ns) => packet.with_namespace(&ns),
                    _ => packet,
                }
            })
            .collect()
    }
}

pub enum DynReceiptSubject {
    Call(ReceiptsCallSubject),
    Return(ReceiptsReturnSubject),
    ReturnData(ReceiptsReturnDataSubject),
    Panic(ReceiptsPanicSubject),
    Revert(ReceiptsRevertSubject),
    Log(ReceiptsLogSubject),
    LogData(ReceiptsLogDataSubject),
    Transfer(ReceiptsTransferSubject),
    TransferOut(ReceiptsTransferOutSubject),
    ScriptResult(ReceiptsScriptResultSubject),
    MessageOut(ReceiptsMessageOutSubject),
    Mint(ReceiptsMintSubject),
    Burn(ReceiptsBurnSubject),
}

impl DynReceiptSubject {
    pub fn new(
        receipt: &Receipt,
        block_height: BlockHeight,
        tx_id: TxId,
        tx_index: i32,
        receipt_index: i32,
    ) -> Self {
        match receipt {
            Receipt::Call(CallReceipt {
                id: from,
                to,
                asset_id,
                ..
            }) => Self::Call(ReceiptsCallSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                from: Some(from.to_owned()),
                to: Some(to.to_owned()),
                asset: Some(asset_id.to_owned()),
            }),
            Receipt::Return(ReturnReceipt { id, .. }) => {
                Self::Return(ReceiptsReturnSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    receipt_index: Some(receipt_index),
                    contract: Some(id.to_owned()),
                })
            }
            Receipt::ReturnData(ReturnDataReceipt { id, .. }) => {
                Self::ReturnData(ReceiptsReturnDataSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    receipt_index: Some(receipt_index),
                    contract: Some(id.to_owned()),
                })
            }
            Receipt::Panic(PanicReceipt { id, .. }) => {
                Self::Panic(ReceiptsPanicSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    receipt_index: Some(receipt_index),
                    contract: Some(id.to_owned()),
                })
            }
            Receipt::Revert(RevertReceipt { id, .. }) => {
                Self::Revert(ReceiptsRevertSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    receipt_index: Some(receipt_index),
                    contract: Some(id.to_owned()),
                })
            }
            Receipt::Log(LogReceipt { id, .. }) => {
                Self::Log(ReceiptsLogSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    receipt_index: Some(receipt_index),
                    contract: Some(id.to_owned()),
                })
            }
            Receipt::LogData(LogDataReceipt { id, .. }) => {
                Self::LogData(ReceiptsLogDataSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    receipt_index: Some(receipt_index),
                    contract: Some(id.to_owned()),
                })
            }
            Receipt::Transfer(TransferReceipt {
                id: from,
                to,
                asset_id,
                ..
            }) => Self::Transfer(ReceiptsTransferSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                from: Some(from.to_owned()),
                to: Some(to.to_owned()),
                asset: Some(asset_id.to_owned()),
            }),
            Receipt::TransferOut(TransferOutReceipt {
                id: from,
                to,
                asset_id,
                ..
            }) => Self::TransferOut(ReceiptsTransferOutSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                from: Some(from.to_owned()),
                to_address: Some(to.to_owned()),
                asset: Some(asset_id.to_owned()),
            }),
            Receipt::ScriptResult(ScriptResultReceipt { .. }) => {
                Self::ScriptResult(ReceiptsScriptResultSubject {
                    block_height: Some(block_height),
                    tx_id: Some(tx_id),
                    tx_index: Some(tx_index),
                    receipt_index: Some(receipt_index),
                })
            }
            Receipt::MessageOut(MessageOutReceipt {
                sender,
                recipient,
                ..
            }) => Self::MessageOut(ReceiptsMessageOutSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                sender: Some(sender.to_owned()),
                recipient: Some(recipient.to_owned()),
            }),
            Receipt::Mint(MintReceipt {
                contract_id,
                sub_id,
                ..
            }) => Self::Mint(ReceiptsMintSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                contract: Some(contract_id.to_owned()),
                sub_id: Some((*sub_id).to_owned()),
            }),
            Receipt::Burn(BurnReceipt {
                contract_id,
                sub_id,
                ..
            }) => Self::Burn(ReceiptsBurnSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                contract: Some(contract_id.to_owned()),
                sub_id: Some((*sub_id).to_owned()),
            }),
        }
    }

    pub fn build_packet(
        &self,
        receipt: &Receipt,
        block_timestamp: BlockTimestamp,
    ) -> RecordPacket {
        match self {
            Self::Call(subject) => {
                receipt.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
            Self::Return(subject) => {
                receipt.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
            Self::ReturnData(subject) => {
                receipt.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
            Self::Panic(subject) => {
                receipt.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
            Self::Revert(subject) => {
                receipt.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
            Self::Log(subject) => {
                receipt.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
            Self::LogData(subject) => {
                receipt.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
            Self::Transfer(subject) => {
                receipt.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
            Self::TransferOut(subject) => {
                receipt.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
            Self::ScriptResult(subject) => {
                receipt.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
            Self::MessageOut(subject) => {
                receipt.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
            Self::Mint(subject) => {
                receipt.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
            Self::Burn(subject) => {
                receipt.to_packet(&Arc::new(subject.clone()), block_timestamp)
            }
        }
    }

    pub fn to_query_params(&self) -> ReceiptsQuery {
        match self {
            Self::Call(subject) => ReceiptsQuery::from(subject.to_owned()),
            Self::Return(subject) => ReceiptsQuery::from(subject.to_owned()),
            Self::ReturnData(subject) => {
                ReceiptsQuery::from(subject.to_owned())
            }
            Self::Panic(subject) => ReceiptsQuery::from(subject.to_owned()),
            Self::Revert(subject) => ReceiptsQuery::from(subject.to_owned()),
            Self::Log(subject) => ReceiptsQuery::from(subject.to_owned()),
            Self::LogData(subject) => ReceiptsQuery::from(subject.to_owned()),
            Self::Transfer(subject) => ReceiptsQuery::from(subject.to_owned()),
            Self::TransferOut(subject) => {
                ReceiptsQuery::from(subject.to_owned())
            }
            Self::ScriptResult(subject) => {
                ReceiptsQuery::from(subject.to_owned())
            }
            Self::MessageOut(subject) => {
                ReceiptsQuery::from(subject.to_owned())
            }
            Self::Mint(subject) => ReceiptsQuery::from(subject.to_owned()),
            Self::Burn(subject) => ReceiptsQuery::from(subject.to_owned()),
        }
    }
}
