use std::sync::Arc;

use fuel_streams_core::{
    inputs::{InputsCoinSubject, InputsContractSubject, InputsMessageSubject},
    subjects::IntoSubject,
    types::{Input, Transaction},
};
use fuel_streams_domains::{
    inputs::{types::MockInput, InputDbItem},
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

async fn insert_input(input: Input) -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let (tx, tx_id) = create_tx(vec![input]);
    let packets = create_packets(&tx, &tx_id, &prefix);
    assert_eq!(packets.len(), 1);

    // Add namespace handling
    let mut store = setup_store::<Input>().await?;
    let packet = packets.first().unwrap().clone();
    store.with_namespace(&prefix);

    let db_item = InputDbItem::try_from(&packet);
    assert!(
        db_item.is_ok(),
        "Failed to convert packet to db item: {:?}",
        packet
    );

    let db_record = store.insert_record(&packet).await?;
    assert_eq!(db_record.subject, packet.subject_str());

    Ok(())
}

fn create_tx(inputs: Vec<Input>) -> (Transaction, TxId) {
    let tx = MockTransaction::script(inputs, vec![], vec![]);
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
        .map(|(input_index, input)| {
            let subject: Arc<dyn IntoSubject> = match &input {
                Input::Coin(coin) => InputsCoinSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    input_index: Some(input_index as u32),
                    owner: Some(coin.owner.to_owned()),
                    asset: Some(coin.asset_id.to_owned()),
                }
                .arc(),
                Input::Contract(contract) => InputsContractSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    input_index: Some(input_index as u32),
                    contract: Some(contract.contract_id.to_owned().into()),
                }
                .arc(),
                Input::Message(message) => InputsMessageSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone()),
                    tx_index: Some(0),
                    input_index: Some(input_index as u32),
                    sender: Some(message.sender.to_owned()),
                    recipient: Some(message.recipient.to_owned()),
                }
                .arc(),
            };
            input.to_packet(&subject).with_namespace(prefix)
        })
        .collect::<Vec<_>>()
}

#[tokio::test]
async fn store_can_record_coin_input() -> anyhow::Result<()> {
    insert_input(MockInput::coin_signed()).await
}

#[tokio::test]
async fn store_can_record_contract_input() -> anyhow::Result<()> {
    insert_input(MockInput::contract()).await
}

#[tokio::test]
async fn store_can_record_message_input() -> anyhow::Result<()> {
    insert_input(MockInput::message_coin_signed()).await
}

#[tokio::test]
async fn find_many_by_subject_with_sql_columns() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let mut store = setup_store::<Input>().await?;
    store.with_namespace(&prefix);

    // Create a transaction with all types of inputs
    let (tx, tx_id) = create_tx(vec![
        MockInput::coin_signed(),
        MockInput::contract(),
        MockInput::message_coin_signed(),
    ]);
    let packets = create_packets(&tx, &tx_id, &prefix);
    for packet in packets {
        let _ = store
            .find_many_by_subject(&packet.subject, QueryOptions::default())
            .await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_input_subject_to_db_item_conversion() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let db = setup_db().await?;
    let mut store = Store::<Input>::new(&db.arc());
    store.with_namespace(&prefix);

    let inputs = vec![
        MockInput::coin_signed(),
        MockInput::contract(),
        MockInput::message_coin_signed(),
    ];

    let (tx, tx_id) = create_tx(inputs);
    let packets = create_packets(&tx, &tx_id, &prefix);

    for (idx, packet) in packets.into_iter().enumerate() {
        let subject: Subjects = packet.clone().try_into()?;
        let db_item = InputDbItem::try_from(&packet)?;

        // Assert store insert
        let inserted = store.insert_record(&packet).await?;
        assert_eq!(db_item, inserted);

        // Verify common fields
        assert_eq!(db_item.block_height, 1);
        assert_eq!(db_item.tx_id, tx_id.to_string());
        assert_eq!(db_item.tx_index, 0);
        assert_eq!(db_item.input_index, idx as i32);
        assert_eq!(db_item.subject, packet.subject_str());

        match subject {
            Subjects::InputsCoin(subject) => {
                assert_eq!(db_item.input_type, "coin");
                assert_eq!(
                    db_item.owner_id,
                    Some(subject.owner.unwrap().to_string())
                );
                assert_eq!(
                    db_item.asset_id,
                    Some(subject.asset.unwrap().to_string())
                );
                assert_eq!(db_item.contract_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            Subjects::InputsContract(subject) => {
                assert_eq!(db_item.input_type, "contract");
                assert_eq!(
                    db_item.contract_id,
                    Some(subject.contract.unwrap().to_string())
                );
                assert_eq!(db_item.owner_id, None);
                assert_eq!(db_item.asset_id, None);
                assert_eq!(db_item.sender_address, None);
                assert_eq!(db_item.recipient_address, None);
            }
            Subjects::InputsMessage(subject) => {
                assert_eq!(db_item.input_type, "message");
                assert_eq!(
                    db_item.sender_address,
                    Some(subject.sender.unwrap().to_string())
                );
                assert_eq!(
                    db_item.recipient_address,
                    Some(subject.recipient.unwrap().to_string())
                );
                assert_eq!(db_item.owner_id, None);
                assert_eq!(db_item.asset_id, None);
                assert_eq!(db_item.contract_id, None);
            }
            _ => panic!("Unexpected subject type"),
        }
    }

    Ok(())
}
