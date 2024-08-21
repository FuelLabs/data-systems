use fuel_core::combined_database::CombinedDatabase;
use fuel_core_storage::transactional::AtomicView;
use fuel_streams_core::{
    prelude::IntoSubject,
    transactions::TransactionsSubject,
    types::{
        BlockHeight,
        ChainId,
        Transaction,
        TransactionKind,
        TransactionStatus,
        UniqueIdentifier,
    },
    Stream,
};
use tracing::info;

pub async fn publish(
    chain_id: &ChainId,
    block_height: &BlockHeight,
    fuel_core_database: &CombinedDatabase,
    transactions_stream: &Stream<Transaction>,
    transactions: &[Transaction],
) -> anyhow::Result<()> {
    let off_chain_database = fuel_core_database.off_chain().latest_view()?;

    for (transaction_index, transaction) in transactions.iter().enumerate() {
        let tx_id = transaction.id(chain_id);
        let kind = TransactionKind::from(transaction.to_owned());
        let status: TransactionStatus = off_chain_database
            .get_tx_status(&tx_id)?
            .map(|status| status.into())
            .unwrap_or_default();

        let transactions_subject: TransactionsSubject =
            TransactionsSubject::new()
                .with_tx_id(Some(tx_id.into()))
                .with_kind(Some(kind))
                .with_status(Some(status))
                .with_height(Some(block_height.clone()))
                .with_tx_index(Some(transaction_index));

        let transaction_id = transaction.cached_id().unwrap();
        info!("NATS Publisher: Publishing Transaction 0x#{transaction_id}");

        transactions_stream
            .publish(&transactions_subject, transaction)
            .await?;
    }

    Ok(())
}
