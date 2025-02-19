use fuel_streams_core::types::{Output, Transaction};
use fuel_streams_domains::{
    outputs::{types::MockOutput, DynOutputSubject, OutputDbItem},
    transactions::types::MockTransaction,
    Subjects,
};
use fuel_streams_store::{
    record::{QueryOptions, Record, RecordPacket},
    store::Store,
};
use fuel_streams_test::{create_random_db_name, setup_db, setup_store};
use fuel_streams_types::TxId;
use pretty_assertions::assert_eq;

async fn insert_output(output: Output) -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let (tx, tx_id) = create_tx(vec![output]);
    let packets = create_packets(&tx, &tx_id, &prefix);
    assert_eq!(packets.len(), 1);

    let mut store = setup_store::<Output>().await?;
    let packet = packets.first().unwrap().clone();
    store.with_namespace(&prefix);

    let db_item = OutputDbItem::try_from(&packet);
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

fn create_tx(outputs: Vec<Output>) -> (Transaction, TxId) {
    let tx = MockTransaction::script(vec![], outputs, vec![]);
    let tx_id = tx.to_owned().id;
    (tx, tx_id)
}

fn create_packets(
    tx: &Transaction,
    tx_id: &TxId,
    prefix: &str,
) -> Vec<RecordPacket> {
    tx.clone()
        .outputs
        .into_iter()
        .enumerate()
        .map(|(output_index, output)| {
            let subject = DynOutputSubject::from((
                &output,
                1.into(),
                tx_id.clone(),
                0,
                output_index as u32,
                tx,
            ));
            output.to_packet(&subject.into()).with_namespace(prefix)
        })
        .collect()
}

#[tokio::test]
async fn store_can_record_coin_output() -> anyhow::Result<()> {
    insert_output(MockOutput::coin(100)).await
}

#[tokio::test]
async fn store_can_record_contract_output() -> anyhow::Result<()> {
    insert_output(MockOutput::contract()).await
}

#[tokio::test]
async fn store_can_record_change_output() -> anyhow::Result<()> {
    insert_output(MockOutput::change(50)).await
}

#[tokio::test]
async fn store_can_record_variable_output() -> anyhow::Result<()> {
    insert_output(MockOutput::variable(75)).await
}

#[tokio::test]
async fn store_can_record_contract_created_output() -> anyhow::Result<()> {
    insert_output(MockOutput::contract_created()).await
}

#[tokio::test]
async fn find_many_by_subject_with_sql_columns() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let mut store = setup_store::<Output>().await?;
    store.with_namespace(&prefix);

    // Create a transaction with all types of outputs
    let (tx, tx_id) = create_tx(vec![
        MockOutput::coin(100),
        MockOutput::contract(),
        MockOutput::change(50),
        MockOutput::variable(75),
        MockOutput::contract_created(),
    ]);
    let packets = create_packets(&tx, &tx_id, &prefix);
    for packet in packets {
        let payload = packet.subject_payload.clone();
        let subject: Subjects = payload.try_into()?;
        let subject = subject.into();
        let _ = store
            .find_many_by_subject(&subject, QueryOptions::default())
            .await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_output_subject_to_db_item_conversion() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let db = setup_db().await?;
    let mut store = Store::<Output>::new(&db.arc());
    store.with_namespace(&prefix);

    let outputs = vec![
        MockOutput::coin(100),
        MockOutput::contract(),
        MockOutput::change(50),
        MockOutput::variable(75),
        MockOutput::contract_created(),
    ];

    let (tx, tx_id) = create_tx(outputs);
    let packets = create_packets(&tx, &tx_id, &prefix);

    for (idx, packet) in packets.into_iter().enumerate() {
        let payload = packet.subject_payload.clone();
        let subject: Subjects = payload.try_into()?;
        let db_item = OutputDbItem::try_from(&packet)?;
        let inserted = store.insert_record(&db_item).await?;
        assert_eq!(db_item, inserted);

        // Verify common fields
        assert_eq!(db_item.block_height, 1);
        assert_eq!(db_item.tx_id, tx_id.to_string());
        assert_eq!(db_item.tx_index, 0);
        assert_eq!(db_item.output_index, idx as i32);
        assert_eq!(db_item.subject, packet.subject_str());

        match subject {
            Subjects::OutputsCoin(subject) => {
                assert_eq!(db_item.output_type, "coin");
                assert_eq!(
                    db_item.to_address,
                    Some(subject.to.unwrap().to_string())
                );
                assert_eq!(
                    db_item.asset_id,
                    Some(subject.asset.unwrap().to_string())
                );
                assert_eq!(db_item.contract_id, None);
            }
            Subjects::OutputsContract(subject) => {
                assert_eq!(db_item.output_type, "contract");
                assert_eq!(
                    db_item.contract_id,
                    Some(subject.contract.unwrap().to_string())
                );
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.asset_id, None);
            }
            Subjects::OutputsChange(subject) => {
                assert_eq!(db_item.output_type, "change");
                assert_eq!(
                    db_item.to_address,
                    Some(subject.to.unwrap().to_string())
                );
                assert_eq!(
                    db_item.asset_id,
                    Some(subject.asset.unwrap().to_string())
                );
                assert_eq!(db_item.contract_id, None);
            }
            Subjects::OutputsVariable(subject) => {
                assert_eq!(db_item.output_type, "variable");
                assert_eq!(
                    db_item.to_address,
                    Some(subject.to.unwrap().to_string())
                );
                assert_eq!(
                    db_item.asset_id,
                    Some(subject.asset.unwrap().to_string())
                );
                assert_eq!(db_item.contract_id, None);
            }
            Subjects::OutputsContractCreated(subject) => {
                assert_eq!(db_item.output_type, "contract_created");
                assert_eq!(
                    db_item.contract_id,
                    Some(subject.contract.unwrap().to_string())
                );
                assert_eq!(db_item.to_address, None);
                assert_eq!(db_item.asset_id, None);
            }
            _ => panic!("Unexpected subject type"),
        }
    }

    Ok(())
}
