use async_nats::jetstream::context::Publish;
use fuel_core::combined_database::CombinedDatabase;
use fuel_core_types::fuel_types::ChainId;
use fuel_streams_core::prelude::*;
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
        let payload = self.nats.data_parser().encode(block).await?;
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
        let tx_id = &tx.id;
        let subject = self.get_subject(tx, block, index);
        let payload = self.nats.data_parser().encode(block).await?;
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
        let tx_id = &tx.id;
        let subject = self.get_subject(tx, block, index);
        let payload = self.nats.data_parser().encode(block).await?;
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
            .with_index(Some(index))
            .with_block_height(Some(BlockHeight::from(block.height)))
            .with_status(Some(tx.status.clone()));
        subject
    }
}
