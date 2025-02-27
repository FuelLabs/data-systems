use fuel_streams_core::types::{MockReceipt, Receipt, Transaction};
use fuel_streams_domains::{
    receipts::{DynReceiptSubject, ReceiptDbItem},
    transactions::types::MockTransaction,
    MockMsgPayload,
    Subjects,
};
use fuel_streams_store::{
    record::{QueryOptions, Record, RecordPacket},
    store::Store,
};
use fuel_streams_test::{
    close_db,
    create_random_db_name,
    setup_db,
    setup_store,
};
use fuel_streams_types::TxId;
use pretty_assertions::assert_eq;

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

    let db_item = db_item.unwrap();
    let db_record = store.insert_record(&db_item).await?;
    assert_eq!(db_record.subject, packet.subject_str());

    close_db(&store.db).await;
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
            let subject = DynReceiptSubject::from((
                &receipt,
                1.into(),
                tx_id.clone(),
                0,
                receipt_index as u32,
            ));
            let msg_payload = MockMsgPayload::build(1, prefix);
            receipt
                .to_packet(&subject.into(), msg_payload.block_timestamp)
                .with_namespace(prefix)
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
        let payload = packet.subject_payload.clone();
        let subject: Subjects = payload.try_into()?;
        let subject = subject.into();
        let _ = store
            .find_many_by_subject(&subject, QueryOptions::default())
            .await?;
    }

    close_db(&store.db).await;
    Ok(())
}

#[tokio::test]
async fn test_receipt_subject_to_db_item_conversion() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let db = setup_db().await?;
    let mut store = Store::<Receipt>::new(&db);
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

    for packet in packets {
        let payload = packet.subject_payload.clone();
        let subject: Subjects = payload.try_into()?;
        let db_item = ReceiptDbItem::try_from(&packet)?;
        let inserted = store.insert_record(&db_item).await?;

        // Verify common fields
        assert_eq!(db_item.block_height, inserted.block_height);
        assert_eq!(db_item.tx_id, inserted.tx_id);
        assert_eq!(db_item.tx_index, inserted.tx_index);
        assert_eq!(db_item.receipt_index, inserted.receipt_index);
        assert_eq!(db_item.subject, inserted.subject);
        assert_eq!(db_item.value, inserted.value);
        assert_eq!(db_item.created_at, inserted.created_at);
        assert!(inserted.published_at.is_after(&db_item.published_at));

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

    close_db(&store.db).await;
    Ok(())
}
