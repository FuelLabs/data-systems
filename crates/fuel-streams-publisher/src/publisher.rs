use fuel_core::{
    combined_database::CombinedDatabase,
    database::database_description::DatabaseHeight,
};
use fuel_core_storage::transactional::AtomicView;
use fuel_streams_core::{
    blocks::BlocksSubject,
    nats::{IntoSubject, NatsClient, NatsClientOpts},
    types::{AssetId, Block, ChainId, Transaction},
    Stream,
    Streamable,
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

    /// Publish messages from node(`fuel-core`) to NATS stream
    ///   transactions.{height}.{index}.{kind}                           e.g. transactions.1.1.mint
    ///   blocks.{height}                                                e.g. blocks.1
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
        blocks::publish(&self.streams.blocks, block).await?;
        transactions::publish(&self.streams.transactions, block.transactions())
            .await?;

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use async_nats::jetstream::stream::LastRawMessageErrorKind;
//     use fuel_core::combined_database::CombinedDatabase;
//     use fuel_core_types::blockchain::SealedBlock;
//     use nats::SubjectName;
//     use strum::IntoEnumIterator;
//     use tokio::sync::broadcast;
//
//     use super::*;
//
//     #[tokio::test]
//     async fn doesnt_publish_any_message_when_no_block_has_been_mined() {
//         let (_, blocks_subscription) = broadcast::channel::<
//             Arc<dyn Deref<Target = ImportResult> + Send + Sync>,
//         >(1);
//
//         let connection_id = nats::tests::get_random_connection_id();
//         let publisher = Publisher {
//             base_asset_id: AssetId::default(),
//             chain_id: ChainId::default(),
//             fuel_core_database: CombinedDatabase::default(),
//             blocks_subscription,
//             nats: nats::tests::get_nats_connection(&connection_id).await,
//         };
//         let publisher = publisher.run().await.unwrap();
//
//         assert!(publisher.nats.has_no_message().await);
//     }
//
//     #[tokio::test]
//     async fn publishes_a_block_message_when_a_single_block_has_been_mined() {
//         let (blocks_subscriber, blocks_subscription) = broadcast::channel::<
//             Arc<dyn Deref<Target = ImportResult> + Send + Sync>,
//         >(1);
//         let block = Arc::new(ImportResult::default());
//         let _ = blocks_subscriber.send(block);
//
//         // manually drop blocks to ensure `blocks_subscription` completes
//         let _ = blocks_subscriber.clone();
//         drop(blocks_subscriber);
//
//         let connection_id = nats::tests::get_random_connection_id();
//         let publisher = Publisher {
//             base_asset_id: AssetId::default(),
//             chain_id: ChainId::default(),
//             fuel_core_database: CombinedDatabase::default(),
//             blocks_subscription,
//             nats: nats::tests::get_nats_connection(&connection_id).await,
//         };
//
//         let publisher = publisher.run().await.unwrap();
//
//         assert!(publisher
//             .nats
//             .jetstream_messages
//             .get_last_raw_message_by_subject(
//                 &SubjectName::Blocks.get_string(&connection_id)
//             )
//             .await
//             .is_ok_and(|raw_message| raw_message.sequence == 1));
//     }
//
//     #[tokio::test]
//     async fn doesnt_publish_any_other_message_for_blocks_with_no_transactions()
//     {
//         let (blocks_subscriber, blocks_subscription) = broadcast::channel::<
//             Arc<dyn Deref<Target = ImportResult> + Send + Sync>,
//         >(1);
//         let block = Arc::new(ImportResult::default());
//         let _ = blocks_subscriber.send(block);
//
//         // manually drop blocks to ensure `blocks_subscription` completes
//         let _ = blocks_subscriber.clone();
//         drop(blocks_subscriber);
//
//         let connection_id = nats::tests::get_random_connection_id();
//         let publisher = Publisher {
//             base_asset_id: AssetId::default(),
//             chain_id: ChainId::default(),
//             fuel_core_database: CombinedDatabase::default(),
//             blocks_subscription,
//             nats: nats::tests::get_nats_connection(&connection_id).await,
//         };
//         let publisher = publisher.run().await.unwrap();
//
//         let non_block_subjects_count = nats::SubjectName::iter().len() - 1;
//
//         let raw_messages_by_all_subjects =
//             publisher.nats.get_last_raw_messages_by_all_subjects().await;
//         let last_non_block_subjects =
//             raw_messages_by_all_subjects.iter().filter(|result| {
//                 result.as_ref().is_err_and(|e| {
//                     e.kind() == LastRawMessageErrorKind::NoMessageFound
//                 })
//             });
//
//         assert!(non_block_subjects_count == last_non_block_subjects.count());
//
//         assert!(publisher
//             .nats
//             .jetstream_messages
//             .get_last_raw_message_by_subject(
//                 &SubjectName::Blocks.get_string(&connection_id)
//             )
//             .await
//             .is_ok_and(|raw_message| raw_message.sequence == 1));
//     }
//
//     #[tokio::test]
//     async fn publishes_transactions_for_each_published_block() {
//         let (blocks_subscriber, blocks_subscription) = broadcast::channel::<
//             Arc<dyn Deref<Target = ImportResult> + Send + Sync>,
//         >(1);
//
//         let mut block_entity = Block::default();
//         *block_entity.transactions_mut() = vec![Transaction::default_test_tx()];
//
//         let block = Arc::new(ImportResult {
//             sealed_block: SealedBlock {
//                 entity: block_entity,
//                 ..Default::default()
//             },
//             ..Default::default()
//         });
//         let _ = blocks_subscriber.send(block);
//
//         // manually drop blocks to ensure `blocks_subscription` completes
//         let _ = blocks_subscriber.clone();
//         drop(blocks_subscriber);
//
//         let connection_id = nats::tests::get_random_connection_id();
//         let publisher = Publisher {
//             base_asset_id: AssetId::default(),
//             chain_id: ChainId::default(),
//             fuel_core_database: CombinedDatabase::default(),
//             blocks_subscription,
//             nats: nats::tests::get_nats_connection(&connection_id).await,
//         };
//
//         let publisher = publisher.run().await.unwrap();
//
//         assert!(publisher
//             .nats
//             .jetstream_messages
//             .get_last_raw_message_by_subject(
//                 &SubjectName::Transactions.get_string(&connection_id)
//             )
//             .await
//             .is_ok());
//     }
// }
