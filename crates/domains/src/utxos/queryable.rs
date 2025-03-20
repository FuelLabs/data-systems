use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use sea_query::{Condition, Expr, Iden};
use serde::{Deserialize, Serialize};

use super::UtxoDbItem;
use crate::{
    inputs::InputType,
    queryable::{HasPagination, QueryPagination, Queryable},
};

#[allow(dead_code)]
#[derive(Iden)]
pub enum Utxos {
    #[iden = "utxos"]
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
    #[iden = "utxo_type"]
    UtxoType,
    #[iden = "utxo_id"]
    UtxoId,
    #[iden = "contract_id"]
    ContractId,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "published_at"]
    PublishedAt,
}

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct UtxosQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<i32>,
    pub utxo_type: Option<InputType>,
    pub block_height: Option<BlockHeight>,
    pub utxo_id: Option<HexData>,
    #[serde(flatten)]
    pub pagination: QueryPagination,
    pub contract_id: Option<ContractId>, // for the contracts endpoint
    pub address: Option<Address>,        // for the accounts endpoint
}

impl UtxosQuery {
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

    pub fn set_utxo_type(&mut self, utxo_type: Option<InputType>) {
        self.utxo_type = utxo_type;
    }
}

#[async_trait::async_trait]
impl Queryable for UtxosQuery {
    type Record = UtxoDbItem;
    type Table = Utxos;
    type PaginationColumn = Utxos;

    fn table() -> Self::Table {
        Utxos::Table
    }

    fn pagination_column() -> Self::PaginationColumn {
        Utxos::Id
    }

    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }

    fn build_condition(&self) -> Condition {
        let mut condition = Condition::all();

        // handle address query
        // TODO: extend db fields with separate sender, recipient as per utxo type Input::Message
        if let Some(address) = &self.address {
            match self.utxo_type {
                Some(InputType::Coin) => {
                    condition = condition
                        .add(Expr::col(Utxos::UtxoId).eq(address.to_string()));
                }
                Some(InputType::Contract) => {
                    condition = condition
                        .add(Expr::col(Utxos::UtxoId).eq(address.to_string()));
                }
                Some(InputType::Message) => {
                    condition = condition
                        .add(Expr::col(Utxos::UtxoId).eq(address.to_string()));
                }
                _ => {
                    condition = condition
                        .add(Expr::col(Utxos::UtxoId).eq(address.to_string()));
                }
            }
        }

        if let Some(block_height) = &self.block_height {
            condition =
                condition.add(Expr::col(Utxos::BlockHeight).eq(**block_height));
        }

        if let Some(tx_id) = &self.tx_id {
            condition =
                condition.add(Expr::col(Utxos::TxId).eq(tx_id.to_string()));
        }

        if let Some(tx_index) = &self.tx_index {
            condition = condition.add(Expr::col(Utxos::TxIndex).eq(*tx_index));
        }

        if let Some(input_index) = &self.input_index {
            condition =
                condition.add(Expr::col(Utxos::InputIndex).eq(*input_index));
        }

        if let Some(utxo_type) = &self.utxo_type {
            condition = condition
                .add(Expr::col(Utxos::UtxoType).eq(utxo_type.to_string()));
        }

        // unique conditions
        if let Some(utxo_id) = &self.utxo_id {
            condition =
                condition.add(Expr::col(Utxos::UtxoId).eq(utxo_id.to_string()));
        }

        if let Some(contract_id) = &self.contract_id {
            condition = condition
                .add(Expr::col(Utxos::ContractId).eq(contract_id.to_string()));
        }

        condition
    }
}

impl HasPagination for UtxosQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}

#[cfg(test)]
mod test {
    use fuel_streams_types::{BlockHeight, HexData, TxId};
    use pretty_assertions::assert_eq;

    use crate::{
        inputs::InputType,
        queryable::Queryable,
        utxos::queryable::UtxosQuery,
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
    const TEST_UTXO_ID: &str =
        "0x0202020202020202020202020202020202020202020202020202020202020202";

    #[test]
    fn test_sql_with_fixed_conds() {
        // Test 1: basic query with tx_id, block_height and utxo_type
        let query = UtxosQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: Some(BlockHeight::from(TEST_BLOCK_HEIGHT)),
            utxo_type: Some(InputType::Coin),
            tx_index: None,
            input_index: None,
            utxo_id: None,
            pagination: Default::default(),
            address: None,
            contract_id: None,
        };

        assert_eq!(
            query.query_to_string(),
            format!("SELECT * FROM \"utxos\" WHERE \"block_height\" = {} AND \"tx_id\" = '{}' AND \"utxo_type\" = 'coin'",
                TEST_BLOCK_HEIGHT, TEST_TX_ID)
        );

        // Test 2: query with utxo_id and first pagination
        let utxo_id_query = UtxosQuery {
            tx_id: None,
            block_height: None,
            utxo_type: None,
            tx_index: None,
            input_index: None,
            utxo_id: Some(HexData::from(TEST_UTXO_ID)),
            pagination: (None, None, Some(FIRST_POINTER), None).into(),
            address: None,
            contract_id: None,
        };

        assert_eq!(
            utxo_id_query.query_to_string(),
            format!("SELECT * FROM \"utxos\" WHERE \"utxo_id\" = '{}' ORDER BY \"block_height\" ASC LIMIT {}",
                TEST_UTXO_ID, FIRST_POINTER)
        );

        // Test 3: query with transaction indices and range conditions
        let indices_query = UtxosQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: None,
            utxo_type: None,
            tx_index: Some(TEST_TX_INDEX),
            input_index: Some(TEST_INPUT_INDEX),
            utxo_id: None,
            pagination: (Some(AFTER_POINTER), None, None, Some(LAST_POINTER))
                .into(),
            address: None,
            contract_id: None,
        };

