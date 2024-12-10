use std::sync::Arc;

use fuel_core_like::FuelCoreLike;
use fuel_core_types::blockchain::SealedBlock;
use fuel_streams_core::prelude::*;
use serde::{Deserialize, Serialize};

pub mod block_payload;
pub mod cli;
pub mod fuel_core_like;
pub mod shutdown;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishOpts {
    pub chain_id: FuelCoreChainId,
    pub base_asset_id: FuelCoreAssetId,
    pub block_producer: Address,
    pub block_height: BlockHeight,
    pub consensus: Consensus,
}

impl PublishOpts {
    pub fn new(
        fuel_core: &Arc<dyn FuelCoreLike>,
        sealed_block: &SealedBlock,
    ) -> Self {
        let block = sealed_block.entity.clone();
        let consensus = sealed_block.consensus.clone();
        let height = *block.header().consensus().height;
        let producer =
            consensus.block_producer(&block.id()).unwrap_or_default();
        Self {
            chain_id: *fuel_core.chain_id(),
            base_asset_id: *fuel_core.base_asset_id(),
            block_producer: producer.into(),
            block_height: height.into(),
            consensus: consensus.into(),
        }
    }
}
