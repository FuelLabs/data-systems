use std::sync::Arc;

use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Receipt> {
    pub fn process(
        &self,
        tx: &Transaction,
    ) -> Vec<JoinHandle<Result<(), ExecutorError>>> {
        let tx_id = tx.id.clone();
        let receipts = tx.receipts.clone();
        let packets: Vec<PublishPacket<Receipt>> = receipts
            .par_iter()
            .enumerate()
            .flat_map(|(index, receipt)| {
                let main_subject = main_subject(receipt, &tx_id, index);
                let identifier_subjects =
                    identifiers(receipt, &tx_id, index as u8)
                        .into_par_iter()
                        .map(|identifier| identifier.into())
                        .map(|subject: ReceiptsByIdSubject| subject.arc())
                        .collect::<Vec<_>>();

                let receipt: Receipt = receipt.to_owned();
                let mut packets = vec![receipt.to_packet(main_subject)];
                packets.extend(
                    identifier_subjects
                        .into_iter()
                        .map(|subject| receipt.to_packet(subject)),
                );

                packets
            })
            .collect();

        packets.iter().map(|packet| self.publish(packet)).collect()
    }
}

fn main_subject(
    receipt: &Receipt,
    tx_id: &Bytes32,
    index: usize,
) -> Arc<dyn IntoSubject> {
    match receipt {
        Receipt::Call(CallReceipt {
            id: from,
            to,
            asset_id,
            ..
        }) => ReceiptsCallSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            from: Some(from.to_owned()),
            to: Some(to.to_owned()),
            asset_id: Some(asset_id.to_owned()),
        }
        .arc(),
        Receipt::Return(ReturnReceipt { id, .. }) => ReceiptsReturnSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            id: Some(id.to_owned()),
        }
        .arc(),
        Receipt::ReturnData(ReturnDataReceipt { id, .. }) => {
            ReceiptsReturnDataSubject {
                tx_id: Some(tx_id.to_owned()),
                index: Some(index),
                id: Some(id.to_owned()),
            }
            .arc()
        }
        Receipt::Panic(PanicReceipt { id, .. }) => ReceiptsPanicSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            id: Some(id.to_owned()),
        }
        .arc(),
        Receipt::Revert(RevertReceipt { id, .. }) => ReceiptsRevertSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            id: Some(id.to_owned()),
        }
        .arc(),
        Receipt::Log(LogReceipt { id, .. }) => ReceiptsLogSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            id: Some(id.to_owned()),
        }
        .arc(),
        Receipt::LogData(LogDataReceipt { id, .. }) => ReceiptsLogDataSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            id: Some(id.to_owned()),
        }
        .arc(),
        Receipt::Transfer(TransferReceipt {
            id: from,
            to,
            asset_id,
            ..
        }) => ReceiptsTransferSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            from: Some(from.to_owned()),
            to: Some(to.to_owned()),
            asset_id: Some(asset_id.to_owned()),
        }
        .arc(),

        Receipt::TransferOut(TransferOutReceipt {
            id: from,
            to,
            asset_id,
            ..
        }) => ReceiptsTransferOutSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            from: Some(from.to_owned()),
            to: Some(to.to_owned()),
            asset_id: Some(asset_id.to_owned()),
        }
        .arc(),

        Receipt::ScriptResult(ScriptResultReceipt { .. }) => {
            ReceiptsScriptResultSubject {
                tx_id: Some(tx_id.to_owned()),
                index: Some(index),
            }
            .arc()
        }
        Receipt::MessageOut(MessageOutReceipt {
            sender, recipient, ..
        }) => ReceiptsMessageOutSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            sender: Some(sender.to_owned()),
            recipient: Some(recipient.to_owned()),
        }
        .arc(),
        Receipt::Mint(MintReceipt {
            contract_id,
            sub_id,
            ..
        }) => ReceiptsMintSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            contract_id: Some(contract_id.to_owned()),
            sub_id: Some((*sub_id).to_owned()),
        }
        .arc(),
        Receipt::Burn(BurnReceipt {
            contract_id,
            sub_id,
            ..
        }) => ReceiptsBurnSubject {
            tx_id: Some(tx_id.to_owned()),
            index: Some(index),
            contract_id: Some(contract_id.to_owned()),
            sub_id: Some((*sub_id).to_owned()),
        }
        .arc(),
    }
}

pub fn identifiers(
    receipt: &Receipt,
    tx_id: &Bytes32,
    index: u8,
) -> Vec<Identifier> {
    match receipt {
        Receipt::Call(CallReceipt {
            id: from,
            to,
            asset_id,
            ..
        }) => {
            vec![
                Identifier::ContractID(tx_id.to_owned(), index, from.into()),
                Identifier::ContractID(tx_id.to_owned(), index, to.into()),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
            ]
        }
        Receipt::Return(ReturnReceipt { id, .. })
        | Receipt::ReturnData(ReturnDataReceipt { id, .. })
        | Receipt::Panic(PanicReceipt { id, .. })
        | Receipt::Revert(RevertReceipt { id, .. })
        | Receipt::Log(LogReceipt { id, .. })
        | Receipt::LogData(LogDataReceipt { id, .. }) => {
            vec![Identifier::ContractID(tx_id.to_owned(), index, id.into())]
        }
        Receipt::Transfer(TransferReceipt {
            id: from,
            to,
            asset_id,
            ..
        }) => {
            vec![
                Identifier::ContractID(tx_id.to_owned(), index, from.into()),
                Identifier::ContractID(tx_id.to_owned(), index, to.into()),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
            ]
        }
        Receipt::TransferOut(TransferOutReceipt {
            id: from,
            to,
            asset_id,
            ..
        }) => {
            vec![
                Identifier::ContractID(tx_id.to_owned(), index, from.into()),
                Identifier::ContractID(tx_id.to_owned(), index, to.into()),
                Identifier::AssetID(tx_id.to_owned(), index, asset_id.into()),
            ]
        }
        Receipt::MessageOut(MessageOutReceipt {
            sender, recipient, ..
        }) => {
            vec![
                Identifier::Address(tx_id.to_owned(), index, sender.into()),
                Identifier::Address(tx_id.to_owned(), index, recipient.into()),
            ]
        }
        Receipt::Mint(MintReceipt { contract_id, .. })
        | Receipt::Burn(BurnReceipt { contract_id, .. }) => {
            vec![Identifier::ContractID(
                tx_id.to_owned(),
                index,
                contract_id.into(),
            )]
        }
        _ => Vec::new(),
    }
}
