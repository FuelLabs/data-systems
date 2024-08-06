use fuel_core::combined_database::CombinedDatabase;
use fuel_core_types::{blockchain::block::Block, fuel_types::BlockHeight};
use tokio::try_join;
use tracing::info;

use super::{nats::NatsHelper, payload::NatsPayload};

#[derive(Clone)]
pub struct BlockHelper {
    nats: NatsHelper,
    database: CombinedDatabase,
}

impl BlockHelper {
    pub fn new(nats: NatsHelper, database: &CombinedDatabase) -> Self {
        Self {
            nats,
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
        let message = NatsPayload::new(block.clone());
        try_join!(
            self.publish_core(message.clone(), block),
            self.publish_encoded(message.clone(), block),
            self.publish_to_kv(message.clone(), block)
        )?;
        Ok(())
    }
}

/// Publisher
impl BlockHelper {
    async fn publish_core(
        &self,
        mut msg: NatsPayload<Block>,
        block: &Block,
    ) -> anyhow::Result<()> {
        let subject = self.get_subject(Some("sub"), block);
        let payload = msg.with_subject(subject.clone()).serialize()?;
        self.nats.context.publish(subject, payload.into()).await?;

        Ok(())
    }
    async fn publish_encoded(
        &self,
        mut msg: NatsPayload<Block>,
        block: &Block,
    ) -> anyhow::Result<()> {
        let height = self.get_height(block);
        let subject = self.get_subject(Some("encoded"), block);
        let payload = msg.with_subject(subject.clone()).to_publish()?;

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

    async fn publish_to_kv(
        &self,
        mut msg: NatsPayload<Block>,
        block: &Block,
    ) -> anyhow::Result<()> {
        let height = self.get_height(block);
        let subject = self.get_subject(Some("kv"), block);
        let payload = msg.with_subject(subject.clone()).serialize()?;
        self.nats.kv_blocks.put(subject, payload.into()).await?;

        info!("NATS: publishing block {} to kv store \"blocks\"", height);
        Ok(())
    }
}

/// Getters
impl BlockHelper {
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
