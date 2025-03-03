use fuel_streams_core::types::{Input, MockInput, Transaction, Utxo};
use fuel_streams_domains::{
    transactions::types::MockTransaction,
    utxos::{DynUtxoSubject, UtxoDbItem},
    MockMsgPayload,
    Subjects,
};
use fuel_streams_store::{
    record::{QueryOptions, Record, RecordPacket},
    store::Store,
};
use fuel_streams_test::{create_random_db_name, setup_db, setup_store};
use fuel_streams_types::TxId;
use pretty_assertions::assert_eq;

async fn insert_utxo(input: &Input) -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let (_, tx_id) = create_tx();
    let packets = create_packets(input, &tx_id, &prefix, 0);
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

    let db_item = db_item.unwrap();
    let db_record = store.insert_record(&db_item).await?;
    assert_eq!(db_record.subject, packet.subject_str());

    Ok(())
}

fn create_tx() -> (Transaction, TxId) {
    let tx = MockTransaction::script(vec![], vec![], vec![]);
    let tx_id = tx.to_owned().id;
    (tx, tx_id)
}

fn create_packets(
    input: &Input,
    tx_id: &TxId,
    prefix: &str,
    input_index: u32,
) -> Vec<RecordPacket> {
    let subject =
        DynUtxoSubject::from((input, 1.into(), tx_id.clone(), 0, input_index));
    let msg_payload = MockMsgPayload::build(1, prefix);
    let timestamps = msg_payload.timestamp();
    let packet = subject.utxo().to_packet(subject.subject(), timestamps);
    vec![packet.with_namespace(prefix)]
}

#[tokio::test]
async fn store_can_record_coin_utxo() -> anyhow::Result<()> {
    insert_utxo(&MockInput::coin_signed()).await
}

#[tokio::test]
async fn store_can_record_contract_utxo() -> anyhow::Result<()> {
    insert_utxo(&MockInput::contract()).await
}

#[tokio::test]
async fn store_can_record_message_utxo() -> anyhow::Result<()> {
    insert_utxo(&MockInput::message_data_signed()).await
}

#[tokio::test]
async fn find_many_by_subject_with_sql_columns() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let mut store = setup_store::<Utxo>().await?;
    store.with_namespace(&prefix);

    let (_, tx_id) = create_tx();
    let inputs = vec![
        (MockInput::coin_signed(), 0),
        (MockInput::contract(), 1),
        (MockInput::message_data_signed(), 2),
    ];

    for (input, input_index) in inputs {
        let packets = create_packets(&input, &tx_id, &prefix, input_index);
        for packet in packets {
            let payload = packet.subject_payload.clone();
            let subject: Subjects = payload.try_into()?;
            let subject = subject.into();
            let _ = store
                .find_many_by_subject(&subject, QueryOptions::default())
                .await?;
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_utxo_subject_to_db_item_conversion() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let db = setup_db().await?;
    let mut store = Store::<Utxo>::new(&db);
    store.with_namespace(&prefix);

    let inputs = vec![
        (MockInput::coin_signed(), 0),
        (MockInput::contract(), 1),
        (MockInput::message_data_signed(), 2),
    ];

    for (input, input_index) in inputs {
        let (_, tx_id) = create_tx();
        let packets = create_packets(&input, &tx_id, &prefix, input_index);
        let packet = packets.first().unwrap();
        let payload = packet.subject_payload.clone();
        let subject: Subjects = payload.try_into()?;
        let db_item = UtxoDbItem::try_from(packet)?;
        let inserted = store.insert_record(&db_item).await?;

        // Verify common fields
        assert_eq!(db_item.block_height, inserted.block_height);
        assert_eq!(db_item.tx_id, inserted.tx_id);
        assert_eq!(db_item.tx_index, inserted.tx_index);
        assert_eq!(db_item.input_index, inserted.input_index);
        assert_eq!(db_item.subject, inserted.subject);
        assert_eq!(db_item.value, inserted.value);
        assert_eq!(db_item.created_at, inserted.created_at);
        assert!(inserted.published_at.is_after(&db_item.published_at));

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
