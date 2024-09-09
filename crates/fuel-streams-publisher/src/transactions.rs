use fuel_core_storage::transactional::AtomicView;
use fuel_streams_core::{
    prelude::*,
    transactions::TransactionsSubject,
    types::{
        BlockHeight,
        Transaction,
        TransactionKind,
        TransactionStatus,
        UniqueIdentifier,
    },
    Stream,
};
use tracing::info;

use crate::FuelCoreLike;

pub async fn publish(
    block_height: &BlockHeight,
    fuel_core: &dyn FuelCoreLike,
    transactions_stream: &Stream<Transaction>,
    transactions: &[Transaction],
) -> anyhow::Result<()> {
    let chain_id = fuel_core.chain_id();
    let off_chain_database = fuel_core.database().off_chain().latest_view()?;

    for (transaction_index, transaction) in transactions.iter().enumerate() {
        let tx_id = transaction.id(chain_id);
        let kind = TransactionKind::from(transaction.to_owned());
        let status: TransactionStatus = off_chain_database
            .get_tx_status(&tx_id)?
            .map(|status| status.into())
            .unwrap_or_default();

        let transactions_subject = TransactionsSubject::new()
            .with_tx_id(Some(tx_id.into()))
            .with_kind(Some(kind))
            .with_status(Some(status))
            .with_height(Some(block_height.clone()))
            .with_tx_index(Some(transaction_index));

        info!("NATS Publisher: Publishing Transaction 0x#{tx_id}");

        transactions_stream
            .publish(&transactions_subject, transaction)
            .await?;
    }

    Ok(())
}
