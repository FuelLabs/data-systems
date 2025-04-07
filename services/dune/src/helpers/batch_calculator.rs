use apache_avro::{schema::derive::AvroSchemaComponent, AvroSchema};
use fuel_streams_domains::{blocks::Block, transactions::Transaction};
use fuel_streams_types::BlockHeight;
use rayon::prelude::*;

use crate::{
    helpers::AvroParser,
    schemas::{AvroBlock, AvroReceipt, AvroTransaction},
};

#[derive(Debug, Clone)]
pub struct BatchCalculator {
    pub max_file_size: usize,
}

impl BatchCalculator {
    pub fn new(max_file_size: usize) -> Self {
        Self { max_file_size }
    }

    pub fn calculate_batches<T, F>(
        &self,
        blocks: &[(Block, Vec<Transaction>)],
        extractor: F,
    ) -> Vec<(BlockHeight, BlockHeight)>
    where
        T: serde::Serialize
            + AvroSchemaComponent
            + AvroSchema
            + Send
            + Sync
            + 'static,
        F: Fn(&Block, &Vec<Transaction>) -> Vec<T> + Sync,
    {
        if blocks.is_empty() {
            return Vec::new();
        }

        let sizes: Vec<_> = blocks
            .par_iter()
            .map(|(block, transactions)| {
                let items = extractor(block, transactions);
                let size = items
                    .iter()
                    .filter_map(|item| {
                        let mut avro_writer = AvroParser::default()
                            .writer_with_schema::<T>()
                            .expect("Failed to create Avro writer");
                        avro_writer.append(item).unwrap();
                        let serialized = avro_writer.into_inner().ok();
                        serialized.map(|serialized| serialized.len())
                    })
                    .sum::<usize>();
                (block.height, size)
            })
            .collect();

        let mut batches = Vec::new();
        let mut current_size = 0;
        let mut first_height = None;
        let mut last_height = None;

        for (height, size) in sizes {
            if first_height.is_none() {
                first_height = Some(height);
            }

            if current_size + size > self.max_file_size && last_height.is_some()
            {
                batches.push((first_height.unwrap(), last_height.unwrap()));
                first_height = Some(height);
                current_size = size;
                last_height = Some(height);
            } else {
                current_size += size;
                last_height = Some(height);
            }
        }

        if let (Some(first), Some(last)) = (first_height, last_height) {
            if current_size > 0 {
                batches.push((first, last));
            }
        }
        batches
    }

    pub fn calculate_blocks_batches(
        &self,
        blocks: &[(Block, Vec<Transaction>)],
    ) -> Vec<(BlockHeight, BlockHeight)> {
        self.calculate_batches(blocks, |block, transactions| {
            let avro_transactions = transactions
                .iter()
                .map(|tx| AvroTransaction::from((block, tx)))
                .collect();
            vec![AvroBlock::new(block, avro_transactions)]
        })
    }

    pub fn calculate_txs_batches(
        &self,
        blocks: &[(Block, Vec<Transaction>)],
    ) -> Vec<(BlockHeight, BlockHeight)> {
        self.calculate_batches(blocks, |block, transactions| {
            transactions
                .iter()
                .map(|tx| {
                    AvroTransaction::new(
                        tx,
                        Some(block.height.into()),
                        Some(block.header.get_timestamp_utc().timestamp()),
                        Some(block.id.as_ref().to_vec()),
                        Some(block.version.to_string()),
                        Some(block.producer.as_ref().to_vec()),
                    )
                })
                .collect()
        })
    }

    pub fn calculate_receipts_batches(
        &self,
        blocks_data: &[(Block, Vec<Transaction>)],
    ) -> Vec<(BlockHeight, BlockHeight)> {
        self.calculate_batches(blocks_data, |block, transactions| {
            let mut receipts_batch = Vec::new();
            let receipts: Vec<_> = transactions
                .iter()
                .flat_map(|tx| tx.receipts.iter().map(|r| (r, tx.clone())))
                .collect();
            for (receipt, tx) in receipts {
                let avro_receipt = AvroReceipt::from((block, &tx, receipt));
                receipts_batch.push(avro_receipt);
            }
            receipts_batch
        })
    }
}

#[cfg(test)]
mod tests {
    use fuel_streams_domains::mocks::{
        MockBlock,
        MockInput,
        MockOutput,
        MockReceipt,
        MockTransaction,
    };
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn test_single_large_block() -> anyhow::Result<()> {
        let calculator = BatchCalculator::new(1024);
        let block = MockBlock::random();
        let txs = vec![
            MockTransaction::script(
                MockInput::all(),
                MockOutput::all(),
                MockReceipt::all()
            );
            1000
        ];
        let blocks_and_txs = vec![(block.clone(), txs.clone())];
        let batches = calculator.calculate_blocks_batches(&blocks_and_txs);
        assert_eq!(
            batches.len(),
            1,
            "Should create one batch despite exceeding max_file_size"
        );
        assert_eq!(
            batches[0],
            (block.height, block.height),
            "Batch should contain only this block's height"
        );

        // Verify size exceeds max_file_size
        let items = vec![AvroBlock::new(
            &block,
            txs.clone()
                .iter()
                .map(|tx| AvroTransaction::from((&block, tx)))
                .collect(),
        )];
        let mut avro_writer = AvroParser::default()
            .writer_with_schema::<AvroBlock>()
            .expect("Failed to create Avro writer");
        avro_writer.append(&items[0]).unwrap();
        let size = avro_writer.into_inner().unwrap().len();
        assert!(size > 1024, "Serialized size should exceed max_file_size");

        Ok(())
    }
}
