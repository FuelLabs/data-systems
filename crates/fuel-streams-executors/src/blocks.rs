use std::{sync::Arc, time::Instant};

use fuel_streams_core::prelude::*;
use futures::{future::try_join_all, stream::FuturesUnordered};
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Block> {
    pub fn process(&self) -> JoinHandle<Result<(), ExecutorError>> {
        let metadata = self.metadata();
        let block = self.block();
        let block_height = (*metadata.block_height).clone();
        let block_producer = (*metadata.block_producer).clone();
        let packet = PublishPacket::<Block>::new(
            block.to_owned(),
            BlocksSubject {
                height: Some(block_height),
                producer: Some(block_producer),
            }
            .arc(),
        );
        self.publish(&packet)
    }

    pub async fn process_all(
        payload: Arc<BlockPayload>,
        fuel_streams: &Arc<dyn FuelStreamsExt>,
    ) -> Result<(), ExecutorError> {
        let start_time = Instant::now();
        let metadata = Arc::new(payload.metadata().clone());

        let block_stream = fuel_streams.blocks().arc();
        let tx_stream = fuel_streams.transactions().arc();
        let input_stream = fuel_streams.inputs().arc();
        let output_stream = fuel_streams.outputs().arc();
        let receipt_stream = fuel_streams.receipts().arc();
        let log_stream = fuel_streams.logs().arc();
        let utxo_stream = fuel_streams.utxos().arc();

        let block_executor = Executor::new(&payload, &block_stream);
        let tx_executor = Executor::new(&payload, &tx_stream);
        let input_executor = Executor::new(&payload, &input_stream);
        let output_executor = Executor::new(&payload, &output_stream);
        let receipt_executor = Executor::new(&payload, &receipt_stream);
        let log_executor = Executor::new(&payload, &log_stream);
        let utxo_executor = Executor::new(&payload, &utxo_stream);

        let transactions = payload.transactions.to_owned();
        let tx_tasks =
            transactions
                .iter()
                .enumerate()
                .flat_map(|tx_item @ (_, tx)| {
                    vec![
                        tx_executor.process(tx_item),
                        input_executor.process(tx),
                        output_executor.process(tx),
                        receipt_executor.process(tx),
                        log_executor.process(tx),
                        utxo_executor.process(tx),
                    ]
                });

        let block_task = block_executor.process();
        let all_tasks = std::iter::once(block_task)
            .chain(tx_tasks.into_iter().flatten())
            .collect::<FuturesUnordered<_>>();

        try_join_all(all_tasks).await?;

        let elapsed = start_time.elapsed();
        let height = metadata.block_height.clone();
        tracing::info!(
            "Published streams for BlockHeight: {height} in {:?}",
            elapsed
        );

        Ok(())
    }
}
