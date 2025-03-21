use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use sea_query::{Condition, Expr, Iden};
use serde::{Deserialize, Serialize};

use super::{types::*, InputDbItem};
use crate::queryable::{HasPagination, QueryPagination, Queryable};

#[allow(dead_code)]
#[derive(Iden)]
pub enum Inputs {
    #[iden = "inputs"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "subject"]
    Subject,
    #[iden = "value"]
    Value,
    #[iden = "block_height"]
    BlockHeight,
    #[iden = "tx_id"]
    TxId,
    #[iden = "tx_index"]
    TxIndex,
    #[iden = "input_index"]
    InputIndex,
    #[iden = "input_type"]
    InputType,
    #[iden = "owner_id"]
    InputOwnerId,
    #[iden = "asset_id"]
    InputAssetId,
    #[iden = "contract_id"]
    InputContractId,
    #[iden = "sender_address"]
    InputSenderAddress,
    #[iden = "recipient_address"]
    InputRecipientAddress,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "published_at"]
    PublishedAt,
}

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct InputsQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<i32>,
    pub input_type: Option<InputType>,
    pub block_height: Option<BlockHeight>,
    pub owner_id: Option<Address>, // for coin inputs
    pub asset_id: Option<AssetId>, // for coin inputs
    pub contract_id: Option<ContractId>, // for contract inputs
    pub sender_address: Option<Address>, // for message inputs
    pub recipient_address: Option<Address>, // for message inputs
    #[serde(flatten)]
    pub pagination: QueryPagination,
    pub address: Option<Address>, // for the accounts endpoint
}

impl InputsQuery {
    pub fn set_address(&mut self, address: &str) {
        self.address = Some(Address::from(address));
    }

    pub fn set_contract_id(&mut self, contract_id: &str) {
        self.contract_id = Some(ContractId::from(contract_id));
    }

    pub fn set_block_height(&mut self, height: u64) {
        self.block_height = Some(height.into());
    }

    pub fn set_tx_id(&mut self, tx_id: &str) {
        self.tx_id = Some(tx_id.into());
    }

    pub fn set_input_type(&mut self, input_type: Option<InputType>) {
        self.input_type = input_type;
    }
}

#[async_trait::async_trait]
impl Queryable for InputsQuery {
    type Record = InputDbItem;
    type Table = Inputs;
    type PaginationColumn = Inputs;

    fn table() -> Self::Table {
        Inputs::Table
    }

    fn pagination_column() -> Self::PaginationColumn {
        Inputs::Id
    }

    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }

    fn build_condition(&self) -> Condition {
        let mut condition = Condition::all();

        // handle address query
        if let Some(address) = &self.address {
            match self.input_type {
                Some(InputType::Coin) => {
                    condition = condition.add(
                        Expr::col(Inputs::InputOwnerId)
                            .eq(address.to_string())
                            .eq(Expr::col(Inputs::InputAssetId)
                                .eq(address.to_string())),
                    );
                }
                Some(InputType::Contract) => {
                    condition = condition.add(
                        Expr::col(Inputs::InputContractId)
                            .eq(address.to_string()),
                    );
                }
                Some(InputType::Message) => {
                    condition = condition.add(
                        Expr::col(Inputs::InputSenderAddress)
                            .eq(address.to_string())
                            .or(Expr::col(Inputs::InputRecipientAddress)
                                .eq(address.to_string())),
                    );
                }
                _ => {
                    condition = condition.add(
                        Expr::col(Inputs::InputOwnerId)
                            .eq(address.to_string())
                            .or(Expr::col(Inputs::InputAssetId)
                                .eq(address.to_string()))
                            .or(Expr::col(Inputs::InputContractId)
                                .eq(address.to_string()))
                            .or(Expr::col(Inputs::InputSenderAddress)
                                .eq(address.to_string()))
                            .or(Expr::col(Inputs::InputRecipientAddress)
                                .eq(address.to_string())),
                    );
                }
            }
        }

        if let Some(block_height) = &self.block_height {
            condition = condition
                .add(Expr::col(Inputs::BlockHeight).eq(**block_height));
        }

        if let Some(tx_id) = &self.tx_id {
            condition =
                condition.add(Expr::col(Inputs::TxId).eq(tx_id.to_string()));
        }

        if let Some(tx_index) = &self.tx_index {
            condition = condition.add(Expr::col(Inputs::TxIndex).eq(*tx_index));
        }

        if let Some(input_index) = &self.input_index {
            condition =
                condition.add(Expr::col(Inputs::InputIndex).eq(*input_index));
        }

        if let Some(input_type) = &self.input_type {
            condition = condition
                .add(Expr::col(Inputs::InputType).eq(input_type.to_string()));
        }

        // unique conditions
        if let Some(owner_id) = &self.owner_id {
            condition = condition
                .add(Expr::col(Inputs::InputOwnerId).eq(owner_id.to_string()));
        }

        if let Some(asset_id) = &self.asset_id {
            condition = condition
                .add(Expr::col(Inputs::InputAssetId).eq(asset_id.to_string()));
        }

        if let Some(contract_id) = &self.contract_id {
            condition = condition.add(
                Expr::col(Inputs::InputContractId).eq(contract_id.to_string()),
            );
        }

        if let Some(sender_address) = &self.sender_address {
            condition = condition.add(
                Expr::col(Inputs::InputSenderAddress)
                    .eq(sender_address.to_string()),
            );
        }

        if let Some(recipient_address) = &self.recipient_address {
            condition = condition.add(
                Expr::col(Inputs::InputRecipientAddress)
                    .eq(recipient_address.to_string()),
            );
        }

        condition
    }
}

