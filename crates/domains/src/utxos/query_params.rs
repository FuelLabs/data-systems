use fuel_streams_types::{Address, BlockHeight, ContractId, HexData, TxId};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use crate::{
    infra::{
        repository::{HasPagination, QueryPagination, QueryParamsBuilder},
        QueryOptions,
    },
    inputs::InputType,
};

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct UtxosQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<i32>,
    pub input_index: Option<i32>,
    pub utxo_type: Option<InputType>,
    pub block_height: Option<BlockHeight>,
    pub utxo_id: Option<HexData>,
    pub contract_id: Option<ContractId>, // for the contracts endpoint
    pub address: Option<Address>,        // for the accounts endpoint
    #[serde(flatten)]
    pub pagination: QueryPagination,
    #[serde(flatten)]
    pub options: QueryOptions,
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

impl QueryParamsBuilder for UtxosQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }

    fn pagination_mut(&mut self) -> &mut QueryPagination {
        &mut self.pagination
    }

    fn with_pagination(&mut self, pagination: &QueryPagination) {
        self.pagination = pagination.clone();
    }

    fn options(&self) -> &QueryOptions {
        &self.options
    }

    fn options_mut(&mut self) -> &mut QueryOptions {
        &mut self.options
    }

    fn with_options(&mut self, options: &QueryOptions) {
        self.options = options.clone();
    }

    fn query_builder(&self) -> QueryBuilder<'static, Postgres> {
        let mut conditions = Vec::new();
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();
        query_builder.push("SELECT * FROM utxos");

        if let Some(tx_id) = &self.tx_id {
            conditions.push(format!("tx_id = '{}'", tx_id));
        }

        if let Some(tx_index) = &self.tx_index {
            conditions.push(format!("tx_index = {}", tx_index));
        }

        if let Some(input_index) = &self.input_index {
            conditions.push(format!("input_index = {}", input_index));
        }

        if let Some(utxo_type) = &self.utxo_type {
            conditions.push(format!("utxo_type = '{}'", utxo_type));
        }

        if let Some(block_height) = &self.block_height {
            conditions.push(format!("block_height = {}", block_height));
        }

        if let Some(utxo_id) = &self.utxo_id {
            conditions.push(format!("utxo_id = '{}'", utxo_id));
        }

        if let Some(contract_id) = &self.contract_id {
            conditions.push(format!("contract_id = '{}'", contract_id));
        }

        if let Some(address) = &self.address {
            conditions.push(format!("address = '{}'", address));
        }

        Self::apply_conditions(
            &mut query_builder,
            &mut conditions,
            &self.options,
            &self.pagination,
            "cursor",
            None,
        );

        Self::apply_pagination(
            &mut query_builder,
            &self.pagination,
            "cursor",
            None,
        );

        query_builder
    }
}

impl HasPagination for UtxosQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}
