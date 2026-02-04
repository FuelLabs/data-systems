use std::{
    fs::{
        self,
        File,
    },
    io::{
        BufReader,
        BufWriter,
        Write,
    },
    path::{
        Path,
        PathBuf,
    },
};

use fuel_streams_types::BlockHeight;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    DuneError,
    DuneResult,
    helpers::AvroParser,
    schemas::{
        AvroBlock,
        AvroReceipt,
        AvroTransaction,
        ReceiptMetadata,
    },
};
use fuel_streams_domains::{
    blocks::Block,
    transactions::Transaction,
};

/// The result of finalizing a batch, containing Avro-encoded data ready for upload
pub struct FinalizedBatch {
    pub first_height: BlockHeight,
    pub last_height: BlockHeight,
    pub blocks_data: Vec<u8>,
    pub transactions_data: Vec<u8>,
    pub receipts_data: Vec<u8>,
}

/// Configuration for the buffer type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    /// Store blocks in memory (faster but uses more RAM)
    Memory,
    /// Store blocks on disk (slower but uses less RAM)
    Disk,
}

impl std::fmt::Display for BufferType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferType::Memory => write!(f, "Memory"),
            BufferType::Disk => write!(f, "Disk"),
        }
    }
}

impl std::str::FromStr for BufferType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "memory" | "mem" => Ok(BufferType::Memory),
            "disk" | "file" => Ok(BufferType::Disk),
            _ => Err(anyhow::anyhow!("Unknown buffer type: {}", s)),
        }
    }
}

/// Trait for block buffering implementations.
/// This allows swapping between memory and disk-based buffering.
pub trait BlockBuffer: Send {
    /// Returns the number of blocks in the buffer
    fn len(&self) -> usize;

    /// Returns true if the buffer is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the first block height in the buffer
    fn first_height(&self) -> Option<BlockHeight>;

    /// Returns the last block height in the buffer
    fn last_height(&self) -> Option<BlockHeight>;

    /// Appends a block and its transactions to the buffer
    fn append(&mut self, block: &Block, transactions: &[Transaction]) -> DuneResult<()>;

    /// Finalizes the buffer, converting all data to Avro format for upload.
    /// This consumes the buffer.
    fn finalize(self: Box<Self>) -> DuneResult<FinalizedBatch>;

    /// Resets the buffer for reuse, clearing all data
    fn reset(&mut self) -> DuneResult<()>;
}

/// Creates a new block buffer of the specified type
pub fn create_buffer(buffer_type: BufferType) -> DuneResult<Box<dyn BlockBuffer>> {
    match buffer_type {
        BufferType::Memory => Ok(Box::new(MemoryBuffer::new())),
        BufferType::Disk => Ok(Box::new(DiskBuffer::new()?)),
    }
}

// ============================================================================
// Memory-based buffer implementation
// ============================================================================

/// Intermediate representation of a block for buffering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferedBlock {
    pub block: AvroBlock,
    pub transactions: Vec<AvroTransaction>,
    pub receipts: Vec<AvroReceipt>,
}

impl BufferedBlock {
    /// Creates a new buffered block from domain types
    pub fn from_domain(block: &Block, transactions: &[Transaction]) -> Self {
        let avro_block = AvroBlock::new(block);

        let avro_transactions: Vec<_> = transactions
            .iter()
            .map(|tx| {
                AvroTransaction::new(
                    tx,
                    Some(block.height.into()),
                    Some(block.header.get_timestamp_utc().timestamp()),
                    Some(block.id.as_ref().to_vec().into()),
                    Some(block.version.to_string()),
                    Some(block.producer.as_ref().to_vec().into()),
                )
            })
            .collect();

        let avro_receipts: Vec<_> = transactions
            .iter()
            .flat_map(|tx| {
                let receipt_metadata = ReceiptMetadata {
                    block_time: Some(block.header.get_timestamp_utc().timestamp()),
                    block_height: Some(block.height.0 as i64),
                    block_version: Some(block.version.to_string()),
                    block_producer: Some(block.producer.clone().into()),
                    transaction_id: Some(tx.id.clone().into()),
                };
                tx.receipts
                    .iter()
                    .map(move |receipt| AvroReceipt::new(receipt, &receipt_metadata))
            })
            .collect();

        Self {
            block: avro_block,
            transactions: avro_transactions,
            receipts: avro_receipts,
        }
    }
}