        assert_eq!(
            indices_query.query_to_string(),
            format!("SELECT * FROM \"utxos\" WHERE \"tx_id\" = '{}' AND \"tx_index\" = {} AND \"input_index\" = {} AND \"block_height\" > {} ORDER BY \"block_height\" DESC LIMIT {}",
                TEST_TX_ID, TEST_TX_INDEX, TEST_INPUT_INDEX, AFTER_POINTER, LAST_POINTER)
        );

        // Test 4: query with utxo_type and before condition
        let type_query = UtxosQuery {
            tx_id: None,
            block_height: None,
            utxo_type: Some(InputType::Message),
            tx_index: None,
            input_index: None,
            utxo_id: None,
            pagination: (None, Some(BEFORE_POINTER), Some(FIRST_POINTER), None)
                .into(),
            address: None,
            contract_id: None,
        };

        assert_eq!(
            type_query.query_to_string(),
            format!("SELECT * FROM \"utxos\" WHERE \"utxo_type\" = 'message' AND \"block_height\" < {} ORDER BY \"block_height\" ASC LIMIT {}",
                BEFORE_POINTER, FIRST_POINTER)
        );

        // Test 5: query with block_height and all parameters
        let complex_query = UtxosQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: Some(BlockHeight::from(TEST_BLOCK_HEIGHT)),
            utxo_type: Some(InputType::Contract),
            tx_index: Some(TEST_TX_INDEX),
            input_index: Some(TEST_INPUT_INDEX),
            utxo_id: Some(HexData::from(TEST_UTXO_ID)),
            pagination: (
                Some(AFTER_POINTER),
                Some(BEFORE_POINTER),
                Some(FIRST_POINTER),
                None,
            )
                .into(),
            address: None,
            contract_id: None,
        };

        assert_eq!(
            complex_query.query_to_string(),
            format!("SELECT * FROM \"utxos\" WHERE \"block_height\" = {} AND \"tx_id\" = '{}' AND \"tx_index\" = {} AND \"input_index\" = {} AND \"utxo_type\" = 'contract' AND \"utxo_id\" = '{}' AND \"block_height\" > {} AND \"block_height\" < {} ORDER BY \"block_height\" ASC LIMIT {}",
                TEST_BLOCK_HEIGHT, TEST_TX_ID, TEST_TX_INDEX, TEST_INPUT_INDEX, TEST_UTXO_ID, AFTER_POINTER, BEFORE_POINTER, FIRST_POINTER)
        );
    }

    #[test]
    fn test_utxos_query_from_query_string() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&txIndex={}&inputIndex={}&utxoType=Coin&blockHeight={}&utxoId={}&after={}&before={}&first={}&last={}",
            TEST_TX_ID,
            TEST_TX_INDEX,
            TEST_INPUT_INDEX,
            TEST_BLOCK_HEIGHT,
            TEST_UTXO_ID,
            AFTER_POINTER,
            BEFORE_POINTER,
            FIRST_POINTER,
            LAST_POINTER
        );

        let query: UtxosQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, Some(TEST_TX_INDEX));
        assert_eq!(query.input_index, Some(TEST_INPUT_INDEX));
        assert_eq!(query.utxo_type, Some(InputType::Coin));
        assert_eq!(
            query.block_height,
            Some(BlockHeight::from(TEST_BLOCK_HEIGHT))
        );
        assert_eq!(query.utxo_id, Some(HexData::from(TEST_UTXO_ID)));
        assert_eq!(query.pagination().after, Some(AFTER_POINTER));
        assert_eq!(query.pagination().before, Some(BEFORE_POINTER));
        assert_eq!(query.pagination().first, Some(FIRST_POINTER));
        assert_eq!(query.pagination().last, Some(LAST_POINTER));
    }

    #[test]
    fn test_utxos_query_from_query_string_partial() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&utxoType=Contract&utxoId={}&after={}&first={}",
            TEST_TX_ID, TEST_UTXO_ID, AFTER_POINTER, FIRST_POINTER
        );

        let query: UtxosQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, None);
        assert_eq!(query.input_index, None);
        assert_eq!(query.utxo_type, Some(InputType::Contract));
        assert_eq!(query.block_height, None);
        assert_eq!(query.utxo_id, Some(HexData::from(TEST_UTXO_ID)));
        assert_eq!(query.pagination().after, Some(AFTER_POINTER));
        assert_eq!(query.pagination().before, None);
        assert_eq!(query.pagination().first, Some(FIRST_POINTER));
        assert_eq!(query.pagination().last, None);
    }

    #[test]
    fn test_set_methods() {
        let mut query = UtxosQuery::default();

        // Test set_block_height
        query.set_block_height(TEST_BLOCK_HEIGHT as u64);
        assert_eq!(
            query.block_height,
            Some(BlockHeight::from(TEST_BLOCK_HEIGHT))
        );

        // Test set_tx_id
        query.set_tx_id(TEST_TX_ID);
        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));

        // Test set_utxo_type
        query.set_utxo_type(Some(InputType::Coin));
        assert_eq!(query.utxo_type, Some(InputType::Coin));

        // Test query building after setting values
        assert_eq!(
            query.query_to_string(),
            format!("SELECT * FROM \"utxos\" WHERE \"block_height\" = {} AND \"tx_id\" = '{}' AND \"utxo_type\" = 'coin'",
                TEST_BLOCK_HEIGHT, TEST_TX_ID)
        );
    }
}
