use async_nats::jetstream::context::Publish;
use fuel_core::combined_database::CombinedDatabase;
use fuel_core_types::{
    blockchain::block::Block,
    fuel_tx::{Transaction, UniqueIdentifier},
    fuel_types::ChainId,
    services::txpool::TransactionStatus as TxPoolTransactionStatus,
};
use fuel_streams_core::{
    nats::{streams::transactions::TransactionsSubject, Subject},
    types::TransactionKind,
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
        let subject = self.get_subject(block, tx, index);
        let payload = self
            .nats
            .data_parser()
            .to_nats_payload(&subject, block)
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
        let subject = self.get_subject(block, tx, index);
        let payload = self
            .nats
            .data_parser()
            .to_nats_payload(&subject, block)
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
        let subject = self.get_subject(block, tx, index);
        let payload = self
            .nats
            .data_parser()
            .to_nats_payload(&subject, block)
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
    fn get_id(&self, tx: &Transaction) -> String {
        let id = tx.id(&self.chain_id).to_string();
        format!("0x{id}")
    }

    fn get_status(&self, tx: &Transaction) -> Option<TxPoolTransactionStatus> {
        self.database
            .off_chain()
            .get_tx_status(&tx.id(&self.chain_id))
            .ok()
            .flatten()
    }

    fn get_subject(
        &self,
        block: &Block,
        tx: &Transaction,
        index: usize,
    ) -> TransactionsSubject {
        let height = *block.header().consensus().height;
        let id = self.get_id(tx);
        let status = self.get_status(tx);
        TransactionsSubject {
            height: Some(height),
            tx_index: Some(index),
            tx_id: Some(id),
            status: status.map(Into::into),
            kind: Some(TransactionKind::from(tx)),
        }
    }
}
