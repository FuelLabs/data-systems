use std::sync::Arc;

use fuel_streams_core::{
    subjects::{
        IntoSubject,
        ReceiptsBurnSubject,
        ReceiptsCallSubject,
        ReceiptsLogDataSubject,
        ReceiptsLogSubject,
        ReceiptsMessageOutSubject,
        ReceiptsMintSubject,
        ReceiptsPanicSubject,
        ReceiptsReturnDataSubject,
        ReceiptsReturnSubject,
        ReceiptsRevertSubject,
        ReceiptsScriptResultSubject,
        ReceiptsTransferOutSubject,
        ReceiptsTransferSubject,
    },
    types::{MockReceipt, Receipt, Transaction},
};
use fuel_streams_domains::{
    receipts::ReceiptDbItem,
    transactions::types::MockTransaction,
    Subjects,
};
use fuel_streams_store::{
    record::{QueryOptions, Record, RecordPacket},
    store::Store,
};
use fuel_streams_test::{create_random_db_name, setup_db, setup_store};
use fuel_streams_types::TxId;

async fn insert_receipt(receipt: Receipt) -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let (tx, tx_id) = create_tx(vec![receipt]);
    let packets = create_packets(&tx, &tx_id, &prefix);
    assert_eq!(packets.len(), 1);

    let mut store = setup_store::<Receipt>().await?;
    let packet = packets.first().unwrap().clone();
    store.with_namespace(&prefix);

    let db_item = ReceiptDbItem::try_from(&packet);
    assert!(
        db_item.is_ok(),
        "Failed to convert packet to db item: {:?}",
        packet
    );

    let db_record = store.insert_record(&packet).await?;
    assert_eq!(db_record.subject, packet.subject_str());

    Ok(())
}

fn create_tx(receipts: Vec<Receipt>) -> (Transaction, TxId) {
    let tx = MockTransaction::script(vec![], vec![], receipts);
    let tx_id = tx.to_owned().id;
    (tx, tx_id)
}

fn create_packets(
    tx: &Transaction,
    tx_id: &TxId,
    prefix: &str,
) -> Vec<RecordPacket> {
    tx.clone()
        .receipts
        .into_iter()
        .enumerate()
        .map(|(receipt_index, receipt)| {
            let subject: Arc<dyn IntoSubject> = match &receipt {
                Receipt::Call(data) => ReceiptsCallSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    from: Some(data.id.to_owned()),
                    to: Some(data.to.to_owned()),
                    asset: Some(data.asset_id.to_owned()),
                }
                .arc(),
                Receipt::Return(data) => ReceiptsReturnSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract: Some(data.id.to_owned()),
                }
                .arc(),
                Receipt::ReturnData(data) => ReceiptsReturnDataSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract: Some(data.id.to_owned()),
                }
                .arc(),
                Receipt::Panic(data) => ReceiptsPanicSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract: Some(data.id.to_owned()),
                }
                .arc(),
                Receipt::Revert(data) => ReceiptsRevertSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract: Some(data.id.to_owned()),
                }
                .arc(),
                Receipt::Log(data) => ReceiptsLogSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract: Some(data.id.to_owned()),
                }
                .arc(),
                Receipt::LogData(data) => ReceiptsLogDataSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract: Some(data.id.to_owned()),
                }
                .arc(),
                Receipt::Transfer(data) => ReceiptsTransferSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    from: Some(data.id.to_owned()),
                    to: Some(data.to.to_owned()),
                    asset: Some(data.asset_id.to_owned()),
                }
                .arc(),
                Receipt::TransferOut(data) => ReceiptsTransferOutSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    from: Some(data.id.to_owned()),
                    to_address: Some(data.to.to_owned()),
                    asset: Some(data.asset_id.to_owned()),
                }
                .arc(),
                Receipt::ScriptResult(_) => ReceiptsScriptResultSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                }
                .arc(),
                Receipt::MessageOut(data) => ReceiptsMessageOutSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    sender: Some(data.sender.to_owned()),
                    recipient: Some(data.recipient.to_owned()),
                }
                .arc(),
                Receipt::Mint(data) => ReceiptsMintSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract: Some(data.contract_id.to_owned()),
                    sub_id: Some(data.sub_id.to_owned()),
                }
                .arc(),
                Receipt::Burn(data) => ReceiptsBurnSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract: Some(data.contract_id.to_owned()),
                    sub_id: Some(data.sub_id.to_owned()),
                }
                .arc(),
            };
            receipt.to_packet(&subject).with_namespace(prefix)
        })
        .collect()
}

#[tokio::test]
async fn store_can_record_call_receipt() -> anyhow::Result<()> {
    insert_receipt(MockReceipt::call()).await
}

#[tokio::test]
async fn store_can_record_return_receipt() -> anyhow::Result<()> {
    insert_receipt(MockReceipt::return_receipt()).await
}

#[tokio::test]
async fn store_can_record_return_data_receipt() -> anyhow::Result<()> {
    insert_receipt(MockReceipt::return_data()).await
}