/// Memory-based block buffer that stores all blocks in a Vec.
/// Faster than disk-based buffering but uses more memory.
pub struct MemoryBuffer {
    blocks: Vec<BufferedBlock>,
    first_height: Option<BlockHeight>,
    last_height: Option<BlockHeight>,
}

impl MemoryBuffer {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            first_height: None,
            last_height: None,
        }
    }

    /// Converts buffered blocks to Avro format
    fn to_avro(&self) -> DuneResult<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        let parser = AvroParser::default();

        let mut blocks_writer =
            parser.writer_with_schema::<AvroBlock>().map_err(|e| {
                DuneError::Other(anyhow::anyhow!("Failed to create blocks writer: {}", e))
            })?;
        let mut transactions_writer = parser
            .writer_with_schema::<AvroTransaction>()
            .map_err(|e| {
                DuneError::Other(anyhow::anyhow!(
                    "Failed to create transactions writer: {}",
                    e
                ))
            })?;
        let mut receipts_writer =
            parser.writer_with_schema::<AvroReceipt>().map_err(|e| {
                DuneError::Other(anyhow::anyhow!(
                    "Failed to create receipts writer: {}",
                    e
                ))
            })?;

        for buffered in &self.blocks {
            blocks_writer.append(&buffered.block)?;
            for tx in &buffered.transactions {
                transactions_writer.append(tx)?;
            }
            for receipt in &buffered.receipts {
                receipts_writer.append(receipt)?;
            }
        }

        let blocks_data = blocks_writer.into_inner()?;
        let transactions_data = transactions_writer.into_inner()?;
        let receipts_data = receipts_writer.into_inner()?;

        Ok((blocks_data, transactions_data, receipts_data))
    }
}

impl Default for MemoryBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockBuffer for MemoryBuffer {
    fn len(&self) -> usize {
        self.blocks.len()
    }

    fn first_height(&self) -> Option<BlockHeight> {
        self.first_height
    }

    fn last_height(&self) -> Option<BlockHeight> {
        self.last_height
    }

    fn append(&mut self, block: &Block, transactions: &[Transaction]) -> DuneResult<()> {
        let height = block.height;

        if self.first_height.is_none() {
            self.first_height = Some(height);
        }
        self.last_height = Some(height);

        let buffered = BufferedBlock::from_domain(block, transactions);
        self.blocks.push(buffered);

        Ok(())
    }

    fn finalize(self: Box<Self>) -> DuneResult<FinalizedBatch> {
        let first_height = self.first_height.ok_or_else(|| {
            DuneError::Other(anyhow::anyhow!("Cannot finalize empty buffer"))
        })?;
        let last_height = self.last_height.ok_or_else(|| {
            DuneError::Other(anyhow::anyhow!("Cannot finalize empty buffer"))
        })?;

        let (blocks_data, transactions_data, receipts_data) = self.to_avro()?;

        Ok(FinalizedBatch {
            first_height,
            last_height,
            blocks_data,
            transactions_data,
            receipts_data,
        })
    }

    fn reset(&mut self) -> DuneResult<()> {
        self.blocks.clear();
        self.first_height = None;
        self.last_height = None;
        Ok(())
    }
}

// ============================================================================
// Disk-based buffer implementation
// ============================================================================

/// Disk-based block buffer that writes blocks to temporary files.
/// Uses less memory than the memory-based buffer but is slower.
/// Blocks are stored in JSON Lines format and converted to Avro on finalize.
pub struct DiskBuffer {
    temp_dir: PathBuf,
    data_file: PathBuf,
    writer: Option<BufWriter<File>>,
    first_height: Option<BlockHeight>,
    last_height: Option<BlockHeight>,
    block_count: usize,
}

