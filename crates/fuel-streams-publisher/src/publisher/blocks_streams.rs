use std::{cmp::max, sync::Arc};

use fuel_streams_core::prelude::*;
use futures::{
    stream::{self, BoxStream},
    StreamExt,
    TryStreamExt,
};
use tokio_stream::wrappers::BroadcastStream;

pub fn build_blocks_stream<'a>(
    fuel_streams: &'a Arc<dyn FuelStreamsExt>,
    fuel_core: &'a Arc<dyn FuelCoreLike>,
    max_retained_blocks: u32,
) -> BoxStream<'a, anyhow::Result<FuelCoreSealedBlock>> {
    #[derive(Debug, Default, Clone)]
    struct State {
        has_published_latest: bool,
        has_reached_new_blocks_stream: bool,
    }
    let stream_state = State::default();

    stream::try_unfold(stream_state, move |mut stream_state| {
        let fuel_core = Arc::clone(fuel_core);
        let fuel_streams = Arc::clone(fuel_streams);

        async move {
            let latest_block_height = fuel_core.get_latest_block_height()?;

            let last_published_block_height = get_last_published_block_height(
                fuel_streams,
                latest_block_height,
                max_retained_blocks,
            )
            .await?;

            stream_state.has_published_latest =
                latest_block_height == last_published_block_height;

            match stream_state {
                State {
                    has_published_latest: false,
                    has_reached_new_blocks_stream: false,
                } => {
                    let old_blocks_stream = stream::iter(
                        last_published_block_height..latest_block_height,
                    )
                    .map({
                        let fuel_core = fuel_core.clone();

                        move |height| {
                            fuel_core.get_sealed_block_by_height(height)
                        }
                    })
                    .map(Ok)
                    .boxed();

                    anyhow::Ok(Some((old_blocks_stream, stream_state.clone())))
                }
                State {
                    has_published_latest: true,
                    has_reached_new_blocks_stream: false,
                } => {
                    let new_blocks_stream =
                        BroadcastStream::new(fuel_core.blocks_subscription())
                            .map(|import_result| {
                                import_result
                                    .expect("Must get ImporterResult")
                                    .sealed_block
                                    .clone()
                            })
                            .map(Ok)
                            .boxed();

                    stream_state.has_reached_new_blocks_stream = true;
                    anyhow::Ok(Some((new_blocks_stream, stream_state.clone())))
                }
                State {
                    has_reached_new_blocks_stream: true,
                    ..
                } => anyhow::Ok(None),
            }
        }
    })
    .try_flatten()
    .boxed()
}