#[tokio::test]
async fn store_can_record_panic_receipt() -> anyhow::Result<()> {
    insert_receipt(MockReceipt::panic()).await
}

#[tokio::test]
async fn store_can_record_revert_receipt() -> anyhow::Result<()> {
    insert_receipt(MockReceipt::revert()).await
}

#[tokio::test]
async fn store_can_record_log_receipt() -> anyhow::Result<()> {
    insert_receipt(MockReceipt::log()).await
}

#[tokio::test]
async fn store_can_record_log_data_receipt() -> anyhow::Result<()> {
    insert_receipt(MockReceipt::log_data()).await
}

#[tokio::test]
async fn store_can_record_transfer_receipt() -> anyhow::Result<()> {
    insert_receipt(MockReceipt::transfer()).await
}

#[tokio::test]
async fn store_can_record_transfer_out_receipt() -> anyhow::Result<()> {
    insert_receipt(MockReceipt::transfer_out()).await
}

#[tokio::test]
async fn store_can_record_script_result_receipt() -> anyhow::Result<()> {
    insert_receipt(MockReceipt::script_result()).await
}

#[tokio::test]
async fn store_can_record_message_out_receipt() -> anyhow::Result<()> {
    insert_receipt(MockReceipt::message_out()).await
}

#[tokio::test]
async fn store_can_record_mint_receipt() -> anyhow::Result<()> {
    insert_receipt(MockReceipt::mint()).await
}

#[tokio::test]
async fn store_can_record_burn_receipt() -> anyhow::Result<()> {
    insert_receipt(MockReceipt::burn()).await
}

