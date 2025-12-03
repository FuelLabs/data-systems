use std::collections::HashMap;

use apache_avro::{
    Error,
    schema::{
        Name,
        Namespace,
        Schema,
        derive::AvroSchemaComponent,
    },
    types::Value,
};
use fuel_streams_types::{
    Address,
    AssetId,
    BlobId,
    BlockId,
    Bytes32,
    Bytes64,
    ContractId,
    HexData,
    MessageId,
    Nonce,
    Salt,
    Signature,
    TxId,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AvroBytes(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl AvroBytes {
    pub fn random(len: usize) -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        let mut bytes = vec![0u8; len];
        rng.fill(bytes.as_mut_slice());
        Self(bytes)
    }
}

impl TryFrom<AvroBytes> for Value {
    type Error = Error;
    fn try_from(value: AvroBytes) -> Result<Self, Self::Error> {
        Ok(Value::Bytes(value.0))
    }
}

impl TryFrom<Value> for AvroBytes {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bytes(bytes) => Ok(AvroBytes(bytes)),
            _ => Err(Error::Validation),
        }
    }
}

impl AvroSchemaComponent for AvroBytes {
    fn get_schema_in_ctxt(
        _named_schemas: &mut HashMap<Name, Schema>,
        _enclosing_namespace: &Namespace,
    ) -> Schema {
        Schema::Bytes
    }
}

impl From<Vec<u8>> for AvroBytes {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<Bytes32> for AvroBytes {
    fn from(value: Bytes32) -> Self {
        AvroBytes(value.as_ref().to_vec())
    }
}

impl From<Bytes64> for AvroBytes {
    fn from(value: Bytes64) -> Self {
        AvroBytes(value.as_ref().to_vec())
    }
}

impl From<HexData> for AvroBytes {
    fn from(value: HexData) -> Self {
        AvroBytes(value.0.0.clone())
    }
}

impl From<Address> for AvroBytes {
    fn from(value: Address) -> Self {
        AvroBytes(value.as_ref().to_vec())
    }
}

impl From<BlockId> for AvroBytes {
    fn from(value: BlockId) -> Self {
        AvroBytes(value.as_ref().to_vec())
    }
}

impl From<Signature> for AvroBytes {
    fn from(value: Signature) -> Self {
        AvroBytes(value.as_ref().to_vec())
    }
}

impl From<AssetId> for AvroBytes {
    fn from(value: AssetId) -> Self {
        AvroBytes(value.as_ref().to_vec())
    }
}

impl From<BlobId> for AvroBytes {
    fn from(value: BlobId) -> Self {
        AvroBytes(value.as_ref().to_vec())
    }
}

impl From<MessageId> for AvroBytes {
    fn from(value: MessageId) -> Self {
        AvroBytes(value.as_ref().to_vec())
    }
}

impl From<Nonce> for AvroBytes {
    fn from(value: Nonce) -> Self {
        AvroBytes(value.as_ref().to_vec())
    }
}

impl From<Salt> for AvroBytes {
    fn from(value: Salt) -> Self {
        AvroBytes(value.as_ref().to_vec())
    }
}

impl From<TxId> for AvroBytes {
    fn from(value: TxId) -> Self {
        AvroBytes(value.as_ref().to_vec())
    }
}

impl From<ContractId> for AvroBytes {
    fn from(value: ContractId) -> Self {
        AvroBytes(value.as_ref().to_vec())
    }
}
