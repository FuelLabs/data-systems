use fuel_core::{
    combined_database::CombinedDatabase,
    database::database_description::DatabaseHeight,
};
use fuel_core_storage::transactional::AtomicView;
use fuel_streams_core::{
    blocks::BlocksSubject,
    nats::{IntoSubject, NatsClient, NatsClientOpts},
    types::{AssetId, Block, BlockHeight, ChainId, Transaction},
    Stream, Streamable,
};
use tokio::sync::broadcast::Receiver;
use tracing::warn;

use crate::{blocks, transactions};

/// Streams we currently support publishing to.
struct Streams {
    transactions: Stream<Transaction>,
    blocks: Stream<Block>,
}

impl Streams {
    pub async fn new(nats_client: &NatsClient) -> anyhow::Result<Self> {
        Ok(Self {
            transactions: Transaction::create_stream(nats_client).await?,
            blocks: Block::create_stream(nats_client).await?,
        })
    }

    #[cfg(test)]
    pub async fn is_empty(&self) -> bool {
        use fuel_streams_core::transactions::TransactionsSubject;

        self.blocks.is_empty(BlocksSubject::WILDCARD).await
            && self
                .transactions
                .is_empty(TransactionsSubject::WILDCARD)
                .await
    }
}

#[allow(dead_code)]
/// TODO: Remove right after using chain_id and base_asset_id to publish
/// TransactionsById subject
pub struct Publisher {
    chain_id: ChainId,
    base_asset_id: AssetId,
    fuel_core_database: CombinedDatabase,
    blocks_subscription: Receiver<fuel_core_importer::ImporterResult>,
    streams: Streams,
}

impl Publisher {
    pub async fn new(
        nats_url: &str,
        chain_id: ChainId,
        base_asset_id: AssetId,
        fuel_core_database: CombinedDatabase,
        blocks_subscription: Receiver<fuel_core_importer::ImporterResult>,
    ) -> anyhow::Result<Self> {
        let nats_client_opts = NatsClientOpts::admin_opts(nats_url);
        let nats_client = NatsClient::connect(&nats_client_opts).await?;

        Ok(Publisher {
            chain_id,
            base_asset_id,
            fuel_core_database,
            blocks_subscription,
            streams: Streams::new(&nats_client).await?,
        })
    }

    pub async fn run(mut self) -> anyhow::Result<Self> {
        let last_published_block = self
            .streams
            .blocks
            .get_last_published(BlocksSubject::WILDCARD)
            .await?;
        let last_published_height = last_published_block
            .map(|block| block.header().height().as_u64())
            .unwrap_or(0);
        let next_height_to_publish = last_published_height + 1;

        // Catch up the streams with the FuelCore
        if let Some(latest_fuel_core_height) = self
            .fuel_core_database
            .on_chain()
            .latest_height()?
            .map(|h| h.as_u64())
        {
            if latest_fuel_core_height > last_published_height + 1 {
                warn!("Missing blocks: last block height in Node={latest_fuel_core_height}, last published block height={last_published_height}");
            }

            for height in next_height_to_publish..=latest_fuel_core_height {
                let block = self
                    .fuel_core_database
                    .on_chain()
                    .latest_view()?
                    .get_sealed_block_by_height(&(height as u32).into())?
                    .expect("NATS Publisher: no block at height {height}")
                    .entity;

                self.publish(&block).await?;
            }
        }

        while let Ok(result) = self.blocks_subscription.recv().await {
            let block = &result.sealed_block.entity;
            self.publish(block).await?;
        }

        Ok(self)
    }

    async fn publish(&self, block: &Block<Transaction>) -> anyhow::Result<()> {
        let block_height: BlockHeight =
            block.header().consensus().height.into();

        blocks::publish(&block_height, &self.streams.blocks, block).await?;

        transactions::publish(
            &self.chain_id,
            &block_height,
            &self.fuel_core_database,
            &self.streams.transactions,
            block.transactions(),
        )
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use fuel_core::combined_database::CombinedDatabase;
    use fuel_core_importer::ImporterResult;
    use fuel_core_types::blockchain::SealedBlock;
    use fuel_streams_core::{
        transactions::TransactionsSubject, types::ImportResult,
    };
    use tokio::sync::broadcast;

    use super::*;

    #[tokio::test]
    async fn doesnt_publish_any_message_when_no_block_has_been_mined() {
        let (_, blocks_subscription) = broadcast::channel::<ImporterResult>(1);

        let publisher = Publisher {
            base_asset_id: AssetId::default(),
            chain_id: ChainId::default(),
            fuel_core_database: CombinedDatabase::default(),
            blocks_subscription,
            streams: streams().await,
        };
        let publisher = publisher.run().await.unwrap();

        assert!(publisher.streams.is_empty().await);
    }

    #[tokio::test]
    async fn publishes_a_block_message_when_a_single_block_has_been_mined() {
        let (blocks_subscriber, blocks_subscription) =
            broadcast::channel::<ImporterResult>(1);

        let block = ImporterResult {
            shared_result: Arc::new(ImportResult::default()),
            changes: Arc::new(HashMap::new()),
        };
        let _ = blocks_subscriber.send(block);

        // manually drop blocks to ensure `blocks_subscription` completes
        let _ = blocks_subscriber.clone();
        drop(blocks_subscriber);

        let publisher = Publisher {
            base_asset_id: AssetId::default(),
            chain_id: ChainId::default(),
            fuel_core_database: CombinedDatabase::default(),
            blocks_subscription,
            streams: streams().await,
        };

        let publisher = publisher.run().await.unwrap();

        assert!(publisher
            .streams
            .blocks
            .get_last_published(BlocksSubject::WILDCARD)
            .await
            .is_ok_and(|result| result.is_some()));
    }

    #[tokio::test]
    async fn publishes_transaction_for_each_published_block() {
        let (blocks_subscriber, blocks_subscription) =
            broadcast::channel::<ImporterResult>(1);

        let mut block_entity = Block::default();
        *block_entity.transactions_mut() = vec![Transaction::default_test_tx()];

        // publish block
        let block = ImporterResult {
            shared_result: Arc::new(ImportResult {
                sealed_block: SealedBlock {
                    entity: block_entity,
                    ..Default::default()
                },
                ..Default::default()
            }),
            changes: Arc::new(HashMap::new()),
        };
        let _ = blocks_subscriber.send(block);

        // manually drop blocks to ensure `blocks_subscription` completes
        let _ = blocks_subscriber.clone();
        drop(blocks_subscriber);

        let publisher = Publisher {
            base_asset_id: AssetId::default(),
            chain_id: ChainId::default(),
            fuel_core_database: CombinedDatabase::default(),
            blocks_subscription,
            streams: streams().await,
        };

        let publisher = publisher.run().await.unwrap();

        assert!(publisher
            .streams
            .transactions
            .get_last_published(TransactionsSubject::WILDCARD)
            .await
            .is_ok_and(|result| result.is_some()));
    }

    async fn streams() -> Streams {
        Streams::new(&nats_client().await)
            .await
            .expect("Streams creation failed")
    }

    async fn nats_client() -> NatsClient {
        const NATS_URL: &str = "nats://localhost:4222";
        let nats_client_opts =
            NatsClientOpts::admin_opts(NATS_URL).with_rdn_namespace();
        NatsClient::connect(&nats_client_opts)
            .await
            .expect("NATS connection failed")
    }
}