impl HasPagination for InputsQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}

#[cfg(test)]
mod test {
    use fuel_streams_types::{Address, AssetId, BlockHeight, ContractId, TxId};
    use pretty_assertions::assert_eq;

    use crate::{
        inputs::queryable::{InputType, InputsQuery},
        queryable::Queryable,
    };

    // Test constants
    const AFTER_POINTER: i32 = 10000;
    const BEFORE_POINTER: i32 = 20000;
    const FIRST_POINTER: i32 = 100;
    const LAST_POINTER: i32 = 100;
    const TEST_BLOCK_HEIGHT: i32 = 55;
    const TEST_TX_INDEX: u32 = 3;
    const TEST_INPUT_INDEX: i32 = 7;
    const TEST_TX_ID: &str =
        "0x0101010101010101010101010101010101010101010101010101010101010101";
    const TEST_OWNER_ID: &str =
        "0x0202020202020202020202020202020202020202020202020202020202020202";
    const TEST_ASSET_ID: &str =
        "0x0303030303030303030303030303030303030303030303030303030303030303";
    const TEST_CONTRACT_ID: &str =
        "0x0404040404040404040404040404040404040404040404040404040404040404";
    const TEST_ADDRESS: &str =
        "0x0505050505050505050505050505050505050505050505050505050505050505";
    const _ASSET_ID: &str =
        "0x0606060606060606060606060606060606060606060606060606060606060606";

    #[test]
    fn test_sql_with_fixed_conds() {
        // Test 1: basic query with tx_id, block_height and input_type
        let query = InputsQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: Some(BlockHeight::from(TEST_BLOCK_HEIGHT)),
            input_type: Some(InputType::Coin),
            tx_index: None,
            input_index: None,
            owner_id: None,
            asset_id: None,
            contract_id: None,
            sender_address: None,
            recipient_address: None,
            pagination: Default::default(),
            address: None,
        };

        assert_eq!(
            query.query_to_string(),
            format!("SELECT * FROM \"inputs\" WHERE \"block_height\" = {} AND \"tx_id\" = '{}' AND \"input_type\" = 'coin'",
                TEST_BLOCK_HEIGHT, TEST_TX_ID)
        );

        // Test 2: coin input with owner and asset
        let coin_query = InputsQuery {
            tx_id: None,
            block_height: None,
            input_type: Some(InputType::Coin),
            tx_index: None,
            input_index: None,
            owner_id: Some(Address::from(TEST_OWNER_ID)),
            asset_id: Some(AssetId::from(TEST_ASSET_ID)),
            contract_id: None,
            sender_address: None,
            recipient_address: None,
            pagination: (None, None, Some(FIRST_POINTER), None).into(),
            address: None,
        };

        assert_eq!(
            coin_query.query_to_string(),
            format!("SELECT * FROM \"inputs\" WHERE \"input_type\" = 'coin' AND \"owner_id\" = '{}' AND \"asset_id\" = '{}' ORDER BY \"id\" ASC LIMIT {}",
                TEST_OWNER_ID, TEST_ASSET_ID, FIRST_POINTER)
        );

        // Test 3: contract input with contract_id and pagination
        let contract_query = InputsQuery {
            tx_id: None,
            block_height: None,
            input_type: Some(InputType::Contract),
            tx_index: None,
            input_index: None,
            owner_id: None,
            asset_id: None,
            contract_id: Some(ContractId::from(TEST_CONTRACT_ID)),
            sender_address: None,
            recipient_address: None,
            pagination: (Some(AFTER_POINTER), None, None, Some(LAST_POINTER))
                .into(),
            address: None,
        };

        assert_eq!(
            contract_query.query_to_string(),
            format!("SELECT * FROM \"inputs\" WHERE \"input_type\" = 'contract' AND \"contract_id\" = '{}' AND \"id\" > {} ORDER BY \"id\" DESC LIMIT {}",
                TEST_CONTRACT_ID, AFTER_POINTER, LAST_POINTER)
        );

