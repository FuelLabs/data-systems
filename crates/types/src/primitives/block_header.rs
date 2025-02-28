use serde::{Deserialize, Serialize};
use wrapped_int::WrappedU32;

use crate::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockTime(pub FuelCoreTai64);
impl BlockTime {
    pub fn into_inner(self) -> FuelCoreTai64 {
        self.0
    }
    pub fn from_unix(secs: i64) -> Self {
        Self(FuelCoreTai64::from_unix(secs))
    }
}

impl From<FuelCoreTai64> for BlockTime {
    fn from(value: FuelCoreTai64) -> Self {
        Self(value)
    }
}

impl Default for BlockTime {
    fn default() -> Self {
        Self(FuelCoreTai64::from_unix(0))
    }
}

// Header type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    pub application_hash: Bytes32,
    pub consensus_parameters_version: WrappedU32,
    pub da_height: DaBlockHeight,
    pub event_inbox_root: Bytes32,
    pub id: BlockId,
    pub height: BlockHeight,
    pub message_outbox_root: Bytes32,
    pub message_receipt_count: WrappedU32,
    pub prev_root: Bytes32,
    pub state_transition_bytecode_version: WrappedU32,
    pub time: BlockTime,
    pub transactions_count: u16,
    pub transactions_root: Bytes32,
    pub version: BlockHeaderVersion,
}

impl From<&FuelCoreBlockHeader> for BlockHeader {
    fn from(header: &FuelCoreBlockHeader) -> Self {
        let version = match header {
            FuelCoreBlockHeader::V1(_) => BlockHeaderVersion::V1,
        };

        Self {
            application_hash: (*header.application_hash()).into(),
            consensus_parameters_version: header
                .consensus_parameters_version
                .into(),
            da_height: header.da_height.into(),
            event_inbox_root: header.event_inbox_root.into(),
            id: header.id().into(),
            height: (*header.height()).into(),
            message_outbox_root: header.message_outbox_root.into(),
            message_receipt_count: header.message_receipt_count.into(),
            prev_root: (*header.prev_root()).into(),
            state_transition_bytecode_version: header
                .state_transition_bytecode_version
                .into(),
            time: header.time().into(),
            transactions_count: header.transactions_count,
            transactions_root: header.transactions_root.into(),
            version,
        }
    }
}

// BlockHeaderVersion enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockHeaderVersion {
    #[default]
    V1,
}
