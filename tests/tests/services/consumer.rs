use std::{collections::HashSet, sync::Arc};

use fuel_data_parser::DataEncoder;
use fuel_message_broker::{NatsMessageBroker, NatsQueue, NatsSubject};
use fuel_streams_core::{types::*, FuelStreams};
use fuel_streams_domains::{
    blocks::BlocksQuery,
    infra::{db::Db, QueryParamsBuilder, Repository},
    inputs::InputsQuery,
    outputs::OutputsQuery,
    predicates::PredicatesQuery,
    receipts::ReceiptsQuery,
    transactions::TransactionsQuery,
    utxos::{DynUtxoSubject, UtxosQuery},
    MockMsgPayload,
    MsgPayload,
};
use fuel_streams_test::{close_db, create_random_db_name, setup_db};
use fuel_web_utils::{shutdown::ShutdownController, telemetry::Telemetry};
use pretty_assertions::assert_eq;
use sv_consumer::BlockExecutor;

async fn verify_blocks(
    db: &Arc<Db>,
    prefix: &str,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let mut query = BlocksQuery {
        height: Some(msg_payload.block_height()),
        ..Default::default()
    };
    query.with_namespace(Some(prefix.to_string()));
    let blocks = Block::find_many(db.pool_ref(), &query).await?;
    assert!(!blocks.is_empty(), "Expected blocks to be inserted");

    let msg_payload_height = msg_payload.block_height();
    let saved_height = blocks[0].block_height;
    assert_eq!(saved_height, msg_payload_height);

    Ok(())
}

async fn verify_transactions(
    db: &Arc<Db>,
    prefix: &str,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let mut query = TransactionsQuery {
        block_height: Some(msg_payload.block_height()),
        ..Default::default()
    };
    query.with_namespace(Some(prefix.to_string()));
    let transactions = Transaction::find_many(db.pool_ref(), &query).await?;
    assert!(
        !transactions.is_empty(),
        "Expected transactions to be inserted"
    );

    let expected_tx_ids: Vec<String> = msg_payload
        .tx_ids()
        .into_iter()
        .map(|id| id.to_string())
        .collect();
    let actual_tx_ids: Vec<String> = transactions
        .iter()
        .map(|tx| {
            let decoded = Transaction::decode_json(&tx.value).unwrap();
            decoded.id.to_string()
        })
        .collect();

    assert_eq!(
        actual_tx_ids.len(),
        expected_tx_ids.len(),
        "Expected all transactions to be inserted"
    );
    for tx_id in expected_tx_ids {
        assert!(
            actual_tx_ids.contains(&tx_id),
            "Expected transaction {} to be inserted",
            tx_id
        );
    }

    assert_eq!(
        transactions.len(),
        msg_payload.transactions.len(),
        "Expected exact number of transactions to be inserted"
    );

    Ok(())
}

async fn verify_receipts(
    db: &Arc<Db>,
    prefix: &str,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let mut query = ReceiptsQuery {
        block_height: Some(msg_payload.block_height()),
        ..Default::default()
    };
    query.with_namespace(Some(prefix.to_string()));
    let receipts = Receipt::find_many(db.pool_ref(), &query).await?;

    let expected_receipts_count: usize = msg_payload
        .transactions
        .iter()
        .map(|tx| tx.receipts.len())
        .sum();

    assert_eq!(
        receipts.len(),
        expected_receipts_count,
        "Expected exact number of receipts to be inserted"
    );

    Ok(())
}

async fn verify_inputs(
    db: &Arc<Db>,
    prefix: &str,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let expected_inputs_count: usize = msg_payload
        .transactions
        .iter()
        .map(|tx| tx.inputs.len())
        .sum();

    let mut query = InputsQuery {
        block_height: Some(msg_payload.block_height()),
        ..Default::default()
    };
    query.with_namespace(Some(prefix.to_string()));
    let inputs = Input::find_many(db.pool_ref(), &query).await?;
    assert_eq!(
        inputs.len(),
        expected_inputs_count,
        "Expected exact number of inputs to be inserted"
    );

    Ok(())
}

async fn verify_outputs(
    db: &Arc<Db>,
    prefix: &str,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let expected_outputs_count: usize = msg_payload
        .transactions
        .iter()
        .map(|tx| tx.outputs.len())
        .sum();

    let mut query = OutputsQuery {
        block_height: Some(msg_payload.block_height()),
        ..Default::default()
    };
    query.with_namespace(Some(prefix.to_string()));
    let outputs = Output::find_many(db.pool_ref(), &query).await?;
    assert_eq!(
        outputs.len(),
        expected_outputs_count,
        "Expected exact number of outputs to be inserted"
    );

    Ok(())
}

