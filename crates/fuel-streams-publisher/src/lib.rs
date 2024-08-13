use fuel_core::combined_database::CombinedDatabase;
use fuel_core_importer::ImporterResult;
use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::{
    blockchain::block::Block,
    fuel_tx::{Receipt, Transaction, UniqueIdentifier},
    fuel_types::{AssetId, ChainId},
};
use fuel_data_parser::DataParser;
use fuel_streams_core::{
    blocks::BlocksSubject,
    nats::{
        types::NatsStoreConfig,
        ClientOpts,
        IntoSubject,
        NatsClient,
        NatsNamespace,
        NatsUserRole,
        Store,
    },
    transactions::TransactionsSubject,
    types::{Block as StoreBlock, Transaction as StoreTransaction},
};
use futures_util::TryStreamExt;
use tokio::sync::broadcast::Receiver;
use tracing::{info, warn};

pub struct Publisher {
    chain_id: ChainId,
    base_asset_id: AssetId,
    fuel_core_database: CombinedDatabase,
    blocks_subscription: Receiver<ImporterResult>,
    nats: NatsClient,
}

impl Publisher {
    pub async fn new(
        nats_url: &str,
        chain_id: ChainId,
        base_asset_id: AssetId,
        fuel_core_database: CombinedDatabase,
        blocks_subscription: Receiver<ImporterResult>,
    ) -> anyhow::Result<Self> {
        let nats = NatsClient::connect(
            ClientOpts::new(nats_url).with_role(NatsUserRole::Admin),
        )
        .await?;

        Ok(Publisher {
            chain_id,
            base_asset_id,
            fuel_core_database,
            blocks_subscription,
            nats,
        })
    }

    /// Publish messages from node(`fuel-core`) to NATS stream
    ///   transactions.{height}.{index}.{kind}                           e.g. transactions.1.1.mint
    ///   blocks.{height}                                                e.g. blocks.1
    pub async fn run(mut self) -> anyhow::Result<Self> {
        info!(
            "NATS Publisher chain_id={} base_asset_id={} started",
            self.chain_id, self.base_asset_id
        );

        // Check the last block height in the stream
        let (stream_height, fuel_block_store, fuel_tx_store) = {
            let config = async_nats::jetstream::consumer::pull::Config {
                deliver_policy:
                    async_nats::jetstream::consumer::DeliverPolicy::Last,
                filter_subject: BlocksSubject::WILDCARD.to_owned(),
                ..Default::default()
            };
            let store = self
                .nats
                .create_store("pub_store", Some(NatsStoreConfig::default()))
                .await?;

            let fuel_block_store: Store<StoreBlock> = Store::new(
                store.clone(),
                &NatsNamespace::Fuel,
                &DataParser::default(),
            );
            let fuel_tx_store: Store<StoreTransaction> =
                Store::new(store, &NatsNamespace::Fuel, &DataParser::default());

            let mut batch = fuel_block_store
                .create_consumer(config)
                .await?
                .fetch()
                .max_messages(1)
                .messages()
                .await?;

            if let Ok(Some(message)) = batch.try_next().await {
                let block_height: u32 =
                    message.subject.strip_prefix("blocks.").unwrap().parse()?;
                (block_height, fuel_block_store, fuel_tx_store)
            } else {
                (0, fuel_block_store, fuel_tx_store)
            }
        };

        // Fast-forward the stream using the local Fuel node database
        if let Some(chain_height) =
            self.fuel_core_database.on_chain().latest_height()?
        {
            let chain_height: u32 = chain_height.into();
            if chain_height > stream_height + 1 {
                warn!("NATS Publisher: missing blocks: stream block height={stream_height}, chain block height={chain_height}");
            }

            for height in stream_height + 1..=chain_height {
                let block: Block = self
                    .fuel_core_database
                    .on_chain()
                    .latest_view()?
                    .get_sealed_block_by_height(&height.into())?
                    .unwrap_or_else(|| {
                        panic!("NATS Publisher: no block at height {height}")
                    })
                    .entity;

                use fuel_core_types::services::txpool::TransactionStatus;
                let mut receipts_: Vec<Receipt> = vec![];
                let chain_id = self.chain_id;

                for t in block.transactions().iter() {
                    let status: Option<TransactionStatus> = self
                        .fuel_core_database
                        .off_chain()
                        .latest_view()?
                        .get_tx_status(&t.id(&chain_id))?;

                    match status {
                        Some(TransactionStatus::Failed {
                            mut receipts,
                            ..
                        }) => {
                            receipts_.append(&mut receipts);
                        }
                        Some(TransactionStatus::Success {
                            mut receipts,
                            ..
                        }) => {
                            receipts_.append(&mut receipts);
                        }
                        Some(TransactionStatus::Submitted { .. }) => (),
                        Some(TransactionStatus::SqueezedOut { .. }) => (),
                        // TODO: check that we'd get the same result from the block importer subscription
                        None => (),
                    }
                }

                let height: u32 = **block.header().height();

                info!(
                "NATS Publisher: publishing block {height} / {chain_height} with {} receipts",
                receipts_.len()
            );

                self.publish_block(&fuel_block_store, &fuel_tx_store, &block)
                    .await?;
            }
        }

        // Continue publishing blocks from the block importer subscription
        while let Ok(result) = self.blocks_subscription.recv().await {
            let result = &**result;
            self.publish_block(
                &fuel_block_store,
                &fuel_tx_store,
                &result.sealed_block.entity,
            )
            .await?;
        }

        Ok(self)
    }

    /// Publish the Block, its Transactions, and the given Receipts into NATS.
    async fn publish_block(
        &self,
        fuel_block_store: &Store<StoreBlock>,
        fuel_tx_store: &Store<StoreTransaction>,
        block: &Block<Transaction>,
    ) -> anyhow::Result<()> {
        let chain_id = self.chain_id;
        let height: u32 = *block.header().consensus().height;

        // Publish the block.
        info!("NATS Publisher: Block#{height}");

        let block_subject: BlocksSubject = block.into();

        if let Err(e) = fuel_block_store.upsert(&block_subject, block).await {
            panic!("Failed to publish block: {}", e);
        }

        for tx in block.transactions().iter() {
            // Publish the transaction.
            let tx_id = tx.id(&chain_id);

            let tx_subject: TransactionsSubject = tx.into();

            let _ = fuel_tx_store.upsert(&tx_subject, tx).await?;
            info!("NATS Publisher: Transaction 0x#{tx_id}");
        }

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
