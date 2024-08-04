use async_nats::jetstream::context::Publish;
use fuel_core::combined_database::CombinedDatabase;
use fuel_core_types::{
    blockchain::block::Block,
    fuel_types::{BlockHeight, ChainId},
};
use tracing::info;

use super::nats::NatsHelper;

#[derive(Clone)]
pub struct BlockHelper {
    nats: NatsHelper,
    chain_id: ChainId,
    database: CombinedDatabase,
}

impl BlockHelper {
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

    pub fn find_by_height(&self, height: BlockHeight) -> Block {
        self.database
            .on_chain()
            .get_sealed_block_by_height(&height)
            .unwrap()
            .unwrap_or_else(|| {
                panic!("NATS Publisher: no block at height {height}")
            })
            .entity
    }

    pub async fn publish(&self, block: &Block) -> anyhow::Result<()> {
        self.publish_encoded(block).await?;
        self.publish_json(block).await?;
        self.publish_to_kv(block).await?;
        Ok(())
    }
}

/// Publisher
impl BlockHelper {
    async fn publish_encoded(&self, block: &Block) -> anyhow::Result<()> {
        let encoded = self.encode_block(block)?;
        let height = self.get_height(block);
        let subject = self.get_subject(Some("encoded"), block);
        let payload = Publish::build()
            .message_id(&subject)
            .payload(encoded.into());

        self.nats
            .context
            .send_publish(subject, payload)
            .await?
            .await?;

        info!(
            "NATS: publishing block {} encoded to stream \"blocks_encoded\"",
            height
        );
        Ok(())
    }

    async fn publish_json(&self, block: &Block) -> anyhow::Result<()> {
        let block_str = serde_json::to_string(block)?;
        let height = self.get_height(block);
        let subject = self.get_subject(Some("json"), block);
        let payload = Publish::build()
            .message_id(subject.clone())
            .payload(block_str.into());

        self.nats
            .context
            .send_publish(subject, payload)
            .await?
            .await?;

        info!(
            "NATS: publishing block {} json to stream \"blocks_json\"",
            height
        );
        Ok(())
    }

    async fn publish_to_kv(&self, block: &Block) -> anyhow::Result<()> {
        let encoded = self.encode_block(block)?;
        let height = self.get_height(block);
        let subject = self.get_subject(None, block);
        self.nats
            .kv_blocks
            .put(subject, encoded.clone().into())
            .await?;

        info!("NATS: publishing block {} to kv store \"blocks\"", height);
        Ok(())
    }
}

/// Getters
impl BlockHelper {
    fn encode_block(&self, block: &Block) -> Result<Vec<u8>, bincode::Error> {
        let compressed = block.compress(&self.chain_id);
        bincode::serialize(&compressed)
    }

    fn get_height(&self, block: &Block) -> u32 {
        *block.header().consensus().height
    }

    fn get_subject(
        &self,
        publish_type: Option<&'static str>,
        block: &Block,
    ) -> String {
        let height = self.get_height(block);
        if publish_type.is_some() {
            let pt = publish_type.unwrap();
            format!("blocks.{}.{}", pt, height)
        } else {
            format!("blocks.{}", height)
        }
    }
}
