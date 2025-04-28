use std::collections::{BTreeMap, HashMap};

use apache_avro::schema::{
    derive::AvroSchemaComponent,
    FixedSchema,
    Name,
    Namespace,
    Schema,
};
use fuel_core_types::fuel_types;

use super::{LongBytes, UtxoId};
use crate::{
    fuel_core::*,
    generate_bool_type_wrapper,
    generate_byte_type_wrapper,
    impl_avro_schema_for_bool,
    impl_avro_schema_for_fixed_bytes,
    impl_avro_schema_for_variable_bytes,
    impl_bytes32_to_type,
    impl_from_type_to_bytes32,
    impl_utoipa_for_byte_type_detailed,
};

generate_byte_type_wrapper!(Address, fuel_types::Address, 32);
generate_byte_type_wrapper!(Bytes32, fuel_types::Bytes32, 32);
generate_byte_type_wrapper!(Bytes64, fuel_types::Bytes64, 64);
generate_byte_type_wrapper!(ContractId, fuel_types::ContractId, 32);
generate_byte_type_wrapper!(AssetId, fuel_types::AssetId, 32);
generate_byte_type_wrapper!(BlobId, fuel_types::BlobId, 32);
generate_byte_type_wrapper!(Nonce, fuel_types::Nonce, 32);
generate_byte_type_wrapper!(Salt, fuel_types::Salt, 32);
generate_byte_type_wrapper!(MessageId, fuel_types::MessageId, 32);
generate_byte_type_wrapper!(BlockId, fuel_types::Bytes32, 32);
generate_byte_type_wrapper!(Signature, fuel_types::Bytes64, 64);
generate_byte_type_wrapper!(TxId, fuel_types::TxId, 32);
generate_byte_type_wrapper!(HexData, LongBytes);

impl_utoipa_for_byte_type_detailed!(
    Address,
    32,
    "A 32-byte Fuel address with 0x prefix"
);

impl_utoipa_for_byte_type_detailed!(
    BlobId,
    32,
    "A 32-byte Fuel blob id with 0x prefix"
);

impl_utoipa_for_byte_type_detailed!(Salt, 32, "A 32-byte salt with 0x prefix");

impl_utoipa_for_byte_type_detailed!(
    AssetId,
    32,
    "A 32-byte asset identifier with 0x prefix"
);
impl_utoipa_for_byte_type_detailed!(
    Bytes32,
    32,
    "A 32-byte value with 0x prefix"
);
impl_utoipa_for_byte_type_detailed!(
    ContractId,
    32,
    "A 32-byte contract identifier with 0x prefix"
);
impl_utoipa_for_byte_type_detailed!(
    TxId,
    32,
    "A 32-byte transaction identifier with 0x prefix"
);

impl_utoipa_for_byte_type_detailed!(
    BlockId,
    32,
    "A 32-byte block identifier with 0x prefix"
);

impl_utoipa_for_byte_type_detailed!(
    Signature,
    64,
    "A 64-byte signature with 0x prefix"
);

impl_utoipa_for_byte_type_detailed!(
    HexData,
    "Variable-length hexadecimal data with 0x prefix"
);

impl_utoipa_for_byte_type_detailed!(
    Nonce,
    32,
    "A 32-byte Fuel nonce with 0x prefix"
);

impl From<&UtxoId> for HexData {
    fn from(value: &UtxoId) -> Self {
        value.to_owned().into()
    }
}

impl From<UtxoId> for HexData {
    fn from(value: UtxoId) -> Self {
        let mut bytes = Vec::with_capacity(34);
        bytes.extend_from_slice(value.tx_id.0.as_ref());
        bytes.extend_from_slice(&value.output_index.to_be_bytes());
        HexData(bytes.into())
    }
}

impl From<Address> for HexData {
    fn from(value: Address) -> Self {
        HexData(value.into_inner().to_vec().into())
    }
}

impl From<BlockId> for HexData {
    fn from(value: BlockId) -> Self {
        HexData(value.into_inner().to_vec().into())
    }
}

impl From<Bytes32> for HexData {
    fn from(value: Bytes32) -> Self {
        HexData(value.into_inner().to_vec().into())
    }
}

impl From<Bytes64> for HexData {
    fn from(value: Bytes64) -> Self {
        HexData(value.into_inner().to_vec().into())
    }
}

impl_bytes32_to_type!(MessageId);
impl_bytes32_to_type!(ContractId);
impl_bytes32_to_type!(AssetId);
impl_bytes32_to_type!(Address);
impl_bytes32_to_type!(BlobId);
impl_bytes32_to_type!(Nonce);
impl_bytes32_to_type!(Salt);
impl_bytes32_to_type!(BlockId);
impl_bytes32_to_type!(TxId);

impl_from_type_to_bytes32!(fuel_types::ContractId);
impl_from_type_to_bytes32!(fuel_types::AssetId);
impl_from_type_to_bytes32!(fuel_types::Address);

impl From<FuelCoreBlockId> for BlockId {
    fn from(value: FuelCoreBlockId) -> Self {
        Self(FuelCoreBytes32::from(value))
    }
}

generate_bool_type_wrapper!(BoolValue);
impl_avro_schema_for_bool!(BoolValue);

impl_avro_schema_for_fixed_bytes!(Address, 32);
impl_avro_schema_for_fixed_bytes!(Bytes32, 32);
impl_avro_schema_for_fixed_bytes!(Bytes64, 64);
impl_avro_schema_for_fixed_bytes!(ContractId, 32);
impl_avro_schema_for_fixed_bytes!(AssetId, 32);
impl_avro_schema_for_fixed_bytes!(BlobId, 32);
impl_avro_schema_for_fixed_bytes!(Nonce, 32);
impl_avro_schema_for_fixed_bytes!(Salt, 32);
impl_avro_schema_for_fixed_bytes!(MessageId, 32);
impl_avro_schema_for_fixed_bytes!(BlockId, 32);
impl_avro_schema_for_fixed_bytes!(Signature, 64);
impl_avro_schema_for_fixed_bytes!(TxId, 32);
impl_avro_schema_for_variable_bytes!(HexData);
