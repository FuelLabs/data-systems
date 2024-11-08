use super::protobuftypes::ApplicationHeader;

// use borsh::{
//     BorshDeserialize,
//     BorshSchema, BorshSerialize,
// };

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub enum TestBlock {
//     /// V1 Block
//     V1(BlockV1),
// }

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub struct BlockV1 {
//     /// Generated complete header.
//     header: BlockHeader,
//     /// Executed transactions.
//     transactions: Vec<Transaction>,
// }

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub enum Transaction {
//     Script(Script),
// }

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub struct Script {
//     pub(crate) body: ScriptBody,
//     pub(crate) policies: Policies,
//     pub(crate) inputs: Vec<u8>,
//     pub(crate) outputs: Vec<u8>,
//     pub(crate) witnesses: Vec<u8>,
// }

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub struct Policies {
//     values: [u64; 10],
// }

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub struct ScriptBody {
//     pub(crate) script_gas_limit: u64,
//     pub(crate) receipts_root: Vec<u8>,
//     pub(crate) script: ScriptCode,
//     pub(crate) script_data: Vec<u8>,
// }

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub struct ScriptCode {
//     pub bytes: Vec<u8>,
// }

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub enum BlockHeader {
//     /// V1 BlockHeader
//     V1(BlockHeaderV1),
//     V2(BlockHeaderV1),
// }

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub struct BlockHeaderV1 {
//     pub application: ApplicationHeader,
//     pub consensus: ConsensusHeader,
//     metadata: Option<Box<BlockHeaderMetadata>>,
// }

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub struct BlockHeaderMetadata {
//     /// Hash of the header.
//     id: u64,
// }

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub struct ApplicationHeader {
//     pub da_height: u32,
//     pub consensus_parameters_version: u32,
//     pub state_transition_bytecode_version: u32,
//     pub generated: GeneratedApplicationFields,
// }

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub struct GeneratedApplicationFields {
//     /// Number of transactions in this block.
//     pub transactions_count: u16,
//     /// Number of message receipts in this block.
//     pub message_receipt_count: u32,
//     /// Merkle root of transactions.
//     pub transactions_root: Vec<u8>,
//     /// Merkle root of message receipts in this block.
//     pub message_outbox_root: Vec<u8>,
//     /// Root hash of all imported events from L1
//     pub event_inbox_root: Vec<u8>,
// }

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub struct ConsensusHeader {
//     /// Merkle root of all previous block header hashes.
//     pub prev_root: Vec<u8>,
//     /// Fuel block height.
//     pub height: u64,
//     /// The block producer time.
//     pub time: u64,
//     /// generated consensus fields.
//     pub generated: GeneratedConsensusFields,
// }

// #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub struct GeneratedConsensusFields {
//     /// Hash of the application header.
//     pub application_hash: Vec<u8>,
// }