impl DiskBuffer {
    pub fn new() -> DuneResult<Self> {
        let temp_dir = std::env::temp_dir().join(format!(
            "dune-buffer-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        Self::with_dir(&temp_dir)
    }

    pub fn with_dir(dir: impl AsRef<Path>) -> DuneResult<Self> {
        let temp_dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&temp_dir)?;

        let data_file = temp_dir.join("blocks.jsonl");
        let writer = BufWriter::new(File::create(&data_file)?);

        Ok(Self {
            temp_dir,
            data_file,
            writer: Some(writer),
            first_height: None,
            last_height: None,
            block_count: 0,
        })
    }

    fn read_and_convert_to_avro(&self) -> DuneResult<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        let file = File::open(&self.data_file)?;
        let reader = BufReader::new(file);

        let parser = AvroParser::default();
        let mut blocks_writer =
            parser.writer_with_schema::<AvroBlock>().map_err(|e| {
                DuneError::Other(anyhow::anyhow!("Failed to create blocks writer: {}", e))
            })?;
        let mut transactions_writer = parser
            .writer_with_schema::<AvroTransaction>()
            .map_err(|e| {
                DuneError::Other(anyhow::anyhow!(
                    "Failed to create transactions writer: {}",
                    e
                ))
            })?;
        let mut receipts_writer =
            parser.writer_with_schema::<AvroReceipt>().map_err(|e| {
                DuneError::Other(anyhow::anyhow!(
                    "Failed to create receipts writer: {}",
                    e
                ))
            })?;

        for line in std::io::BufRead::lines(reader) {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            let buffered: BufferedBlock = serde_json::from_str(&line).map_err(|e| {
                DuneError::Other(anyhow::anyhow!("Failed to deserialize block: {}", e))
            })?;

            blocks_writer.append(&buffered.block)?;
            for tx in buffered.transactions {
                transactions_writer.append(&tx)?;
            }
            for receipt in buffered.receipts {
                receipts_writer.append(&receipt)?;
            }
        }

        let blocks_data = blocks_writer.into_inner()?;
        let transactions_data = transactions_writer.into_inner()?;
        let receipts_data = receipts_writer.into_inner()?;

        Ok((blocks_data, transactions_data, receipts_data))
    }
}

impl BlockBuffer for DiskBuffer {
    fn len(&self) -> usize {
        self.block_count
    }

    fn first_height(&self) -> Option<BlockHeight> {
        self.first_height
    }

    fn last_height(&self) -> Option<BlockHeight> {
        self.last_height
    }

    fn append(&mut self, block: &Block, transactions: &[Transaction]) -> DuneResult<()> {
        let height = block.height;

        if self.first_height.is_none() {
            self.first_height = Some(height);
        }
        self.last_height = Some(height);

        let buffered = BufferedBlock::from_domain(block, transactions);

        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| DuneError::Other(anyhow::anyhow!("Writer not available")))?;

        serde_json::to_writer(&mut *writer, &buffered).map_err(|e| {
            DuneError::Other(anyhow::anyhow!("Failed to serialize block: {}", e))
        })?;
        writer.write_all(b"\n")?;

        self.block_count += 1;

        Ok(())
    }

    fn finalize(mut self: Box<Self>) -> DuneResult<FinalizedBatch> {
        let first_height = self.first_height.ok_or_else(|| {
            DuneError::Other(anyhow::anyhow!("Cannot finalize empty buffer"))
        })?;
        let last_height = self.last_height.ok_or_else(|| {
            DuneError::Other(anyhow::anyhow!("Cannot finalize empty buffer"))
        })?;

        // Flush and close the writer
        if let Some(mut writer) = self.writer.take() {
            writer.flush()?;
        }

        let (blocks_data, transactions_data, receipts_data) =
            self.read_and_convert_to_avro()?;

        Ok(FinalizedBatch {
            first_height,
            last_height,
            blocks_data,
            transactions_data,
            receipts_data,
        })
    }

    fn reset(&mut self) -> DuneResult<()> {
        // Close current writer
        let _ = self.writer.take();

        // Remove old file
        let _ = fs::remove_file(&self.data_file);

        // Create new writer
        self.writer = Some(BufWriter::new(File::create(&self.data_file)?));

        self.first_height = None;
        self.last_height = None;
        self.block_count = 0;

        Ok(())
    }
}

