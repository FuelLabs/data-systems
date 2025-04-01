use std::str::FromStr;

use fuel_data_parser::DataEncoder;
use fuel_streams_types::primitives::*;
use serde::{Deserialize, Serialize};

use super::{blob_id_from_bytecode, PredicateDbItem};
use crate::infra::{record::ToPacket, DbError};

#[derive(
    Debug,
    Clone,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct Predicate {
    pub blob_id: Option<HexData>,
    pub predicate_address: Address,
    pub predicate_bytecode: HexData,
    pub tx_id: TxId,
    pub tx_index: u32,
    pub input_index: u32,
    pub asset_id: AssetId,
}

impl DataEncoder for Predicate {}
impl ToPacket for Predicate {}

impl Predicate {
    pub fn new(
        tx_id: &TxId,
        tx_index: u32,
        input_index: u32,
        blob_id: Option<HexData>,
        predicate_address: &Address,
        predicate_bytecode: &HexData,
        asset_id: &AssetId,
    ) -> Self {
        Self {
            blob_id,
            tx_id: tx_id.clone(),
            tx_index,
            input_index,
            predicate_address: predicate_address.clone(),
            predicate_bytecode: predicate_bytecode.clone(),
            asset_id: asset_id.to_owned(),
        }
    }
}

impl TryFrom<&PredicateDbItem> for Predicate {
    type Error = DbError;

    fn try_from(item: &PredicateDbItem) -> Result<Self, Self::Error> {
        let tx_id = TxId::from_str(&item.tx_id)
            .map_err(|e| DbError::Other(e.to_string()))?;
        let blob_id = item
            .blob_id
            .as_ref()
            .and_then(|b| HexData::from_str(b).ok());
        let predicate_address = Address::from_str(&item.predicate_address)
            .map_err(|e| DbError::Other(e.to_string()))?;
        let predicate_bytecode = HexData::from_str(&item.bytecode)
            .map_err(|e| DbError::Other(e.to_string()))?;
        let asset_id = AssetId::from_str(&item.asset_id)
            .map_err(|e| DbError::Other(e.to_string()))?;
        let predicate = Predicate::new(
            &tx_id,
            item.tx_index as u32,
            item.input_index as u32,
            blob_id,
            &predicate_address,
            &predicate_bytecode,
            &asset_id,
        );
        Ok(predicate)
    }
}

#[cfg(any(test, feature = "test-helpers"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockPredicate;
#[cfg(any(test, feature = "test-helpers"))]
impl MockPredicate {
    const VALID_BYTECODE: &str = "1a403000504100301a445000ba49000032400481504100205d490000504100083240048220451300524510044a440000cf534ed3e0f17779f12866863001e53beb68e87621308fbe7f575652dba0dc000000000000000108f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad0700000000000000010000000000002710666c984d4c0aa70abb14a6d6e7693fc5bda8275d6c6716c8bcae33b3c21a1dfb6fd333a74ac52ca7d50d7e768996acd0fb339fcc8a109796b2c55d2f341631d3a0265fb5c32f6e8db3197af3c7eb05c48ae373605b8165b6f4a51c5b0ba4812edfda4cd39004d68b93c8be7a82f67c18661345e0b8e03a479a9eb4118277c2f190d67a87f1def93ab95e5d940d1534e2d9fed411ba78c9add53930d5b567d3b2cccccccccccc00020000000000000000000000000000000000000000000000000000000000000000000000000000158c0000000000000cf4";

    pub fn with_blob_id() -> Predicate {
        let bytes = hex::decode(Self::VALID_BYTECODE).unwrap();
        let blob_id = blob_id_from_bytecode(bytes.into());
        Predicate::new(
            &TxId::random(),
            0,
            0,
            blob_id,
            &Address::random(),
            &HexData::from_str(Self::VALID_BYTECODE).unwrap(),
            &AssetId::random(),
        )
    }

    pub fn without_blob_id() -> Predicate {
        Predicate::new(
            &TxId::random(),
            0,
            0,
            None,
            &Address::random(),
            &HexData::zeroed(),
            &AssetId::random(),
        )
    }
}
