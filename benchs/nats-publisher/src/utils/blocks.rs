use async_nats::jetstream::context::Publish;
use fuel_core::combined_database::CombinedDatabase;
use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::{blockchain::block::Block, fuel_types::BlockHeight};
use fuel_streams_core::{blocks::BlocksSubject, prelude::IntoSubject};
use tokio::try_join;
use tracing::info;

use super::nats::NatsHelper;

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
            .latest_view()
            .unwrap()
            .get_sealed_block_by_height(&height)
            .unwrap()
            .unwrap_or_else(|| {
                panic!("NATS Publisher: no block at height {height}")
            })
            .entity
    }

    pub async fn publish(&self, block: &Block) -> anyhow::Result<()> {
        try_join!(
            self.publish_core(block),
            self.publish_encoded(block),
            self.publish_to_kv(block)
        )?;
        Ok(())
    }
}

/// Publisher
impl BlockHelper {
    async fn publish_core(&self, block: &Block) -> anyhow::Result<()> {
        let subject: BlocksSubject = block.into();
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
    async fn publish_encoded(&self, block: &Block) -> anyhow::Result<()> {
        let height = self.get_height(block);
        let subject: BlocksSubject = block.into();
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
            "NATS: publishing block {} encoded to stream \"blocks_encoded\"",
            height
        );
        Ok(())
    }

    async fn publish_to_kv(&self, block: &Block) -> anyhow::Result<()> {
        let height = self.get_height(block);
        let subject: BlocksSubject = block.into();

        let payload = self
            .nats
            .data_parser()
            .to_nats_payload(&subject.parse(), block)
            .await?;
        self.nats
            .kv_blocks
            .put(subject.parse(), payload.into())
            .await?;

        info!("NATS: publishing block {} to kv store \"blocks\"", height);
        Ok(())
    }
}

/// Getters
impl BlockHelper {
    fn get_height(&self, block: &Block) -> u32 {
        *block.header().consensus().height
    }
}