        // Test 4: message input with addresses and before condition
        let message_query = InputsQuery {
            tx_id: None,
            block_height: None,
            input_type: Some(InputType::Message),
            tx_index: None,
            input_index: None,
            owner_id: None,
            asset_id: None,
            contract_id: None,
            sender_address: Some(Address::from(TEST_ADDRESS)),
            recipient_address: Some(Address::from(TEST_ADDRESS)),
            pagination: (None, Some(BEFORE_POINTER), Some(FIRST_POINTER), None)
                .into(),
            address: None,
        };

        assert_eq!(
            message_query.query_to_string(),
            format!("SELECT * FROM \"inputs\" WHERE \"input_type\" = 'message' AND \"sender_address\" = '{}' AND \"recipient_address\" = '{}' AND \"id\" < {} ORDER BY \"id\" ASC LIMIT {}",
                TEST_ADDRESS, TEST_ADDRESS, BEFORE_POINTER, FIRST_POINTER)
        );

        // Test 5: detailed input query with indices
        let detailed_query = InputsQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: None,
            input_type: None,
            tx_index: Some(TEST_TX_INDEX),
            input_index: Some(TEST_INPUT_INDEX),
            owner_id: None,
            asset_id: None,
            contract_id: None,
            sender_address: None,
            recipient_address: None,
            pagination: Default::default(),
            address: None,
        };

        assert_eq!(
            detailed_query.query_to_string(),
            format!("SELECT * FROM \"inputs\" WHERE \"tx_id\" = '{}' AND \"tx_index\" = {} AND \"input_index\" = {}",
                TEST_TX_ID, TEST_TX_INDEX, TEST_INPUT_INDEX)
        );
    }

    #[test]
    fn test_inputs_query_from_query_string() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&txIndex={}&inputIndex={}&inputType=Coin&blockHeight={}&ownerId={}&assetId={}&contractId={}&senderAddress={}&recipientAddress={}&after={}&before={}&first={}&last={}",
            TEST_TX_ID,
            TEST_TX_INDEX,
            TEST_INPUT_INDEX,
            TEST_BLOCK_HEIGHT,
            TEST_OWNER_ID,
            TEST_ASSET_ID,
            TEST_CONTRACT_ID,
            TEST_ADDRESS,
            TEST_ADDRESS,
            AFTER_POINTER,
            BEFORE_POINTER,
            FIRST_POINTER,
            LAST_POINTER
        );

        let query: InputsQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, Some(TEST_TX_INDEX));
        assert_eq!(query.input_index, Some(TEST_INPUT_INDEX));
        assert_eq!(query.input_type, Some(InputType::Coin));
        assert_eq!(
            query.block_height,
            Some(BlockHeight::from(TEST_BLOCK_HEIGHT))
        );
        assert_eq!(query.owner_id, Some(Address::from(TEST_OWNER_ID)));
        assert_eq!(query.asset_id, Some(AssetId::from(TEST_ASSET_ID)));
        assert_eq!(query.contract_id, Some(ContractId::from(TEST_CONTRACT_ID)));
        assert_eq!(query.sender_address, Some(Address::from(TEST_ADDRESS)));
        assert_eq!(query.recipient_address, Some(Address::from(TEST_ADDRESS)));
        assert_eq!(query.pagination().after, Some(AFTER_POINTER));
        assert_eq!(query.pagination().before, Some(BEFORE_POINTER));
        assert_eq!(query.pagination().first, Some(FIRST_POINTER));
        assert_eq!(query.pagination().last, Some(LAST_POINTER));
    }

    #[test]
    fn test_inputs_query_from_query_string_partial() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&inputType=Message&senderAddress={}&recipientAddress={}&after={}&first={}",
            TEST_TX_ID,
            TEST_ADDRESS,
            TEST_ADDRESS,
            AFTER_POINTER,
            FIRST_POINTER
        );

        let query: InputsQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, None);
        assert_eq!(query.input_index, None);
        assert_eq!(query.input_type, Some(InputType::Message));
        assert_eq!(query.block_height, None);
        assert_eq!(query.owner_id, None);
        assert_eq!(query.asset_id, None);
        assert_eq!(query.contract_id, None);
        assert_eq!(query.sender_address, Some(Address::from(TEST_ADDRESS)));
        assert_eq!(query.recipient_address, Some(Address::from(TEST_ADDRESS)));
        assert_eq!(query.pagination().after, Some(AFTER_POINTER));
        assert_eq!(query.pagination().before, None);
        assert_eq!(query.pagination().first, Some(FIRST_POINTER));
        assert_eq!(query.pagination().last, None);
    }
}
