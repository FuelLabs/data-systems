use async_nats::jetstream::context::Publish;
use fuel_core::combined_database::CombinedDatabase;
use fuel_core_types::{
    blockchain::block::Block,
    fuel_tx::{Transaction, UniqueIdentifier},
    fuel_types::ChainId,
    services::txpool::TransactionStatus,
};
use tracing::info;

use super::nats::NatsHelper;

#[derive(Clone)]
pub struct TxHelper {
    nats: NatsHelper,
    chain_id: ChainId,
    database: CombinedDatabase,
}

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
        self.publish_core(block, tx, index).await?;
        self.publish_encoded(block, tx, index).await?;
        self.publish_json(block, tx, index).await?;
        self.publish_to_kv(block, tx, index).await?;
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
        let encoded = bincode::serialize(tx)?;
        let subject = self.get_subject(None, block, tx, index);
        self.nats.context.publish(subject, encoded.into()).await?;
        Ok(())
    }

    async fn publish_encoded(
        &self,
        block: &Block,
        tx: &Transaction,
        index: usize,
    ) -> anyhow::Result<()> {
        let tx_id = self.get_id(tx);
        let encoded = bincode::serialize(tx)?;
        let subject = self.get_subject(Some("encoded"), block, tx, index);
        let payload = Publish::build()
            .message_id(&subject)
            .payload(encoded.into());

        self.nats
            .context
            .send_publish(subject, payload)
            .await?
            .await?;

        info!(
            "NATS: publishing transaction {} json to stream \"transactions_encoded\"",
            tx_id
        );
        Ok(())
    }

    async fn publish_json(
        &self,
        block: &Block,
        tx: &Transaction,
        index: usize,
    ) -> anyhow::Result<()> {
        let tx_id = self.get_id(tx);
        let tx_str = tx.to_json();
        let subject = self.get_subject(Some("json"), block, tx, index);
        let payload =
            Publish::build().message_id(&subject).payload(tx_str.into());

        self.nats
            .context
            .send_publish(subject, payload)
            .await?
            .await?;

        info!(
            "NATS: publishing transaction {} json to stream \"transactions_json\"",
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
        let encoded = bincode::serialize(tx)?;
        let subject = self.get_subject(Some("kv"), block, tx, index);
        self.nats
            .kv_transactions
            .put(subject, encoded.clone().into())
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

    fn get_kind(&self, tx: &Transaction) -> &'static str {
        match tx {
            Transaction::Create(_) => "create",
            Transaction::Mint(_) => "mint",
            Transaction::Script(_) => "script",
            Transaction::Upload(_) => "upload",
            Transaction::Upgrade(_) => "upgrade",
        }
    }

    fn get_status(&self, tx: &Transaction) -> &'static str {
        let status = self
            .database
            .off_chain()
            .get_tx_status(&tx.id(&self.chain_id))
            .unwrap();

        match status {
            Some(TransactionStatus::Failed { .. }) => "failed",
            Some(TransactionStatus::Success { .. }) => "success",
            Some(TransactionStatus::Submitted { .. }) => "submitted",
            Some(TransactionStatus::SqueezedOut { .. }) => "squeezed_out",
            None => "none",
        }
    }

    fn get_subject(
        &self,
        publish_type: Option<&'static str>,
        block: &Block,
        tx: &Transaction,
        index: usize,
    ) -> String {
        let height = *block.header().consensus().height;
        let id = self.get_id(tx);
        let kind = self.get_kind(tx);
        let status = self.get_status(tx);
        if publish_type.is_some() {
            let pt = publish_type.unwrap();
            format!(
                "transactions.{}.{}.{}.{}.{}.{}",
                pt, height, index, id, kind, status
            )
        } else {
            format!(
                "transactions.{}.{}.{}.{}.{}",
                height, index, id, kind, status
            )
        }
    }
}
