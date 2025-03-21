use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use sea_query::{Condition, Expr, Iden};
use serde::{Deserialize, Serialize};

use super::{OutputDbItem, OutputType};
use crate::queryable::{HasPagination, QueryPagination, Queryable};

#[allow(dead_code)]
#[derive(Iden)]
pub enum Outputs {
    #[iden = "outputs"]
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
    #[iden = "output_index"]
    OutputIndex,
    #[iden = "output_type"]
    OutputType,
    #[iden = "to_address"]
    OutputToAddress,
    #[iden = "asset_id"]
    OutputAssetId,
    #[iden = "contract_id"]
    OutputContractId,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "published_at"]
    PublishedAt,
}

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct OutputsQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub output_index: Option<i32>,
    pub output_type: Option<OutputType>,
    pub block_height: Option<BlockHeight>,
    pub to_address: Option<Address>, // for coin, change, and variable outputs
    pub asset_id: Option<AssetId>,   // for coin, change, and variable outputs
    pub contract_id: Option<ContractId>, /* for contract and contract_created outputs */
    #[serde(flatten)]
    pub pagination: QueryPagination,
    pub address: Option<Address>, // for the accounts endpoint
}

impl OutputsQuery {
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

    pub fn set_output_type(&mut self, output_type: Option<OutputType>) {
        self.output_type = output_type;
    }
}

#[async_trait::async_trait]
impl Queryable for OutputsQuery {
    type Record = OutputDbItem;
    type Table = Outputs;
    type PaginationColumn = Outputs;

    fn table() -> Self::Table {
        Outputs::Table
    }

    fn pagination_column() -> Self::PaginationColumn {
        Outputs::Id
    }

    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }

    fn build_condition(&self) -> Condition {
        let mut condition = Condition::all();

        // handle address query
        if let Some(address) = &self.address {
            match self.output_type {
                Some(OutputType::Coin)
                | Some(OutputType::Variable)
                | Some(OutputType::Change) => {
                    condition = condition.add(
                        Expr::col(Outputs::OutputToAddress)
                            .eq(address.to_string())
                            .or(Expr::col(Outputs::OutputAssetId)
                                .eq(address.to_string())),
                    );
                }
                Some(OutputType::Contract)
                | Some(OutputType::ContractCreated) => {
                    condition = condition.add(
                        Expr::col(Outputs::OutputContractId)
                            .eq(address.to_string()),
                    );
                }
                _ => {
                    condition = condition.add(
                        Expr::col(Outputs::OutputToAddress)
                            .eq(address.to_string())
                            .or(Expr::col(Outputs::OutputAssetId)
                                .eq(address.to_string()))
                            .or(Expr::col(Outputs::OutputContractId)
                                .eq(address.to_string())),
                    );
                }
            }
        }

        if let Some(block_height) = &self.block_height {
            condition = condition
                .add(Expr::col(Outputs::BlockHeight).eq(**block_height));
        }

        if let Some(tx_id) = &self.tx_id {
            condition =
                condition.add(Expr::col(Outputs::TxId).eq(tx_id.to_string()));
        }

        if let Some(tx_index) = &self.tx_index {
            condition =
                condition.add(Expr::col(Outputs::TxIndex).eq(*tx_index));
        }

        if let Some(output_index) = &self.output_index {
            condition = condition
                .add(Expr::col(Outputs::OutputIndex).eq(*output_index));
        }

        if let Some(output_type) = &self.output_type {
            condition = condition.add(
                Expr::col(Outputs::OutputType).eq(output_type.to_string()),
            );
        }

        // unique conditions
        if let Some(to_address) = &self.to_address {
            condition = condition.add(
                Expr::col(Outputs::OutputToAddress).eq(to_address.to_string()),
            );
        }

        if let Some(asset_id) = &self.asset_id {
            condition = condition.add(
                Expr::col(Outputs::OutputAssetId).eq(asset_id.to_string()),
            );
        }

        if let Some(contract_id) = &self.contract_id {
            condition = condition.add(
                Expr::col(Outputs::OutputContractId)
                    .eq(contract_id.to_string()),
            );
        }

        condition
    }
}

