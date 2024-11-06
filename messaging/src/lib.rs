use bytes::Bytes;
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

/// Prost Message Wrapper allowing serialization/deserialization
pub(crate) struct ProstMessageSerdelizer<T: prost::Message>(pub(crate) T);

impl<T> ProstMessageSerdelizer<T>
where
    T: prost::Message + std::default::Default,
{
    /// Method to serialize
    pub(crate) fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.0.encode(&mut buf).map_err(|e| anyhow::anyhow!("prost encoding error {:?}", e))?;
        Ok(buf)
    }

    /// Method to deserialize
    #[allow(dead_code)]
    pub(crate) fn deserialize(buf: Vec<u8>) -> anyhow::Result<T> {
        T::decode(Bytes::from(buf)).map_err(|e| anyhow::anyhow!("prost decoding error {:?}", e))
    }
}
