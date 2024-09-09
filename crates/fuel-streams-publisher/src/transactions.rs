use std::sync::Arc;

use chrono::Utc;
use fuel_core::combined_database::CombinedDatabase;
use fuel_core_storage::transactional::AtomicView;
use fuel_streams::types::{Address, Block};
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

use crate::metrics::PublisherMetrics;

#[allow(clippy::too_many_arguments)]
pub async fn publish(
    metrics: &Arc<PublisherMetrics>,
    chain_id: &ChainId,
    block_height: &BlockHeight,
    fuel_core_database: &CombinedDatabase,
    transactions_stream: &Stream<Transaction>,
    transactions: &[Transaction],
    block_producer: &Address,
    block: &Block<Transaction>,
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

        info!("NATS Publisher: Publishing Transaction 0x#{tx_id}");

        let published_data_size = match transactions_stream
            .publish(&transactions_subject, transaction)
            .await
        {
            Ok(published_data_size) => published_data_size,
            Err(e) => {
                metrics.error_rates.with_label_values(&[
                    &chain_id.to_string(),
                    &block_producer.to_string(),
                    TransactionsSubject::WILDCARD,
                    &e.to_string(),
                ]);
                return Err(anyhow::anyhow!(e));
            }
        };

        // update metric
        metrics
            .message_size_histogram
            .with_label_values(&[
                &chain_id.to_string(),
                &block_producer.to_string(),
                TransactionsSubject::WILDCARD,
            ])
            .observe(published_data_size as f64);

        let latency = block.header().time().to_unix() - Utc::now().timestamp();
        metrics
            .publishing_latency_histogram
            .with_label_values(&[
                &chain_id.to_string(),
                &block_producer.to_string(),
                TransactionsSubject::WILDCARD,
            ])
            .observe(latency as f64);

        metrics
            .total_published_messages
            .with_label_values(&[
                &chain_id.to_string(),
                &block_producer.to_string(),
            ])
            .inc();

        metrics
            .published_messages_throughput
            .with_label_values(&[
                &chain_id.to_string(),
                &block_producer.to_string(),
                TransactionsSubject::WILDCARD,
            ])
            .inc();
    }

    Ok(())
}
