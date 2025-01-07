use std::sync::Arc;

use fuel_streams_core::{subjects::*, types::*, FuelStreams};
use futures::stream::FuturesUnordered;
use tokio::task::JoinHandle;

use crate::*;

impl Executor<Block> {
    pub fn process(&self) -> JoinHandle<Result<(), ExecutorError>> {
        let metadata = self.metadata();
        let block = self.block();
        let block_height = (*metadata.block_height).clone();
        let block_producer = (*metadata.block_producer).clone();
        let subject = BlocksSubject {
            height: Some(block_height),
            producer: Some(block_producer),
        };
        let order = self.record_order();
        let packet = block.to_packet(subject.parse(), &order);
        self.publish(&packet)
    }

    pub fn process_all(
        payload: Arc<BlockPayload>,
        fuel_streams: &Arc<FuelStreams>,
        semaphore: &Arc<tokio::sync::Semaphore>,
    ) -> FuturesUnordered<JoinHandle<Result<(), ExecutorError>>> {
        let block_stream = fuel_streams.blocks.arc();
        let tx_stream = fuel_streams.transactions.arc();
        let input_stream = fuel_streams.inputs.arc();
        let output_stream = fuel_streams.outputs.arc();
        let receipt_stream = fuel_streams.receipts.arc();
        let log_stream = fuel_streams.logs.arc();
        let utxo_stream = fuel_streams.utxos.arc();

        let block_executor = Executor::new(&payload, &block_stream, semaphore);
        let tx_executor = Executor::new(&payload, &tx_stream, semaphore);
        let input_executor = Executor::new(&payload, &input_stream, semaphore);
        let output_executor =
            Executor::new(&payload, &output_stream, semaphore);
        let receipt_executor =
            Executor::new(&payload, &receipt_stream, semaphore);
        let log_executor = Executor::new(&payload, &log_stream, semaphore);
        let utxo_executor = Executor::new(&payload, &utxo_stream, semaphore);

        let transactions = payload.transactions.to_owned();
        let tx_tasks = transactions.iter().enumerate().flat_map(|tx_item| {
            vec![
                tx_executor.process(tx_item),
                input_executor.process(tx_item),
                output_executor.process(tx_item),
                receipt_executor.process(tx_item),
                log_executor.process(tx_item),
                utxo_executor.process(tx_item),
            ]
        });

        let block_task = block_executor.process();
        std::iter::once(block_task)
            .chain(tx_tasks.into_iter().flatten())
            .collect::<FuturesUnordered<_>>()
    }
}