#[tokio::test]
async fn find_many_by_subject_with_sql_columns() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let mut store = setup_store::<Receipt>().await?;
    store.with_namespace(&prefix);

    // Create a transaction with all types of receipts
    let receipts = vec![
        MockReceipt::call(),
        MockReceipt::return_receipt(),
        MockReceipt::return_data(),
        MockReceipt::panic(),
        MockReceipt::revert(),
        MockReceipt::log(),
        MockReceipt::log_data(),
        MockReceipt::transfer(),
        MockReceipt::transfer_out(),
        MockReceipt::script_result(),
        MockReceipt::message_out(),
        MockReceipt::mint(),
        MockReceipt::burn(),
    ];
    let (tx, tx_id) = create_tx(receipts);
    let packets = create_packets(&tx, &tx_id, &prefix);

    for packet in packets {
        let _ = store
            .find_many_by_subject(&packet.subject, QueryOptions::default())
            .await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_receipt_subject_to_db_item_conversion() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let db = setup_db().await?;
    let mut store = Store::<Receipt>::new(&db.arc());
    store.with_namespace(&prefix);

    let receipts = vec![
        MockReceipt::call(),
        MockReceipt::return_receipt(),
        MockReceipt::return_data(),
        MockReceipt::panic(),
        MockReceipt::revert(),
        MockReceipt::log(),
        MockReceipt::log_data(),
        MockReceipt::transfer(),
        MockReceipt::transfer_out(),
        MockReceipt::script_result(),
        MockReceipt::message_out(),
        MockReceipt::mint(),
        MockReceipt::burn(),
    ];

    let (tx, tx_id) = create_tx(receipts);
    let packets = create_packets(&tx, &tx_id, &prefix);

    for (idx, packet) in packets.into_iter().enumerate() {
        let subject: Subjects = packet.clone().try_into()?;
        let db_item = ReceiptDbItem::try_from(&packet)?;

        // Add store insert verification
        let inserted = store.insert_record(&packet).await?;
        assert_eq!(db_item, inserted);

        // Verify common fields
        assert_eq!(db_item.block_height, 1);
        assert_eq!(db_item.tx_id, tx_id.to_string());
        assert_eq!(db_item.tx_index, 0);
        assert_eq!(db_item.receipt_index, idx as i32);
        assert_eq!(db_item.subject, packet.subject_str());

        match subject {
            Subjects::ReceiptsCall(subject) => {
                assert_eq!(db_item.receipt_type, "call");
                assert_eq!(
                    db_item.from_contract_id,
                    Some(subject.from.unwrap().to_string())
                );
                assert_eq!(
                    db_item.to_contract_id,
                    Some(subject.to.unwrap().to_string())
                );
                assert_eq!(
                    db_item.asset_id,
                    Some(subject.asset.unwrap().to_string())
                );
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.contract_id, None);
                assert_eq!(db_item.sub_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            Subjects::ReceiptsReturn(subject) => {
                assert_eq!(db_item.receipt_type, "return");
                assert_eq!(
                    db_item.contract_id,
                    Some(subject.contract.unwrap().to_string())
                );
                assert_eq!(db_item.from_contract_id, None);
                assert_eq!(db_item.to_contract_id, None);
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.asset_id, None);
                assert_eq!(db_item.sub_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            Subjects::ReceiptsReturnData(subject) => {
                assert_eq!(db_item.receipt_type, "return_data");
                assert_eq!(
                    db_item.contract_id,
                    Some(subject.contract.unwrap().to_string())
                );
                assert_eq!(db_item.from_contract_id, None);
                assert_eq!(db_item.to_contract_id, None);
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.asset_id, None);
                assert_eq!(db_item.sub_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            Subjects::ReceiptsPanic(subject) => {
                assert_eq!(db_item.receipt_type, "panic");
                assert_eq!(
                    db_item.contract_id,
                    Some(subject.contract.unwrap().to_string())
                );
                assert_eq!(db_item.from_contract_id, None);
                assert_eq!(db_item.to_contract_id, None);
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.asset_id, None);
                assert_eq!(db_item.sub_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            Subjects::ReceiptsRevert(subject) => {
                assert_eq!(db_item.receipt_type, "revert");
                assert_eq!(
                    db_item.contract_id,
                    Some(subject.contract.unwrap().to_string())
                );
                assert_eq!(db_item.from_contract_id, None);
                assert_eq!(db_item.to_contract_id, None);
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.asset_id, None);
                assert_eq!(db_item.sub_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            Subjects::ReceiptsLog(subject) => {
                assert_eq!(db_item.receipt_type, "log");
                assert_eq!(
                    db_item.contract_id,
                    Some(subject.contract.unwrap().to_string())
                );
                assert_eq!(db_item.from_contract_id, None);
                assert_eq!(db_item.to_contract_id, None);
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.asset_id, None);
                assert_eq!(db_item.sub_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            Subjects::ReceiptsLogData(subject) => {
                assert_eq!(db_item.receipt_type, "log_data");
                assert_eq!(
                    db_item.contract_id,
                    Some(subject.contract.unwrap().to_string())
                );
                assert_eq!(db_item.from_contract_id, None);
                assert_eq!(db_item.to_contract_id, None);
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.asset_id, None);
                assert_eq!(db_item.sub_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            Subjects::ReceiptsTransfer(subject) => {
                assert_eq!(db_item.receipt_type, "transfer");
                assert_eq!(
                    db_item.from_contract_id,
                    Some(subject.from.unwrap().to_string())
                );
                assert_eq!(
                    db_item.to_contract_id,
                    Some(subject.to.unwrap().to_string())
                );
                assert_eq!(
                    db_item.asset_id,
                    Some(subject.asset.unwrap().to_string())
                );
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.contract_id, None);
                assert_eq!(db_item.sub_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            Subjects::ReceiptsTransferOut(subject) => {
                assert_eq!(db_item.receipt_type, "transfer_out");
                assert_eq!(
                    db_item.from_contract_id,
                    Some(subject.from.unwrap().to_string())
                );
                assert_eq!(
                    db_item.to_address,
                    Some(subject.to_address.unwrap().to_string())
                );
                assert_eq!(
                    db_item.asset_id,
                    Some(subject.asset.unwrap().to_string())
                );
                assert_eq!(db_item.to_contract_id, None);
                assert_eq!(db_item.contract_id, None);
                assert_eq!(db_item.sub_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            Subjects::ReceiptsScriptResult(_) => {
                assert_eq!(db_item.receipt_type, "script_result");
                assert_eq!(db_item.from_contract_id, None);
                assert_eq!(db_item.to_contract_id, None);
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.asset_id, None);
                assert_eq!(db_item.contract_id, None);
                assert_eq!(db_item.sub_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            Subjects::ReceiptsMessageOut(subject) => {
                assert_eq!(db_item.receipt_type, "message_out");
                assert_eq!(
                    db_item.sender_address,
                    Some(subject.sender.unwrap().to_string())
                );
                assert_eq!(
                    db_item.recipient_address,
                    Some(subject.recipient.unwrap().to_string())
                );
                assert_eq!(db_item.from_contract_id, None);
                assert_eq!(db_item.to_contract_id, None);
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.asset_id, None);
                assert_eq!(db_item.contract_id, None);
                assert_eq!(db_item.sub_id, None);
            }
            Subjects::ReceiptsMint(subject) => {
                assert_eq!(db_item.receipt_type, "mint");
                assert_eq!(
                    db_item.contract_id,
                    Some(subject.contract.unwrap().to_string())
                );
                assert_eq!(
                    db_item.sub_id,
                    Some(subject.sub_id.unwrap().to_string())
                );
                assert_eq!(db_item.from_contract_id, None);
                assert_eq!(db_item.to_contract_id, None);
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.asset_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            Subjects::ReceiptsBurn(subject) => {
                assert_eq!(db_item.receipt_type, "burn");
                assert_eq!(
                    db_item.contract_id,
                    Some(subject.contract.unwrap().to_string())
                );
                assert_eq!(
                    db_item.sub_id,
                    Some(subject.sub_id.unwrap().to_string())
                );
                assert_eq!(db_item.from_contract_id, None);
                assert_eq!(db_item.to_contract_id, None);
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.asset_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            _ => panic!("Unexpected subject type"),
        }
    }

    Ok(())
}
