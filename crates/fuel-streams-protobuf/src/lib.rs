#![doc = include_str!("../README.md")]

#[allow(clippy::disallowed_methods)]
pub mod fueltypes;

pub use fuel_core_types::blockchain::block::{Block, BlockV1};

impl From<Block> for fueltypes::Block {
    fn from(value: Block) -> Self {
        let height = *value.header().consensus().height;
        fueltypes::Block {
            header: Some(fueltypes::BlockHeader {
                da_height: height,
                consensus_parameters_version: value
                    .header()
                    .application()
                    .consensus_parameters_version,
                state_transition_bytecode_version: value
                    .header()
                    .application()
                    .state_transition_bytecode_version,
            }),
            transactions: vec![],
        }
    }
}
