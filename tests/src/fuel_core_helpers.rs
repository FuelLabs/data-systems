use fuel_core_client::client::FuelClient;
use fuel_core_types::fuel_tx::{
    Finalizable,
    Transaction,
    TransactionBuilder,
    Word,
};
use fuel_streams_types::fuel_core::{
    FuelCoreClientTransactionStatus,
    FuelCoreTransaction,
};

pub fn tx_for_gas_limit(max_fee_limit: Word) -> FuelCoreTransaction {
    TransactionBuilder::script(vec![], vec![])
        .max_fee_limit(max_fee_limit)
        .add_fee_input()
        .finalize()
        .into()
}

pub fn create_multiple_txs(number: usize) -> Vec<Transaction> {
    let mut txs = Vec::new();
    for _ in 0..number {
        txs.push(tx_for_gas_limit(1));
    }
    txs
}

pub async fn submit_txs(
    client: &FuelClient,
    txs: Vec<Transaction>,
) -> Vec<FuelCoreClientTransactionStatus> {
    let mut statuses = Vec::new();
    for tx in txs {
        let status = client.submit_and_await_commit(&tx).await.unwrap();
        client.latest_gas_price().await.unwrap();
        statuses.push(status);
    }
    statuses
}
