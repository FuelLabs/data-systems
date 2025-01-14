use fuel_core::service::Config;
use fuel_core_bin::FuelService;
use fuel_core_client::client::FuelClient;
use fuel_streams_core::{
    subjects::{SubjectBuildable, TransactionsSubject},
    types::{MockInput, MockOutput, MockReceipt, MockTransaction, Transaction},
};
use fuel_streams_domains::transactions::TransactionStatus;
use fuel_streams_store::record::RecordPacket;
use fuel_streams_test::{create_random_db_name, setup_store};
use fuel_streams_types::{
    Bytes32,
    FuelCoreAssetId,
    FuelCoreChainId,
    FuelCoreTransaction,
    FuelCoreUniqueIdentifier,
};

async fn insert_transaction(tx: &Transaction) -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let mut store = setup_store::<Transaction>().await?;
    store.with_namespace(&prefix);

    let packet = RecordPacket::new(
        TransactionsSubject::new()
            .with_block_height(Some(1.into()))
            .with_tx_id(Some(tx.id.clone()))
            .with_tx_index(Some(0))
            .with_tx_status(Some(tx.status.clone()))
            .with_kind(Some(tx.kind.clone()))
            .arc(),
        tx,
    )
    .with_namespace(&prefix);

    let db_record = store.insert_record(&packet).await?;
    assert_eq!(db_record.subject, packet.subject_str());
    Ok(())
}

#[tokio::test]
async fn test_record_transactions_from_fuel_core() -> anyhow::Result<()> {
    let config = Config::local_node();
    let srv = FuelService::new_node(config).await?;
    let client = FuelClient::from(srv.bound_address);
    let tx = FuelCoreTransaction::default_test_tx();
    let ftx_id = tx.id(&FuelCoreChainId::default());
    let tx_id = Bytes32::from(ftx_id);
    client.submit_and_await_commit(&tx).await?;

    let client_tx = client.transaction(&ftx_id).await?;
    assert!(client_tx.is_some());

    let tx_response = client_tx.unwrap();
    let status: TransactionStatus = tx_response.status.into();
    let transaction = Transaction::new(
        &tx_id,
        &tx_response.transaction,
        &status,
        &FuelCoreAssetId::default(),
        &[],
    );

    insert_transaction(&transaction).await?;
    Ok(())
}

#[tokio::test]
async fn test_store_script_transaction() -> anyhow::Result<()> {
    let tx = MockTransaction::script(
        vec![MockInput::coin_signed()],
        vec![MockOutput::coin(100)],
        vec![MockReceipt::script_result()],
    );
    insert_transaction(&tx).await
}

#[tokio::test]
async fn test_store_create_transaction() -> anyhow::Result<()> {
    let tx = MockTransaction::create(
        vec![MockInput::contract()],
        vec![MockOutput::contract()],
        vec![MockReceipt::call()],
    );
    insert_transaction(&tx).await
}

#[tokio::test]
async fn test_store_mint_transaction() -> anyhow::Result<()> {
    let tx = MockTransaction::mint(
        vec![MockInput::contract()],
        vec![MockOutput::coin(1000)],
        vec![MockReceipt::mint()],
    );
    insert_transaction(&tx).await
}

#[tokio::test]
async fn test_store_upgrade_transaction() -> anyhow::Result<()> {
    let tx = MockTransaction::upgrade(
        vec![MockInput::coin_signed()],
        vec![MockOutput::coin(100)],
        vec![MockReceipt::script_result()],
    );
    insert_transaction(&tx).await
}

#[tokio::test]
async fn test_store_upload_transaction() -> anyhow::Result<()> {
    let tx = MockTransaction::upload(
        vec![MockInput::coin_signed()],
        vec![MockOutput::coin(100)],
        vec![MockReceipt::script_result()],
    );
    insert_transaction(&tx).await
}

#[tokio::test]
async fn test_store_blob_transaction() -> anyhow::Result<()> {
    let tx = MockTransaction::blob(
        vec![MockInput::coin_signed()],
        vec![MockOutput::coin(100)],
        vec![MockReceipt::script_result()],
    );
    insert_transaction(&tx).await
}
