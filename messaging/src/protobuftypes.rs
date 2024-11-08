// This file is @generated by prost-build.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestBlock {
    #[prost(oneof = "test_block::Block", tags = "1")]
    pub block: ::core::option::Option<test_block::Block>,
}
/// Nested message and enum types in `TestBlock`.
pub mod test_block {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Block {
        #[prost(message, tag = "1")]
        V1(super::BlockV1),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockV1 {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<BlockHeader>,
    #[prost(message, repeated, tag = "2")]
    pub transactions: ::prost::alloc::vec::Vec<Transaction>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(oneof = "transaction::Transaction", tags = "1")]
    pub transaction: ::core::option::Option<transaction::Transaction>,
}
/// Nested message and enum types in `Transaction`.
pub mod transaction {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Transaction {
        #[prost(message, tag = "1")]
        Script(super::Script),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Script {
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<ScriptBody>,
    #[prost(message, optional, tag = "2")]
    pub policies: ::core::option::Option<Policies>,
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub inputs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(bytes = "vec", repeated, tag = "4")]
    pub outputs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(bytes = "vec", repeated, tag = "5")]
    pub witnesses: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Policies {
    #[prost(uint64, repeated, tag = "1")]
    pub values: ::prost::alloc::vec::Vec<u64>,
    #[prost(message, optional, tag = "2")]
    pub data: ::core::option::Option<::prost_types::Any>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScriptBody {
    #[prost(uint64, tag = "1")]
    pub script_gas_limit: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub receipts_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub script: ::core::option::Option<ScriptCode>,
    #[prost(bytes = "vec", tag = "4")]
    pub script_data: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScriptCode {
    #[prost(bytes = "vec", tag = "1")]
    pub bytes: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockHeader {
    #[prost(oneof = "block_header::BlockHeader", tags = "1, 2")]
    pub block_header: ::core::option::Option<block_header::BlockHeader>,
}
/// Nested message and enum types in `BlockHeader`.
pub mod block_header {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum BlockHeader {
        #[prost(message, tag = "1")]
        V1(super::BlockHeaderV1),
        #[prost(message, tag = "2")]
        V2(super::BlockHeaderV1),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockHeaderV1 {
    #[prost(message, optional, tag = "1")]
    pub application: ::core::option::Option<ApplicationHeader>,
    #[prost(message, optional, tag = "2")]
    pub consensus: ::core::option::Option<ConsensusHeader>,
    #[prost(message, optional, tag = "3")]
    pub metadata: ::core::option::Option<BlockHeaderMetadata>,
}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct BlockHeaderMetadata {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ApplicationHeader {
    #[prost(uint32, tag = "1")]
    pub da_height: u32,
    #[prost(uint32, tag = "2")]
    pub consensus_parameters_version: u32,
    #[prost(uint32, tag = "3")]
    pub state_transition_bytecode_version: u32,
    #[prost(message, optional, tag = "4")]
    pub generated: ::core::option::Option<GeneratedApplicationFields>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeneratedApplicationFields {
    #[prost(uint32, tag = "1")]
    pub transactions_count: u32,
    #[prost(uint32, tag = "2")]
    pub message_receipt_count: u32,
    #[prost(bytes = "vec", tag = "3")]
    pub transactions_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub message_outbox_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub event_inbox_root: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusHeader {
    #[prost(bytes = "vec", tag = "1")]
    pub prev_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub height: u64,
    #[prost(uint64, tag = "3")]
    pub time: u64,
    #[prost(message, optional, tag = "4")]
    pub generated: ::core::option::Option<GeneratedConsensusFields>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeneratedConsensusFields {
    #[prost(bytes = "vec", tag = "1")]
    pub application_hash: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MyDemoType {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(int32, tag = "2")]
    pub age: i32,
    #[prost(bool, tag = "3")]
    pub is_active: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(oneof = "request::ReqType", tags = "1, 2")]
    pub req_type: ::core::option::Option<request::ReqType>,
}
/// Nested message and enum types in `Request`.
pub mod request {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ReqType {
        #[prost(message, tag = "1")]
        PubKeyRequest(super::PubKeyRequest),
        #[prost(message, tag = "2")]
        PubKeyResponse(super::PubKeyResponse),
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubKeyRequest {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubKeyResponse {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