impl HasPagination for OutputsQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}

#[cfg(test)]
mod test {
    use fuel_streams_types::{Address, AssetId, BlockHeight, ContractId, TxId};
    use pretty_assertions::assert_eq;

    use crate::{
        outputs::queryable::{OutputType, OutputsQuery},
        queryable::Queryable,
    };

    // Test constants
    const AFTER_POINTER: i32 = 10000;
    const BEFORE_POINTER: i32 = 20000;
    const FIRST_POINTER: i32 = 100;
    const LAST_POINTER: i32 = 100;
    const TEST_BLOCK_HEIGHT: i32 = 55;
    const TEST_TX_INDEX: u32 = 3;
    const TEST_OUTPUT_INDEX: i32 = 7;
    const TEST_TX_ID: &str =
        "0x0101010101010101010101010101010101010101010101010101010101010101";
    const TEST_ADDRESS: &str =
        "0x0202020202020202020202020202020202020202020202020202020202020202";
    const TEST_ASSET_ID: &str =
        "0x0303030303030303030303030303030303030303030303030303030303030303";
    const TEST_CONTRACT_ID: &str =
        "0x0404040404040404040404040404040404040404040404040404040404040404";

    #[test]
    fn test_sql_with_fixed_conds() {
        // Test 1: basic query with tx_id, block_height and output_type
        let query = OutputsQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: Some(BlockHeight::from(TEST_BLOCK_HEIGHT)),
            output_type: Some(OutputType::Coin),
            tx_index: None,
            output_index: None,
            to_address: None,
            asset_id: None,
            contract_id: None,
            pagination: Default::default(),
            address: None,
        };

        assert_eq!(
            query.query_to_string(),
            format!("SELECT * FROM \"outputs\" WHERE \"block_height\" = {} AND \"tx_id\" = '{}' AND \"output_type\" = 'coin'",
                TEST_BLOCK_HEIGHT, TEST_TX_ID)
        );

        // Test 2: coin output with to_address and asset_id
        let coin_query = OutputsQuery {
            tx_id: None,
            block_height: None,
            output_type: Some(OutputType::Coin),
            tx_index: None,
            output_index: None,
            to_address: Some(Address::from(TEST_ADDRESS)),
            asset_id: Some(AssetId::from(TEST_ASSET_ID)),
            contract_id: None,
            pagination: (None, None, Some(FIRST_POINTER), None).into(),
            address: None,
        };

        assert_eq!(
            coin_query.query_to_string(),
            format!("SELECT * FROM \"outputs\" WHERE \"output_type\" = 'coin' AND \"to_address\" = '{}' AND \"asset_id\" = '{}' ORDER BY \"id\" ASC LIMIT {}",
                TEST_ADDRESS, TEST_ASSET_ID, FIRST_POINTER)
        );

        // Test 3: contract output with contract_id and pagination
        let contract_query = OutputsQuery {
            tx_id: None,
            block_height: None,
            output_type: Some(OutputType::Contract),
            tx_index: None,
            output_index: None,
            to_address: None,
            asset_id: None,
            contract_id: Some(ContractId::from(TEST_CONTRACT_ID)),
            pagination: (Some(AFTER_POINTER), None, None, Some(LAST_POINTER))
                .into(),
            address: None,
        };

        assert_eq!(
            contract_query.query_to_string(),
            format!("SELECT * FROM \"outputs\" WHERE \"output_type\" = 'contract' AND \"contract_id\" = '{}' AND \"id\" > {} ORDER BY \"id\" DESC LIMIT {}",
                TEST_CONTRACT_ID, AFTER_POINTER, LAST_POINTER)
        );