impl Drop for DiskBuffer {
    fn drop(&mut self) {
        let _ = self.writer.take();
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fuel_streams_domains::mocks::{
        MockBlock,
        MockReceipt,
        MockTransaction,
    };
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    fn test_buffer_basic_operations(mut buffer: Box<dyn BlockBuffer>) -> DuneResult<()> {
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        // Add some blocks
        for i in 1..=10 {
            let mut block = MockBlock::random();
            block.height = BlockHeight::from(i);
            let txs = vec![MockTransaction::script(vec![], vec![], vec![])];
            buffer.append(&block, &txs)?;
        }

        assert_eq!(buffer.len(), 10);
        assert_eq!(buffer.first_height(), Some(BlockHeight::from(1)));
        assert_eq!(buffer.last_height(), Some(BlockHeight::from(10)));

        // Finalize and verify
        let finalized = buffer.finalize()?;
        assert_eq!(*finalized.first_height, 1);
        assert_eq!(*finalized.last_height, 10);
        assert!(!finalized.blocks_data.is_empty());
        assert!(!finalized.transactions_data.is_empty());

        Ok(())
    }

    fn test_buffer_reset(mut buffer: Box<dyn BlockBuffer>) -> DuneResult<()> {
        // Add blocks
        for i in 1..=5 {
            let mut block = MockBlock::random();
            block.height = BlockHeight::from(i);
            let txs = vec![MockTransaction::script(vec![], vec![], vec![])];
            buffer.append(&block, &txs)?;
        }

        assert_eq!(buffer.len(), 5);

        // Reset
        buffer.reset()?;

        assert!(buffer.is_empty());
        assert_eq!(buffer.first_height(), None);
        assert_eq!(buffer.last_height(), None);

        // Can add more blocks after reset
        let mut block = MockBlock::random();
        block.height = BlockHeight::from(100);
        let txs = vec![MockTransaction::script(vec![], vec![], vec![])];
        buffer.append(&block, &txs)?;

        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.first_height(), Some(BlockHeight::from(100)));

        Ok(())
    }

    fn test_buffer_with_receipts(mut buffer: Box<dyn BlockBuffer>) -> DuneResult<()> {
        for i in 1..=3 {
            let mut block = MockBlock::random();
            block.height = BlockHeight::from(i);
            let txs = vec![MockTransaction::script(vec![], vec![], MockReceipt::all())];
            buffer.append(&block, &txs)?;
        }

        let finalized = buffer.finalize()?;
        assert!(!finalized.receipts_data.is_empty());

        Ok(())
    }

    // Memory buffer tests
    #[test]
    fn test_memory_buffer_basic() -> DuneResult<()> {
        test_buffer_basic_operations(Box::new(MemoryBuffer::new()))
    }

    #[test]
    fn test_memory_buffer_reset() -> DuneResult<()> {
        test_buffer_reset(Box::new(MemoryBuffer::new()))
    }

    #[test]
    fn test_memory_buffer_with_receipts() -> DuneResult<()> {
        test_buffer_with_receipts(Box::new(MemoryBuffer::new()))
    }

    // Disk buffer tests
    #[test]
    fn test_disk_buffer_basic() -> DuneResult<()> {
        let dir = tempdir().unwrap();
        test_buffer_basic_operations(Box::new(DiskBuffer::with_dir(dir.path())?))
    }

    #[test]
    fn test_disk_buffer_reset() -> DuneResult<()> {
        let dir = tempdir().unwrap();
        test_buffer_reset(Box::new(DiskBuffer::with_dir(dir.path())?))
    }

    #[test]
    fn test_disk_buffer_with_receipts() -> DuneResult<()> {
        let dir = tempdir().unwrap();
        test_buffer_with_receipts(Box::new(DiskBuffer::with_dir(dir.path())?))
    }

    // Factory function tests
    #[test]
    fn test_create_memory_buffer() -> DuneResult<()> {
        let buffer = create_buffer(BufferType::Memory)?;
        assert!(buffer.is_empty());
        Ok(())
    }

    #[test]
    fn test_create_disk_buffer() -> DuneResult<()> {
        let buffer = create_buffer(BufferType::Disk)?;
        assert!(buffer.is_empty());
        Ok(())
    }

    #[test]
    fn test_buffer_type_from_str() {
        use std::str::FromStr;
        assert_eq!(BufferType::from_str("memory").unwrap(), BufferType::Memory);
        assert_eq!(BufferType::from_str("Memory").unwrap(), BufferType::Memory);
        assert_eq!(BufferType::from_str("mem").unwrap(), BufferType::Memory);
        assert_eq!(BufferType::from_str("disk").unwrap(), BufferType::Disk);
        assert_eq!(BufferType::from_str("Disk").unwrap(), BufferType::Disk);
        assert_eq!(BufferType::from_str("file").unwrap(), BufferType::Disk);
        assert!(BufferType::from_str("unknown").is_err());
    }
}
