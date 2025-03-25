use std::sync::Arc;

use fuel_message_broker::{NatsMessageBroker, NatsQueue, NatsSubject};
use fuel_streams_core::{
    inputs::InputsSubject,
    outputs::OutputsSubject,
    subjects::{
        BlocksSubject,
        ReceiptsSubject,
        SubjectBuildable,
        TransactionsSubject,
        UtxosSubject,
    },
    types::Transaction,
    FuelStreams,
};
use fuel_streams_domains::{
    predicates::PredicatesSubject,
    MockMsgPayload,
    MsgPayload,
};
use fuel_streams_store::record::{DataEncoder, QueryOptions};
use fuel_streams_test::{close_db, create_random_db_name, setup_db};
use fuel_web_utils::{shutdown::ShutdownController, telemetry::Telemetry};
use pretty_assertions::assert_eq;
use sv_consumer::{BlockExecutor, FuelStores};

async fn verify_blocks(
    prefix: &str,
    fuel_stores: &Arc<FuelStores>,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let block_subject = BlocksSubject::new()
        .with_height(Some(msg_payload.block_height()))
        .dyn_arc();
    let options =
        QueryOptions::default().with_namespace(Some(prefix.to_string()));
    let blocks = fuel_stores
        .blocks
        .find_many_by_subject(&block_subject, options)
        .await?;
    assert!(!blocks.is_empty(), "Expected blocks to be inserted");

    let msg_payload_height = msg_payload.block_height();
    let saved_height = blocks[0].block_height;
    assert_eq!(saved_height, msg_payload_height);

    Ok(())
}

async fn verify_transactions(
    prefix: &str,
    fuel_stores: &Arc<FuelStores>,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let tx_subject = TransactionsSubject::new()
        .with_block_height(Some(msg_payload.block_height()))
        .dyn_arc();
    let options =
        QueryOptions::default().with_namespace(Some(prefix.to_string()));
    let transactions = fuel_stores
        .transactions
        .find_many_by_subject(&tx_subject, options)
        .await?;
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
    prefix: &str,
    fuel_stores: &Arc<FuelStores>,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let receipts_subject = ReceiptsSubject::new()
        .with_block_height(Some(msg_payload.block_height()))
        .dyn_arc();
    let options =
        QueryOptions::default().with_namespace(Some(prefix.to_string()));
    let receipts = fuel_stores
        .receipts
        .find_many_by_subject(&receipts_subject, options)
        .await?;

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
    prefix: &str,
    fuel_stores: &Arc<FuelStores>,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let expected_inputs_count: usize = msg_payload
        .transactions
        .iter()
        .map(|tx| tx.inputs.len())
        .sum();

    let inputs_subject = InputsSubject::new()
        .with_block_height(Some(msg_payload.block_height()))
        .dyn_arc();
    let options =
        QueryOptions::default().with_namespace(Some(prefix.to_string()));
    let inputs = fuel_stores
        .inputs
        .find_many_by_subject(&inputs_subject, options)
        .await?;
    assert_eq!(
        inputs.len(),
        expected_inputs_count,
        "Expected exact number of inputs to be inserted"
    );

    Ok(())
}

async fn verify_outputs(
    prefix: &str,
    fuel_stores: &Arc<FuelStores>,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let expected_outputs_count: usize = msg_payload
        .transactions
        .iter()
        .map(|tx| tx.outputs.len())
        .sum();

    let outputs_subject = OutputsSubject::new()
        .with_block_height(Some(msg_payload.block_height()))
        .dyn_arc();
    let options =
        QueryOptions::default().with_namespace(Some(prefix.to_string()));
    let outputs = fuel_stores
        .outputs
        .find_many_by_subject(&outputs_subject, options)
        .await?;
    assert_eq!(
        outputs.len(),
        expected_outputs_count,
        "Expected exact number of outputs to be inserted"
    );

    Ok(())
}

async fn verify_utxos(
    prefix: &str,
    fuel_stores: &Arc<FuelStores>,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let expected_utxos_count: usize = msg_payload
        .transactions
        .iter()
        .map(|tx| tx.inputs.len())
        .sum();

    let utxos_subject = UtxosSubject::new()
        .with_block_height(Some(msg_payload.block_height()))
        .dyn_arc();
    let options =
        QueryOptions::default().with_namespace(Some(prefix.to_string()));
    let utxos = fuel_stores
        .utxos
        .find_many_by_subject(&utxos_subject, options)
        .await?;
    assert_eq!(
        utxos.len(),
        expected_utxos_count,
        "Expected exact number of UTXOs to be inserted"
    );

    Ok(())
}

async fn verify_predicates(
    prefix: &str,
    fuel_stores: &Arc<FuelStores>,
    msg_payload: &MsgPayload,
) -> anyhow::Result<()> {
    let expected_predicates_count: usize = msg_payload
        .transactions
        .iter()
        .map(|tx| tx.inputs.iter().filter(|i| i.is_coin()).count())
        .sum();

    let predicates_subject = PredicatesSubject::new()
        .with_block_height(Some(msg_payload.block_height()))
        .dyn_arc();
    let options =
        QueryOptions::default().with_namespace(Some(prefix.to_string()));
    let predicates = fuel_stores
        .predicates
        .find_many_by_subject(&predicates_subject, options)
        .await?;
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
    let fuel_stores = FuelStores::new(&db).with_namespace(&prefix).arc();
    let msg_payload =
        MockMsgPayload::new(1).into_inner().with_namespace(&prefix);
    let encoded_payload = msg_payload.encode().await?;
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

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    shutdown.initiate_shutdown();
    let _ = handle.await?;

    verify_blocks(&prefix, &fuel_stores, &msg_payload).await?;
    verify_transactions(&prefix, &fuel_stores, &msg_payload).await?;
    verify_receipts(&prefix, &fuel_stores, &msg_payload).await?;
    verify_inputs(&prefix, &fuel_stores, &msg_payload).await?;
    verify_outputs(&prefix, &fuel_stores, &msg_payload).await?;
    verify_utxos(&prefix, &fuel_stores, &msg_payload).await?;
    verify_predicates(&prefix, &fuel_stores, &msg_payload).await?;

    close_db(&db).await;
    Ok(())
}
