use fuel_streams_core::{
    transactions::TransactionsSubject,
    types::{Transaction, UniqueIdentifier},
    Stream,
};
use tracing::info;

pub async fn publish(
    transactions_stream: &Stream<Transaction>,
    transactions: &[Transaction],
) -> anyhow::Result<()> {
    for transaction in transactions.iter() {
        // Publish the transaction.
        let transactions_subject: TransactionsSubject = transaction.into();

        // Publish the block.
        let transaction_id = transaction.cached_id().unwrap();
        info!("NATS Publisher: Publishing Transaction 0x#{transaction_id}");

        transactions_stream
            .publish(&transactions_subject, transaction)
            .await?;
    }

    Ok(())
}
