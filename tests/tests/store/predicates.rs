use fuel_streams_core::types::{
    Input,
    MockInput,
    MockTransaction,
    Transaction,
};
use fuel_streams_domains::{
    predicates::{DynPredicateSubject, Predicate, PredicateDbItem},
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

fn create_tx(inputs: &[Input]) -> (Transaction, TxId) {
    let tx = MockTransaction::script(inputs.to_vec(), vec![], vec![]);
    let tx_id = tx.to_owned().id;
    (tx, tx_id)
}

fn create_packets(
    tx: &Transaction,
    tx_id: &TxId,
    prefix: &str,
) -> Vec<RecordPacket> {
    tx.clone()
        .inputs
        .into_iter()
        .enumerate()
        .filter_map(|(input_index, input)| {
            let subject = DynPredicateSubject::new(
                &input,
                &1.into(),
                tx_id,
                0,
                input_index as u32,
            );

            subject.map(|dyn_subject| {
                let predicate = dyn_subject.predicate().to_owned();
                let msg_payload = MockMsgPayload::build(1, prefix);
                let timestamps = msg_payload.timestamp();
                let packet =
                    predicate.to_packet(&dyn_subject.into(), timestamps);
                match msg_payload.namespace.clone() {
                    Some(ns) => packet.with_namespace(&ns),
                    _ => packet,
                }
            })
        })
        .collect::<Vec<_>>()
}

async fn insert_predicate(input: Input) -> anyhow::Result<PredicateDbItem> {
    let prefix = create_random_db_name();
    let (tx, tx_id) = create_tx(&vec![input]);
    let packets = create_packets(&tx, &tx_id, &prefix);
    assert_eq!(packets.len(), 1);

    // Add namespace handling
    let mut store = setup_store::<Predicate>().await?;
    let packet = packets.first().unwrap().clone();
    store.with_namespace(&prefix);

    let db_item = PredicateDbItem::try_from(&packet);
    assert!(
        db_item.is_ok(),
        "Failed to convert packet to db item: {:?}",
        packet
    );

    let db_item = db_item.unwrap();
    let db_record = store.insert_record(&db_item).await?;
    assert_eq!(db_record.subject, packet.subject_str());

    close_db(&store.db).await;
    Ok(db_record)
}

#[tokio::test]
async fn store_can_record_predicate_with_blob_id() -> anyhow::Result<()> {
    let input = MockInput::coin_predicate();
    let record = insert_predicate(input).await?;
    assert!(record.blob_id.is_some());
    Ok(())
}

#[tokio::test]
async fn store_can_record_predicate_without_blob_id() -> anyhow::Result<()> {
    let input = MockInput::coin_signed();
    let record = insert_predicate(input).await?;
    assert!(record.blob_id.is_none());
    Ok(())
}

#[tokio::test]
async fn find_many_predicates_by_subject_with_sql_columns() -> anyhow::Result<()>
{
    let prefix = create_random_db_name();
    let mut store = setup_store::<Predicate>().await?;
    store.with_namespace(&prefix);

    let inputs = vec![MockInput::coin_predicate(), MockInput::coin_signed()];
    let (tx, tx_id) = create_tx(&inputs);
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
async fn test_predicate_subject_to_db_item_conversion() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let db = setup_db().await?;
    let mut store = Store::<Predicate>::new(&db);
    store.with_namespace(&prefix);

    let inputs = vec![MockInput::coin_predicate(), MockInput::coin_signed()];
    let (tx, tx_id) = create_tx(&inputs);
    let packets = create_packets(&tx, &tx_id, &prefix);

    for packet in packets {
        let payload = packet.subject_payload.clone();
        let subject: Subjects = payload.try_into()?;
        let db_item = PredicateDbItem::try_from(&packet)?;
        let inserted = store.insert_record(&db_item).await?;

        // Verify common fields
        assert_eq!(db_item.block_height, inserted.block_height);
        assert_eq!(db_item.tx_id, inserted.tx_id);
        assert_eq!(db_item.tx_index, inserted.tx_index);
        assert_eq!(db_item.input_index, inserted.input_index);
        assert_eq!(db_item.subject, inserted.subject);
        assert_eq!(db_item.predicate_bytecode, inserted.predicate_bytecode);
        assert_eq!(db_item.created_at, inserted.created_at);
        assert!(inserted.published_at.is_after(&db_item.published_at));
        assert_eq!(db_item.blob_id, inserted.blob_id);

        match subject {
            Subjects::Predicates(subject) => {
                assert_eq!(db_item.block_height, subject.block_height.unwrap());
                assert_eq!(db_item.tx_id, subject.tx_id.unwrap().to_string());
                assert_eq!(db_item.tx_index, subject.tx_index.unwrap() as i32);
                assert_eq!(
                    db_item.input_index,
                    subject.input_index.unwrap() as i32
                );
                assert_eq!(
                    db_item.predicate_address,
                    subject.predicate_address.unwrap().to_string()
                );
                assert_eq!(
                    db_item.blob_id,
                    subject.blob_id.map(|b| b.to_string())
                );
            }
            _ => panic!("Unexpected subject type"),
        }
    }

    close_db(&store.db).await;
    Ok(())
}
