use std::sync::Arc;

use fuel_streams_core::{
    subjects::IntoSubject,
    types::{Transaction, Utxo},
};
use fuel_streams_domains::{
    transactions::types::MockTransaction,
    utxos::{
        subjects::UtxosSubject,
        types::{MockUtxo, UtxoType},
        UtxoDbItem,
    },
    Subjects,
};
use fuel_streams_store::{
    record::{QueryOptions, Record, RecordPacket},
    store::Store,
};
use fuel_streams_test::{create_random_db_name, setup_db, setup_store};
use fuel_streams_types::{Address, HexData, TxId};

async fn insert_utxo(utxo: Utxo, utxo_type: UtxoType) -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let (_, tx_id) = create_tx();
    let packets = create_packets(&tx_id, utxo, utxo_type, &prefix);
    assert_eq!(packets.len(), 1);

    let mut store = setup_store::<Utxo>().await?;
    let packet = packets.first().unwrap().clone();
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

fn create_tx() -> (Transaction, TxId) {
    let tx = MockTransaction::script(vec![], vec![], vec![]);
    let tx_id = tx.to_owned().id;
    (tx, tx_id)
}

fn create_packets(
    tx_id: &TxId,
    utxo: Utxo,
    utxo_type: UtxoType,
    prefix: &str,
) -> Vec<RecordPacket> {
    let subject: Arc<dyn IntoSubject> = UtxosSubject {
        block_height: Some(1.into()),
        tx_id: Some(tx_id.clone()),
        tx_index: Some(0),
        input_index: Some(0),
        utxo_type: Some(utxo_type),
        utxo_id: Some(HexData::default()),
    }
    .dyn_arc();

    vec![utxo.to_packet(&subject).with_namespace(prefix)]
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

#[tokio::test]
async fn find_many_by_subject_with_sql_columns() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let mut store = setup_store::<Utxo>().await?;
    store.with_namespace(&prefix);

    let (_, tx_id) = create_tx();
    let utxos = vec![
        (MockUtxo::coin(100, Address::default()), UtxoType::Coin),
        (MockUtxo::contract(Address::default()), UtxoType::Contract),
        (
            MockUtxo::message(100, Address::default(), Address::default()),
            UtxoType::Message,
        ),
    ];

    for (utxo, utxo_type) in utxos {
        let packets = create_packets(&tx_id, utxo, utxo_type, &prefix);
        for packet in packets {
            let _ = store
                .find_many_by_subject(&packet.subject, QueryOptions::default())
                .await?;
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_utxo_subject_to_db_item_conversion() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let db = setup_db().await?;
    let mut store = Store::<Utxo>::new(&db.arc());
    store.with_namespace(&prefix);

    let utxos = vec![
        (MockUtxo::coin(100, Address::default()), UtxoType::Coin),
        (MockUtxo::contract(Address::default()), UtxoType::Contract),
        (
            MockUtxo::message(100, Address::default(), Address::default()),
            UtxoType::Message,
        ),
    ];

    for (utxo, utxo_type) in utxos {
        let (_, tx_id) = create_tx();
        let packets = create_packets(&tx_id, utxo, utxo_type, &prefix);
        let packet = packets.first().unwrap();
        let subject: Subjects = packet.clone().try_into()?;
        let db_item = UtxoDbItem::try_from(packet)?;

        // Assert store insert
        let inserted = store.insert_record(packet).await?;
        assert_eq!(db_item, inserted);

        // Verify common fields
        assert_eq!(db_item.block_height, 1);
        assert_eq!(db_item.tx_id, tx_id.to_string());
        assert_eq!(db_item.tx_index, 0);
        assert_eq!(db_item.input_index, 0);
        assert_eq!(db_item.subject, packet.subject_str());

        match subject {
            Subjects::Utxos(subject) => {
                assert_eq!(
                    db_item.utxo_type,
                    subject.utxo_type.unwrap().to_string()
                );
                assert_eq!(
                    db_item.utxo_id,
                    subject.utxo_id.unwrap().to_string()
                );
            }
            _ => panic!("Unexpected subject type"),
        }
    }

    Ok(())
}
