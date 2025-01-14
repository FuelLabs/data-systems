use std::sync::Arc;

use fuel_streams_core::{subjects::IntoSubject, types::Utxo};
use fuel_streams_domains::{
    transactions::types::MockTransaction,
    utxos::{
        subjects::UtxosSubject,
        types::{MockUtxo, UtxoType},
        UtxoDbItem,
    },
};
use fuel_streams_store::record::RecordPacket;
use fuel_streams_test::{create_random_db_name, setup_store};
use fuel_streams_types::{Address, HexData};

async fn insert_utxo(utxo: Utxo, utxo_type: UtxoType) -> anyhow::Result<()> {
    let tx = MockTransaction::script(vec![], vec![], vec![]);
    let tx_id = tx.id;
    let subject: Arc<dyn IntoSubject> = UtxosSubject {
        block_height: Some(1.into()),
        tx_id: Some(tx_id.clone().into()),
        tx_index: Some(0),
        input_index: Some(0),
        utxo_type: Some(utxo_type),
        utxo_id: Some(HexData::default()),
    }
    .arc();

    let prefix = create_random_db_name();
    let mut store = setup_store::<Utxo>().await?;
    let packet = RecordPacket::new(subject, &utxo).with_namespace(&prefix);
    store.with_namespace(&prefix);

    let db_item = UtxoDbItem::try_from(&packet);
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
async fn store_can_record_coin_utxo() -> anyhow::Result<()> {
    let recipient = Address::default();
    insert_utxo(MockUtxo::coin(100, recipient), UtxoType::Coin).await
}

#[tokio::test]
async fn store_can_record_contract_utxo() -> anyhow::Result<()> {
    let contract_id = Address::default();
    insert_utxo(MockUtxo::contract(contract_id), UtxoType::Contract).await
}

#[tokio::test]
async fn store_can_record_message_utxo() -> anyhow::Result<()> {
    let sender = Address::default();
    let recipient = Address::default();
    insert_utxo(MockUtxo::message(100, sender, recipient), UtxoType::Message)
        .await
}
