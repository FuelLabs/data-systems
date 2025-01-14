use std::sync::Arc;

use fuel_streams_core::{
    inputs::{InputsCoinSubject, InputsContractSubject, InputsMessageSubject},
    subjects::IntoSubject,
    types::Input,
};
use fuel_streams_domains::{
    inputs::{types::MockInput, InputDbItem},
    transactions::types::MockTransaction,
};
use fuel_streams_store::record::RecordPacket;
use fuel_streams_test::{create_random_db_name, setup_store};

async fn insert_input(input: Input) -> anyhow::Result<()> {
    let tx = MockTransaction::script(vec![input.clone()], vec![], vec![]);
    let tx_id = tx.id;

    // Create packets from inputs similar to outputs/receipts
    let packets = tx
        .inputs
        .into_iter()
        .enumerate()
        .map(|(input_index, input)| {
            let subject: Arc<dyn IntoSubject> = match &input {
                Input::Coin(coin) => InputsCoinSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    input_index: Some(input_index as u32),
                    owner_id: Some(coin.owner.to_owned()),
                    asset_id: Some(coin.asset_id.to_owned()),
                }
                .arc(),
                Input::Contract(contract) => InputsContractSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    input_index: Some(input_index as u32),
                    contract_id: Some(contract.contract_id.to_owned().into()),
                }
                .arc(),
                Input::Message(message) => InputsMessageSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    input_index: Some(input_index as u32),
                    sender: Some(message.sender.to_owned()),
                    recipient: Some(message.recipient.to_owned()),
                }
                .arc(),
            };
            RecordPacket::new(subject, &input)
        })
        .collect::<Vec<_>>();
    assert_eq!(packets.len(), 1);

    // Add namespace handling
    let prefix = create_random_db_name();
    let mut store = setup_store::<Input>().await?;
    let packet = packets.first().unwrap().clone();
    let packet = packet.with_namespace(&prefix);
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
