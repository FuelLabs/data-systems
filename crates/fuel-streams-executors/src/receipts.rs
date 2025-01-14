use fuel_streams_core::{subjects::*, types::*};
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Receipt> {
    pub fn process(
        &self,
        (tx_index, tx): (usize, &Transaction),
    ) -> Vec<JoinHandle<Result<(), ExecutorError>>> {
        let block_height = self.block_height();
        let tx_id = tx.id.clone();
        let receipts = tx.receipts.clone();
        let packets = receipts
            .par_iter()
            .enumerate()
            .flat_map(|(receipt_index, receipt)| {
                let main_subject = main_subject(
                    block_height.clone(),
                    tx_index as u32,
                    receipt_index as u32,
                    tx_id.clone(),
                    receipt,
                );
                let receipt: Receipt = receipt.to_owned();
                vec![receipt.to_packet(main_subject)]
            })
            .collect::<Vec<_>>();

        packets.iter().map(|packet| self.publish(packet)).collect()
    }
}

fn main_subject(
    block_height: BlockHeight,
    tx_index: u32,
    receipt_index: u32,
    tx_id: TxId,
    receipt: &Receipt,
) -> Arc<dyn IntoSubject> {
    match receipt {
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
            from_contract_id: Some(from.to_owned()),
            to_contract_id: Some(to.to_owned()),
            asset_id: Some(asset_id.to_owned()),
        }
        .arc(),
        Receipt::Return(ReturnReceipt { id, .. }) => ReceiptsReturnSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            receipt_index: Some(receipt_index),
            contract_id: Some(id.to_owned()),
        }
        .arc(),
        Receipt::ReturnData(ReturnDataReceipt { id, .. }) => {
            ReceiptsReturnDataSubject {
                block_height: Some(block_height),
                tx_id: Some(tx_id),
                tx_index: Some(tx_index),
                receipt_index: Some(receipt_index),
                contract_id: Some(id.to_owned()),
            }
            .arc()
        }
        Receipt::Panic(PanicReceipt { id, .. }) => ReceiptsPanicSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            receipt_index: Some(receipt_index),
            contract_id: Some(id.to_owned()),
        }
        .arc(),
        Receipt::Revert(RevertReceipt { id, .. }) => ReceiptsRevertSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            receipt_index: Some(receipt_index),
            contract_id: Some(id.to_owned()),
        }
        .arc(),
        Receipt::Log(LogReceipt { id, .. }) => ReceiptsLogSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            receipt_index: Some(receipt_index),
            contract_id: Some(id.to_owned()),
        }
        .arc(),
        Receipt::LogData(LogDataReceipt { id, .. }) => ReceiptsLogDataSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            receipt_index: Some(receipt_index),
            contract_id: Some(id.to_owned()),
        }
        .arc(),
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
            from_contract_id: Some(from.to_owned()),
            to_contract_id: Some(to.to_owned()),
            asset_id: Some(asset_id.to_owned()),
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
            from_contract_id: Some(from.to_owned()),
            to_address: Some(to.to_owned()),
            asset_id: Some(asset_id.to_owned()),
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
            sender, recipient, ..
        }) => ReceiptsMessageOutSubject {
            block_height: Some(block_height),
            tx_id: Some(tx_id),
            tx_index: Some(tx_index),
            receipt_index: Some(receipt_index),
            sender_address: Some(sender.to_owned()),
            recipient_address: Some(recipient.to_owned()),
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
            contract_id: Some(contract_id.to_owned()),
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
            contract_id: Some(contract_id.to_owned()),
            sub_id: Some((*sub_id).to_owned()),
        }
        .arc(),
    }
}

// pub fn identifiers(
//     receipt: &Receipt,
//     tx_id: &Bytes32,
//     index: u8,
// ) -> Vec<Identifier> {
//     match receipt {
//         Receipt::Call(CallReceipt {
//             id: from,
//             to,
//             asset_id,
//             ..
//         }) => {
//             vec![
//                 Identifier::ContractID(tx_id.to_owned(), index, from.into()),
//                 Identifier::ContractID(tx_id.to_owned(), index, to.into()),
//                 Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
//             ]
//         }
//         Receipt::Return(ReturnReceipt { id, .. })
//         | Receipt::ReturnData(ReturnDataReceipt { id, .. })
//         | Receipt::Panic(PanicReceipt { id, .. })
//         | Receipt::Revert(RevertReceipt { id, .. })
//         | Receipt::Log(LogReceipt { id, .. })
//         | Receipt::LogData(LogDataReceipt { id, .. }) => {
//             vec![Identifier::ContractID(tx_id.to_owned(), index, id.into())]
//         }
//         Receipt::Transfer(TransferReceipt {
//             id: from,
//             to,
//             asset_id,
//             ..
//         }) => {
//             vec![
//                 Identifier::ContractID(tx_id.to_owned(), index, from.into()),
//                 Identifier::ContractID(tx_id.to_owned(), index, to.into()),
//                 Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
//             ]
//         }
//         Receipt::TransferOut(TransferOutReceipt {
//             id: from,
//             to,
//             asset_id,
//             ..
//         }) => {
//             vec![
//                 Identifier::ContractID(tx_id.to_owned(), index, from.into()),
//                 Identifier::ContractID(tx_id.to_owned(), index, to.into()),
//                 Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
//             ]
//         }
//         Receipt::MessageOut(MessageOutReceipt {
//             sender, recipient, ..
//         }) => {
//             vec![
//                 Identifier::Address(tx_id.to_owned(), index, sender.into()),
//                 Identifier::Address(tx_id.to_owned(), index, recipient.into()),
//             ]
//         }
//         Receipt::Mint(MintReceipt { contract_id, .. })
//         | Receipt::Burn(BurnReceipt { contract_id, .. }) => {
//             vec![Identifier::ContractID(
//                 tx_id.to_owned(),
//                 index,
//                 contract_id.into(),
//             )]
//         }
//         _ => Vec::new(),
//     }
// }
