use std::str::FromStr;

use fuel_data_parser::DataEncoder;
use fuel_streams_types::*;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    derive_more::Display,
    derive_more::IsVariant,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    #[default]
    #[display("imported")]
    Imported,
    #[display("consumed")]
    Consumed,
}

impl TryFrom<&str> for MessageType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "imported" => Ok(MessageType::Imported),
            "consumed" => Ok(MessageType::Consumed),
            _ => Err(format!("Invalid message type: {}", s)),
        }
    }
}

impl_enum_string_serialization!(MessageType, "message_type");

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Message {
    /// The message type
    pub r#type: MessageType,
    /// Account that sent the message from the da layer
    pub sender: Address,
    /// Fuel account receiving the message
    pub recipient: Address,
    /// Nonce must be unique. It's used to prevent replay attacks
    pub nonce: Nonce,
    /// The amount of the base asset of Fuel chain sent along this message
    pub amount: Word,
    /// Arbitrary message data
    pub data: HexData,
    /// The block height from the parent da layer that originated this message
    pub da_height: DaBlockHeight,
    /// The block height from the fuel chain that originated this message
    pub block_height: BlockHeight,
    /// The index of the message in the block
    pub message_index: u32,
}

impl Message {
    pub fn new(
        block_height: BlockHeight,
        message_index: u32,
        event: FuelCoreExecutorEvent,
    ) -> Option<Self> {
        match event {
            FuelCoreExecutorEvent::MessageImported(message) => Some(Message {
                r#type: MessageType::Imported,
                sender: message.sender().to_owned().into(),
                recipient: message.recipient().to_owned().into(),
                nonce: message.nonce().to_owned().into(),
                amount: message.amount().to_owned().into(),
                data: message.data().to_owned().into(),
                da_height: message.da_height().to_owned().into(),
                block_height,
                message_index,
            }),
            FuelCoreExecutorEvent::MessageConsumed(message) => Some(Message {
                r#type: MessageType::Consumed,
                sender: message.sender().to_owned().into(),
                recipient: message.recipient().to_owned().into(),
                nonce: message.nonce().to_owned().into(),
                amount: message.amount().to_owned().into(),
                data: message.data().to_owned().into(),
                da_height: message.da_height().to_owned().into(),
                block_height,
                message_index,
            }),
            _ => None,
        }
    }
}

impl DataEncoder for Message {}

#[cfg(any(test, feature = "test-helpers"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockMessage;

#[cfg(any(test, feature = "test-helpers"))]
impl MockMessage {
    pub fn imported() -> Message {
        Message {
            r#type: MessageType::Imported,
            sender: Address::random(),
            recipient: Address::random(),
            nonce: Nonce::random(),
            amount: 100.into(),
            data: HexData::random(),
            da_height: 0.into(),
            block_height: BlockHeight::random(),
            message_index: 0,
        }
    }

    pub fn consumed() -> Message {
        Message {
            r#type: MessageType::Consumed,
            sender: Address::random(),
            recipient: Address::random(),
            nonce: Nonce::random(),
            amount: 100.into(),
            data: HexData::random(),
            da_height: 0.into(),
            block_height: BlockHeight::random(),
            message_index: 0,
        }
    }

    pub fn all() -> Vec<Message> {
        vec![Self::imported(), Self::consumed()]
    }
}