        // Test 4: change output with to_address and before condition
        let change_query = OutputsQuery {
            tx_id: None,
            block_height: None,
            output_type: Some(OutputType::Change),
            tx_index: None,
            output_index: None,
            to_address: Some(Address::from(TEST_ADDRESS)),
            asset_id: Some(AssetId::from(TEST_ASSET_ID)),
            contract_id: None,
            pagination: (None, Some(BEFORE_POINTER), Some(FIRST_POINTER), None)
                .into(),
            address: None,
        };

        assert_eq!(
            change_query.query_to_string(),
            format!("SELECT * FROM \"outputs\" WHERE \"output_type\" = 'change' AND \"to_address\" = '{}' AND \"asset_id\" = '{}' AND \"id\" < {} ORDER BY \"id\" ASC LIMIT {}",
                TEST_ADDRESS, TEST_ASSET_ID, BEFORE_POINTER, FIRST_POINTER)
        );

        // Test 5: detailed output query with indices
        let detailed_query = OutputsQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: None,
            output_type: None,
            tx_index: Some(TEST_TX_INDEX),
            output_index: Some(TEST_OUTPUT_INDEX),
            to_address: None,
            asset_id: None,
            contract_id: None,
            pagination: Default::default(),
            address: None,
        };

        assert_eq!(
            detailed_query.query_to_string(),
            format!("SELECT * FROM \"outputs\" WHERE \"tx_id\" = '{}' AND \"tx_index\" = {} AND \"output_index\" = {}",
                TEST_TX_ID, TEST_TX_INDEX, TEST_OUTPUT_INDEX)
        );
    }

    #[test]
    fn test_outputs_query_from_query_string() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&txIndex={}&outputIndex={}&outputType=Coin&blockHeight={}&toAddress={}&assetId={}&contractId={}&after={}&before={}&first={}&last={}",
            TEST_TX_ID,
            TEST_TX_INDEX,
            TEST_OUTPUT_INDEX,
            TEST_BLOCK_HEIGHT,
            TEST_ADDRESS,
            TEST_ASSET_ID,
            TEST_CONTRACT_ID,
            AFTER_POINTER,
            BEFORE_POINTER,
            FIRST_POINTER,
            LAST_POINTER
        );

        let query: OutputsQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, Some(TEST_TX_INDEX));
        assert_eq!(query.output_index, Some(TEST_OUTPUT_INDEX));
        assert_eq!(query.output_type, Some(OutputType::Coin));
        assert_eq!(
            query.block_height,
            Some(BlockHeight::from(TEST_BLOCK_HEIGHT))
        );
        assert_eq!(query.to_address, Some(Address::from(TEST_ADDRESS)));
        assert_eq!(query.asset_id, Some(AssetId::from(TEST_ASSET_ID)));
        assert_eq!(query.contract_id, Some(ContractId::from(TEST_CONTRACT_ID)));
        assert_eq!(query.pagination.after(), Some(AFTER_POINTER));
        assert_eq!(query.pagination.before(), Some(BEFORE_POINTER));
        assert_eq!(query.pagination.first(), Some(FIRST_POINTER));
        assert_eq!(query.pagination.last(), Some(LAST_POINTER));
    }

    #[test]
    fn test_outputs_query_from_query_string_partial() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&outputType=ContractCreated&contractId={}&after={}&first={}",
            TEST_TX_ID,
            TEST_CONTRACT_ID,
            AFTER_POINTER,
            FIRST_POINTER
        );

        let query: OutputsQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, None);
        assert_eq!(query.output_index, None);
        assert_eq!(query.output_type, Some(OutputType::ContractCreated));
        assert_eq!(query.block_height, None);
        assert_eq!(query.to_address, None);
        assert_eq!(query.asset_id, None);
        assert_eq!(query.contract_id, Some(ContractId::from(TEST_CONTRACT_ID)));
        assert_eq!(query.pagination.after(), Some(AFTER_POINTER));
        assert_eq!(query.pagination.before(), None);
        assert_eq!(query.pagination.first(), Some(FIRST_POINTER));
        assert_eq!(query.pagination.last(), None);
    }
}
