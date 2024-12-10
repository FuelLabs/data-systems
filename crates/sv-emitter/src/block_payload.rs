use fuel_core_types::{blockchain::SealedBlock, fuel_tx::Transaction};
use fuel_streams_core::prelude::*;
use serde::{Deserialize, Serialize};

use crate::PublishOpts;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPayload {
    pub block: FuelCoreBlock,
    pub transactions: Vec<Transaction>,
    pub opts: PublishOpts,
}

impl BlockPayload {
    pub fn new(sealed_block: &SealedBlock, opts: &PublishOpts) -> Self {
        let block = sealed_block.entity.clone();
        let transactions = block.transactions_vec().clone();
        Self {
            block,
            transactions,
            opts: opts.to_owned(),
        }
    }

    pub fn encode(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn decode(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn tx_ids(&self) -> Vec<Bytes32> {
        self.transactions
            .iter()
            .map(|tx| tx.id(&self.opts.chain_id).into())
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use fuel_core_types::{
        blockchain::{
            block::Block as FuelCoreBlock,
            consensus::Consensus as FuelCoreConsensus,
        },
        fuel_tx::Transaction,
    };

    use super::*;

    #[test]
    fn test_block_payload_json_encoding() {
        // Create test data
        let mut block = FuelCoreBlock::default();
        let tx = Transaction::default_test_tx();
        *block.transactions_mut() = vec![tx.clone()];

        let sealed_block = SealedBlock {
            entity: block,
            consensus: FuelCoreConsensus::default(),
        };

        let opts = PublishOpts {
            chain_id: Default::default(),
            base_asset_id: Default::default(),
            block_producer: Default::default(),
            block_height: 1u32.into(),
            consensus: Default::default(),
        };

        // Create BlockPayload
        let original_payload = BlockPayload::new(&sealed_block, &opts);

        // Encode
        let encoded = original_payload.encode().expect("Failed to encode");

        // Decode
        let decoded = BlockPayload::decode(&encoded).expect("Failed to decode");

        // Verify the decoded data matches the original
        assert_eq!(decoded.block, original_payload.block);
        assert_eq!(decoded.transactions, original_payload.transactions);
        assert_eq!(decoded.opts.chain_id, original_payload.opts.chain_id);
        assert_eq!(
            decoded.opts.base_asset_id,
            original_payload.opts.base_asset_id
        );
        assert_eq!(
            decoded.opts.block_producer,
            original_payload.opts.block_producer
        );
        assert_eq!(
            decoded.opts.block_height,
            original_payload.opts.block_height
        );
        assert_eq!(decoded.opts.consensus, original_payload.opts.consensus);
    }
}
