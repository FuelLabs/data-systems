use std::sync::Arc;

use fuel_streams_core::{
    outputs::{
        OutputsChangeSubject,
        OutputsCoinSubject,
        OutputsContractCreatedSubject,
        OutputsContractSubject,
        OutputsVariableSubject,
    },
    subjects::IntoSubject,
    types::Output,
};
use fuel_streams_domains::{
    outputs::{types::MockOutput, OutputDbItem},
    transactions::types::MockTransaction,
};
use fuel_streams_store::record::RecordPacket;
use fuel_streams_test::{create_random_db_name, setup_store};
use fuel_streams_types::ContractId;

#[tokio::test]
async fn store_can_record_coin_output() -> anyhow::Result<()> {
    test_output_type(MockOutput::coin(100)).await
}

#[tokio::test]
async fn store_can_record_contract_output() -> anyhow::Result<()> {
    test_output_type(MockOutput::contract()).await
}

#[tokio::test]
async fn store_can_record_change_output() -> anyhow::Result<()> {
    test_output_type(MockOutput::change(50)).await
}

#[tokio::test]
async fn store_can_record_variable_output() -> anyhow::Result<()> {
    test_output_type(MockOutput::variable(75)).await
}

#[tokio::test]
async fn store_can_record_contract_created_output() -> anyhow::Result<()> {
    test_output_type(MockOutput::contract_created()).await
}

async fn test_output_type(output: Output) -> anyhow::Result<()> {
    let tx = MockTransaction::script(vec![], vec![output.clone()], vec![]);
    let tx_id = tx.id;
    let packets = tx
        .outputs
        .into_iter()
        .enumerate()
        .map(|(output_index, output)| {
            let subject: Arc<dyn IntoSubject> = match &output {
                Output::Coin(coin) => OutputsCoinSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    output_index: Some(output_index as u32),
                    to_address: Some(coin.to.to_owned()),
                    asset_id: Some(coin.asset_id.to_owned()),
                }
                .arc(),
                Output::Contract(_) => OutputsContractSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    output_index: Some(output_index as u32),
                    contract_id: Some(ContractId::default()),
                }
                .arc(),
                Output::Change(change) => OutputsChangeSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    output_index: Some(output_index as u32),
                    to_address: Some(change.to.to_owned()),
                    asset_id: Some(change.asset_id.to_owned()),
                }
                .arc(),
                Output::Variable(variable) => OutputsVariableSubject {
                    block_height: Some(1.into()),
                    tx_id: Some(tx_id.clone().into()),
                    tx_index: Some(0),
                    output_index: Some(output_index as u32),
                    to_address: Some(variable.to.to_owned()),
                    asset_id: Some(variable.asset_id.to_owned()),
                }
                .arc(),
                Output::ContractCreated(contract_created) => {
                    OutputsContractCreatedSubject {
                        block_height: Some(1.into()),
                        tx_id: Some(tx_id.clone().into()),
                        tx_index: Some(0),
                        output_index: Some(output_index as u32),
                        contract_id: Some(
                            contract_created.contract_id.to_owned(),
                        ),
                    }
                    .arc()
                }
            };
            RecordPacket::new(subject, &output)
        })
        .collect::<Vec<_>>();
    assert_eq!(packets.len(), 1);

    // Add namespace handling
    let prefix = create_random_db_name();
    let mut store = setup_store::<Output>().await?;
    let packet = packets.first().unwrap().clone();
    let packet = packet.with_namespace(&prefix);
    store.with_namespace(&prefix);

    let db_item = OutputDbItem::try_from(&packet);
    assert!(
        db_item.is_ok(),
        "Failed to convert packet to db item: {:?}",
        packet
    );

    let db_record = store.insert_record(&packet).await?;
    assert_eq!(db_record.subject, packet.subject_str());

    Ok(())
}
