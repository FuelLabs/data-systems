use async_nats::jetstream::context::Publish;
use fuel_core::combined_database::CombinedDatabase;
use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::{
    blockchain::block::Block,
    fuel_tx::{Transaction, UniqueIdentifier},
    fuel_types::ChainId,
    services::txpool::TransactionStatus as TxPoolTransactionStatus,
};
use fuel_streams_core::{
    blocks::types::BlockHeight,
    prelude::IntoSubject,
    transactions::TransactionsSubject,
};
use tokio::try_join;
use tracing::info;

use super::nats::NatsHelper;

#[allow(unused)]
#[derive(Clone)]
pub struct TxHelper {
    nats: NatsHelper,
    chain_id: ChainId,
    database: CombinedDatabase,
}

#[allow(unused)]
/// Public
impl TxHelper {
    pub fn new(
        nats: NatsHelper,
        chain_id: &ChainId,
        database: &CombinedDatabase,
    ) -> Self {
        Self {
            nats,
            chain_id: chain_id.to_owned(),
            database: database.to_owned(),
        }
    }

    pub async fn publish(
        &self,
        block: &Block,
        tx: &Transaction,
        index: usize,
    ) -> anyhow::Result<()> {
        try_join!(
            self.publish_core(block, tx, index),
            self.publish_encoded(block, tx, index),
            self.publish_to_kv(block, tx, index)
        )?;
        Ok(())
    }
}

/// Publishers
impl TxHelper {
    async fn publish_core(
        &self,
        block: &Block,
        tx: &Transaction,
        index: usize,
    ) -> anyhow::Result<()> {
        let subject = &self.get_subject(tx, block, index);
        let payload = self
            .nats
            .data_parser()
            .to_nats_payload(&subject.parse(), block)
            .await?;
        self.nats
            .context
            .publish(subject.parse(), payload.into())
            .await?;
        Ok(())
    }

    async fn publish_encoded(
        &self,
        block: &Block,
        tx: &Transaction,
        index: usize,
    ) -> anyhow::Result<()> {
        let tx_id = self.get_id(tx);
        let subject = self.get_subject(tx, block, index);
        let payload = self
            .nats
            .data_parser()
            .to_nats_payload(&subject.parse(), block)
            .await?;
        let nats_payload = Publish::build()
            .message_id(subject.parse())
            .payload(payload.into());

        self.nats
            .context
            .send_publish(subject.parse(), nats_payload)
            .await?
            .await?;

        info!(
            "NATS: publishing transaction {} json to stream \"transactions_encoded\"",
            tx_id
        );
        Ok(())
    }

    async fn publish_to_kv(
        &self,
        block: &Block,
        tx: &Transaction,
        index: usize,
    ) -> anyhow::Result<()> {
        let tx_id = self.get_id(tx);
        let subject = self.get_subject(tx, block, index);
        let payload = self
            .nats
            .data_parser()
            .to_nats_payload(&subject.parse(), block)
            .await?;
        self.nats
            .kv_transactions
            .put(subject.parse(), payload.into())
            .await?;

        info!(
            "NATS: publishing transaction {} to kv store \"transactions\"",
            tx_id
        );
        Ok(())
    }
}

/// Getters
impl TxHelper {
    fn get_subject(
        &self,
        tx: &Transaction,
        block: &Block,
        index: usize,
    ) -> TransactionsSubject {
        // construct tx subject
        let mut subject: TransactionsSubject = tx.into();
        subject = subject
            .with_tx_index(Some(index))
            .with_height(Some(BlockHeight::from(self.get_height(block))))
            .with_status(self.get_status(tx).map(Into::into));
        subject
    }

    fn get_id(&self, tx: &Transaction) -> String {
        let id = tx.id(&self.chain_id).to_string();
        format!("0x{id}")
    }

    fn get_height(&self, block: &Block) -> u32 {
        *block.header().consensus().height
    }

    fn get_status(&self, tx: &Transaction) -> Option<TxPoolTransactionStatus> {
        self.database
            .off_chain()
            .latest_view()
            .unwrap()
            .get_tx_status(&tx.id(&self.chain_id))
            .ok()
            .flatten()
    }
}