async fn verify_utxos(
    db: &Arc<Db>,
    prefix: &str,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let input_utxos: Vec<UtxoId> = msg_payload
        .transactions
        .iter()
        .flat_map(|tx| {
            tx.inputs.iter().filter_map(|i| match i {
                Input::Contract(input_contract) => {
                    Some(input_contract.utxo_id.to_owned())
                }
                Input::Coin(input_coin) => Some(input_coin.utxo_id.to_owned()),
                Input::Message(_) => None,
            })
        })
        .collect();

    let unique_utxos: HashSet<_> = input_utxos.into_iter().collect();

    let output_utxos: Vec<UtxoId> = msg_payload
        .transactions
        .iter()
        .flat_map(|tx| {
            tx.outputs.iter().enumerate().filter_map(
                |(output_index, output)| {
                    // Only consider Coin, Change, and Variable outputs as they can be spent
                    match output {
                        Output::Coin(_)
                        | Output::Change(_)
                        | Output::Variable(_) => {
                            Some(DynUtxoSubject::build_utxo_id(
                                &tx.id,
                                output_index as i32,
                            ))
                        }
                        // Contract and ContractCreated outputs don't create UTXOs
                        Output::Contract(_) | Output::ContractCreated(_) => {
                            None
                        }
                    }
                },
            )
        })
        .collect();

    // If you need unique UTXOs (though they should already be unique by construction)
    let unique_output_utxos: HashSet<_> = output_utxos.into_iter().collect();

    let mut query = UtxosQuery {
        block_height: Some(msg_payload.block_height()),
        ..Default::default()
    };
    query.with_namespace(Some(prefix.to_string()));
    let utxos = Utxo::find_many(db.pool_ref(), &query).await?;
    assert_eq!(
        utxos.len(),
        unique_utxos.len() + unique_output_utxos.len(),
        "Expected exact number of UTXOs to be inserted"
    );

    Ok(())
}

async fn verify_predicates(
    db: &Arc<Db>,
    prefix: &str,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let expected_predicates_count: usize = msg_payload
        .transactions
        .iter()
        .map(|tx| tx.inputs.iter().filter(|i| i.is_coin()).count())
        .sum();

    let mut query = PredicatesQuery {
        block_height: Some(msg_payload.block_height()),
        ..Default::default()
    };
    query.with_namespace(Some(prefix.to_string()));
    let predicates = Predicate::find_many(db.pool_ref(), &query).await?;
    assert_eq!(
        predicates.len(),
        expected_predicates_count,
        "Expected exact number of inputs to be inserted"
    );

    Ok(())
}

#[tokio::test]
async fn test_consumer_inserting_records() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let db = setup_db().await?;
    let shutdown = Arc::new(ShutdownController::new());
    shutdown.clone().spawn_signal_handler();
    let message_broker =
        NatsMessageBroker::setup("nats://localhost:4222", Some(&prefix))
            .await?;

    let fuel_streams = FuelStreams::new(&message_broker, &db).await.arc();
    let msg_payload = MockMsgPayload::new(BlockHeight::random())
        .into_inner()
        .with_namespace(&prefix);
    let encoded_payload = msg_payload.encode_json()?;
    let queue = NatsQueue::BlockImporter(message_broker.clone());
    let block_height = msg_payload.block_height().into();
    let subject = NatsSubject::BlockSubmitted(block_height);
    queue.publish(&subject, encoded_payload).await?;

    let handle = tokio::spawn({
        let db = db.clone();
        let shutdown = shutdown.clone();
        let message_broker = Arc::clone(&message_broker);
        let fuel_streams = Arc::clone(&fuel_streams);
        let telemetry = Telemetry::new(None).await?;
        let block_executor =
            BlockExecutor::new(db, &message_broker, &fuel_streams, telemetry);
        async move { block_executor.start(shutdown.token()).await }
    });

    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
    shutdown.initiate_shutdown();
    let _ = handle.await?;

    verify_blocks(&db, &prefix, &msg_payload).await?;
    verify_transactions(&db, &prefix, &msg_payload).await?;
    verify_receipts(&db, &prefix, &msg_payload).await?;
    verify_inputs(&db, &prefix, &msg_payload).await?;
    verify_outputs(&db, &prefix, &msg_payload).await?;
    verify_utxos(&db, &prefix, &msg_payload).await?;
    verify_predicates(&db, &prefix, &msg_payload).await?;

    close_db(&db).await;
    Ok(())
}