async fn get_last_published_block_height(
    fuel_streams: Arc<dyn FuelStreamsExt>,
    latest_block_height: u32,
    max_retained_blocks: u32,
) -> anyhow::Result<u32> {
    let max_last_published_block_height =
        max(0, latest_block_height as i32 - max_retained_blocks as i32) as u32;

    Ok(fuel_streams
        .get_last_published_block()
        .await?
        .map(|block| block.height)
        .map(|block_height: u32| {
            max(block_height, max_last_published_block_height)
        })
        .unwrap_or(max_last_published_block_height))
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    // TODO: Fix this leaky abstraction
    use async_nats::{
        jetstream::stream::State as StreamState,
        RequestErrorKind,
    };
    use fuel_core::combined_database::CombinedDatabase;
    use futures::StreamExt;
    use mockall::{
        mock,
        predicate::{self, *},
    };
    use tokio::{
        sync::broadcast,
        time::{error::Elapsed, timeout},
    };

    use super::*;

    #[tokio::test]
    async fn test_no_old_blocks() {
        let mut mock_fuel_core = MockFuelCoreLike::new();
        let mut mock_fuel_streams = MockFuelStreams::default();

        mock_fuel_core
            .expect_get_latest_block_height()
            .returning(|| Ok(100));
        mock_fuel_streams
            .expect_get_last_published_block()
            .returning(|| Ok(Some(create_mock_block(100)))); // No old blocks

        mock_fuel_core
            .expect_blocks_subscription()
            .returning(move || {
                let (empty_tx, rx) = broadcast::channel(1);
                drop(empty_tx);
                rx
            });

        let fuel_core: Arc<dyn FuelCoreLike> = Arc::new(mock_fuel_core);
        let fuel_streams: Arc<dyn FuelStreamsExt> = Arc::new(mock_fuel_streams);

        let mut stream = build_blocks_stream(&fuel_streams, &fuel_core, 10);

        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_old_blocks_stream() {
        let mut mock_fuel_core = MockFuelCoreLike::new();
        let mut mock_fuel_streams = MockFuelStreams::default();

        mock_fuel_core
            .expect_get_latest_block_height()
            .returning(|| Ok(105));
        mock_fuel_streams
            .expect_get_last_published_block()
            .returning(|| Ok(Some(create_mock_block(100))));
        for height in 100..105 {
            mock_fuel_core
                .expect_get_sealed_block_by_height()
                .with(predicate::eq(height as u32))
                .returning(move |height| {
                    create_mock_fuel_core_sealed_block(height as u64)
                });
        }

        let fuel_core: Arc<dyn FuelCoreLike> = Arc::new(mock_fuel_core);
        let fuel_streams: Arc<dyn FuelStreamsExt> = Arc::new(mock_fuel_streams);

        let mut stream = build_blocks_stream(&fuel_streams, &fuel_core, 10);

        for height in 100..105 {
            let block = stream.next().await.unwrap().unwrap();
            assert_eq!(block.entity.header().consensus().height, height.into());
        }
    }

    #[tokio::test]
    async fn test_infinite_new_blocks_streams() {
        let mut mock_fuel_core = MockFuelCoreLike::new();
        let mut mock_fuel_streams = MockFuelStreams::default();

        mock_fuel_core
            .expect_get_latest_block_height()
            .returning(|| Ok(100));
        mock_fuel_streams
            .expect_get_last_published_block()
            .returning(|| Ok(Some(create_mock_block(100)))); // has published latest block already

        let (tx, _) = broadcast::channel(4);

        mock_fuel_core
            .expect_blocks_subscription()
            .returning(move || tx.clone().subscribe());

        let fuel_core: Arc<dyn FuelCoreLike> = Arc::new(mock_fuel_core);
        let fuel_streams: Arc<dyn FuelStreamsExt> = Arc::new(mock_fuel_streams);

        let mut blocks_stream =
            build_blocks_stream(&fuel_streams, &fuel_core, 10);

        assert_matches!(
            timeout(Duration::from_secs(1), async {
                blocks_stream.next().await
            })
            .await,
            Err(Elapsed { .. })
        );
    }

    #[tokio::test]
    async fn test_new_blocks_streams_that_ends() {
        let mut mock_fuel_core = MockFuelCoreLike::new();
        let mut mock_fuel_streams = MockFuelStreams::default();

        mock_fuel_core
            .expect_get_latest_block_height()
            .returning(|| Ok(100));
        mock_fuel_streams
            .expect_get_last_published_block()
            .returning(|| Ok(Some(create_mock_block(100)))); // has published latest block already

        let (tx, _) = broadcast::channel(4);

        mock_fuel_core
            .expect_blocks_subscription()
            .returning(move || {
                let tx = tx.clone();
                let subscription = tx.subscribe();

                tx.send(create_mock_importer_result(101)).ok();
                tx.send(create_mock_importer_result(102)).ok();

                subscription
            });

        let fuel_core: Arc<dyn FuelCoreLike> = Arc::new(mock_fuel_core);
        let fuel_streams: Arc<dyn FuelStreamsExt> = Arc::new(mock_fuel_streams);

        let mut stream = build_blocks_stream(&fuel_streams, &fuel_core, 10);

        for height in 101..=102 {
            let block = stream.next().await.unwrap().unwrap();
            assert_eq!(block.entity.header().consensus().height, height.into());
        }
    }

    #[tokio::test]
    async fn test_get_last_published_block_height() {
        let mut mock_fuel_streams = MockFuelStreams::default();

        // Case 1: `get_last_published_block` returns Some(block)
        mock_fuel_streams
            .expect_get_last_published_block()
            .returning(|| Ok(Some(create_mock_block(50))));

        let fuel_streams = Arc::new(mock_fuel_streams);

        let result =
            get_last_published_block_height(fuel_streams.clone(), 100, 40)
                .await
                .unwrap();
        assert_eq!(result, 60); // max(50, max_last_published_block_height=60)

        // Case 2: `get_last_published_block` returns None
        let mut mock_fuel_streams = MockFuelStreams::default();
        mock_fuel_streams
            .expect_get_last_published_block()
            .returning(|| Ok(None));

        let fuel_streams = Arc::new(mock_fuel_streams);

        let result =
            get_last_published_block_height(fuel_streams.clone(), 100, 40)
                .await
                .unwrap();
        assert_eq!(result, 60); // No block, fallback to max_last_published_block_height

        // Case 3: `get_last_published_block` returns an error
        let mut mock_fuel_streams = MockFuelStreams::default();
        mock_fuel_streams
            .expect_get_last_published_block()
            .returning(|| Err(anyhow::anyhow!("Error fetching block")));

        let fuel_streams = Arc::new(mock_fuel_streams);

        let result =
            get_last_published_block_height(fuel_streams.clone(), 100, 40)
                .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Error fetching block");

        // Case 4: `get_last_published_block` returns Some(block) where block.height < max_last_published_block_height
        let mut mock_fuel_streams = MockFuelStreams::default();
        mock_fuel_streams
            .expect_get_last_published_block()
            .returning(|| Ok(Some(create_mock_block(30))));

        let fuel_streams = Arc::new(mock_fuel_streams);

        let result =
            get_last_published_block_height(fuel_streams.clone(), 100, 40)
                .await
                .unwrap();
        assert_eq!(result, 60); // max(30, max_last_published_block_height=60)
    }

    mock! {
        FuelCoreLike {}

        #[async_trait::async_trait]
        impl FuelCoreLike for FuelCoreLike {
            fn get_latest_block_height(&self) -> anyhow::Result<u32>;
            fn get_sealed_block_by_height(&self, height: u32) -> FuelCoreSealedBlock;
            fn blocks_subscription(&self) -> broadcast::Receiver<FuelCoreImporterResult>;
            async fn start(&self) -> anyhow::Result<()>;
            fn is_started(&self) -> bool;
            async fn await_synced_at_least_once(&self, historical: bool) -> anyhow::Result<()>;
            async fn stop(&self);
            fn base_asset_id(&self) -> &FuelCoreAssetId;
            fn chain_id(&self) -> &FuelCoreChainId;
            fn database(&self) -> &CombinedDatabase;
            async fn await_offchain_db_sync(
                &self,
                block_id: &FuelCoreBlockId,
            ) -> anyhow::Result<()>;
            fn get_receipts(
                &self,
                tx_id: &FuelCoreBytes32,
            ) -> anyhow::Result<Option<Vec<FuelCoreReceipt>>>;
        }
    }

    mock! {
        FuelStreams {}

        #[async_trait::async_trait]
        impl FuelStreamsExt for FuelStreams {
            async fn get_last_published_block(&self) -> anyhow::Result<Option<Block>>;
            fn blocks(&self) -> &Stream<Block>;
            fn transactions(&self) -> &Stream<Transaction>;
            fn inputs(&self) -> &Stream<Input>;
            fn outputs(&self) -> &Stream<Output>;
            fn receipts(&self) -> &Stream<Receipt>;
            fn utxos(&self) -> &Stream<Utxo>;
            fn logs(&self) -> &Stream<Log>;
            async fn get_consumers_and_state(
                &self,
            ) -> Result<Vec<(String, Vec<String>, StreamState)>, RequestErrorKind> ;
            #[cfg(feature = "test-helpers")]
            async fn is_empty(&self) -> bool;
        }
    }

    fn create_mock_importer_result(height: u64) -> FuelCoreImporterResult {
        FuelCoreImporterResult {
            shared_result: Arc::new(FuelCoreImportResult {
                sealed_block: create_mock_fuel_core_sealed_block(height),
                ..Default::default()
            }),
            #[cfg(feature = "test-helpers")]
            changes: Arc::new(std::collections::HashMap::new()),
        }
    }

    fn create_mock_block(height: u64) -> Block {
        Block::new(
            &create_mock_fuel_core_sealed_block(height).entity,
            FuelCoreConsensus::default().into(),
            vec![],
        )
    }

    fn create_mock_fuel_core_sealed_block(height: u64) -> FuelCoreSealedBlock {
        let mut block = FuelCoreSealedBlock::default();

        block.entity.header_mut().consensus_mut().height =
            FuelCoreBlockHeight::new(height as u32);

        block
    }
}
