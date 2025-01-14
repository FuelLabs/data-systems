use fuel_core::service::Config;
use fuel_core_bin::FuelService;
use fuel_core_client::client::FuelClient;
use fuel_streams_core::{subjects::ReceiptsCallSubject, types::Receipt};
use fuel_streams_domains::receipts::ReceiptDbItem;
use fuel_streams_store::record::RecordPacket;
use fuel_streams_test::{create_random_db_name, setup_store};
use fuel_streams_types::{
    Bytes32,
    FuelCoreChainId,
    FuelCoreReceipt,
    FuelCoreTransaction,
    FuelCoreUniqueIdentifier,
};

#[tokio::test]
async fn can_record_receipts() -> anyhow::Result<()> {
    let config = Config::local_node();
    let srv = FuelService::new_node(config).await?;
    let client = FuelClient::from(srv.bound_address);
    let tx = FuelCoreTransaction::default_test_tx();
    let tx_id = Bytes32::from(tx.id(&FuelCoreChainId::default()));
    client.submit_and_await_commit(&tx).await?;

    let client_receipts = client.all_receipts().await?;
    assert!(!client_receipts.is_empty());

    let packets = client_receipts
        .into_iter()
        .enumerate()
        .filter_map(|(receipt_index, receipt)| match receipt {
            FuelCoreReceipt::Call { .. } => {
                let receipt = Receipt::from(receipt);
                let call_receipt = receipt.as_call();
                let subject = ReceiptsCallSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    receipt_index: Some(receipt_index as u32),
                    from_contract_id: Some(call_receipt.id),
                    to_contract_id: Some(call_receipt.to),
                    asset_id: Some(call_receipt.asset_id),
                }
                .arc();
                Some(RecordPacket::new(subject, &receipt))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(packets.len(), 1);

    let packet = packets.first().unwrap();
    let db_item = ReceiptDbItem::try_from(packet);
    assert!(db_item.is_ok());

    let store = setup_store::<Receipt>().await?;
    let prefix = create_random_db_name();
    let packet = packet.clone().with_namespace(&prefix);
    let db_record = store.insert_record(&packet).await?;
    assert_eq!(db_record.subject, packet.subject_str());

    Ok(())
}
