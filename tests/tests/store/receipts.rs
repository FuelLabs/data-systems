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
    types::{MockReceipt, MockTransaction, Receipt},
};
use fuel_streams_domains::receipts::ReceiptDbItem;
use fuel_streams_store::record::RecordPacket;
use fuel_streams_test::{create_random_db_name, setup_store};
use fuel_streams_types::Bytes32;

async fn insert_receipt(receipt: Receipt) -> anyhow::Result<()> {
    let tx = MockTransaction::script(vec![], vec![], vec![receipt]);
    let tx_id = Bytes32::from(tx.id.clone());
    let packets = tx
        .receipts
        .into_iter()
        .enumerate()
        .map(|(receipt_index, receipt)| {
            let subject: Arc<dyn IntoSubject> = match &receipt {
                Receipt::Call(data) => ReceiptsCallSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    from_contract_id: Some(data.id.to_owned()),
                    to_contract_id: Some(data.to.to_owned()),
                    asset_id: Some(data.asset_id.to_owned()),
                }
                .arc(),
                Receipt::Return(data) => ReceiptsReturnSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract_id: Some(data.id.to_owned()),
                }
                .arc(),
                Receipt::ReturnData(data) => ReceiptsReturnDataSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract_id: Some(data.id.to_owned()),
                }
                .arc(),
                Receipt::Panic(data) => ReceiptsPanicSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract_id: Some(data.id.to_owned()),
                }
                .arc(),
                Receipt::Revert(data) => ReceiptsRevertSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract_id: Some(data.id.to_owned()),
                }
                .arc(),
                Receipt::Log(data) => ReceiptsLogSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract_id: Some(data.id.to_owned()),
                }
                .arc(),
                Receipt::LogData(data) => ReceiptsLogDataSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract_id: Some(data.id.to_owned()),
                }
                .arc(),
                Receipt::Transfer(data) => ReceiptsTransferSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    from_contract_id: Some(data.id.to_owned()),
                    to_contract_id: Some(data.to.to_owned()),
                    asset_id: Some(data.asset_id.to_owned()),
                }
                .arc(),
                Receipt::TransferOut(data) => ReceiptsTransferOutSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    from_contract_id: Some(data.id.to_owned()),
                    to_address: Some(data.to.to_owned()),
                    asset_id: Some(data.asset_id.to_owned()),
                }
                .arc(),
                Receipt::ScriptResult(_) => ReceiptsScriptResultSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                }
                .arc(),
                Receipt::MessageOut(data) => ReceiptsMessageOutSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    sender_address: Some(data.sender.to_owned()),
                    recipient_address: Some(data.recipient.to_owned()),
                }
                .arc(),
                Receipt::Mint(data) => ReceiptsMintSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract_id: Some(data.contract_id.to_owned()),
                    sub_id: Some(data.sub_id.to_owned()),
                }
                .arc(),
                Receipt::Burn(data) => ReceiptsBurnSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    contract_id: Some(data.contract_id.to_owned()),
                    sub_id: Some(data.sub_id.to_owned()),
                }
                .arc(),
            };
            RecordPacket::new(subject, &receipt)
        })
        .collect::<Vec<_>>();
    assert_eq!(packets.len(), 1);

    let prefix = create_random_db_name();
    let mut store = setup_store::<Receipt>().await?;
    let packet = packets.first().unwrap().clone();
    let packet = packet.with_namespace(&prefix);
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
