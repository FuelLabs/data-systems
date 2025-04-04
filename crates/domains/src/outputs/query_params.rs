use fuel_streams_types::*;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use crate::infra::{
    repository::{HasPagination, QueryPagination, QueryParamsBuilder},
    QueryOptions,
};

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub struct OutputsQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<i32>,
    pub output_index: Option<i32>,
    pub output_type: Option<OutputType>,
    pub block_height: Option<BlockHeight>,
    // coin, change, and variable outputs
    pub to_address: Option<Address>,
    pub asset_id: Option<AssetId>,
    // contract and contract_created outputs
    pub contract_id: Option<ContractId>,
    // the accounts endpoint
    pub address: Option<Address>,
    #[serde(flatten)]
    pub pagination: QueryPagination,
    #[serde(flatten)]
    pub options: QueryOptions,
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

impl QueryParamsBuilder for OutputsQuery {
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
        query_builder.push("SELECT * FROM outputs");

        if let Some(tx_id) = &self.tx_id {
            conditions.push(format!("tx_id = '{}'", tx_id));
        }

        if let Some(tx_index) = &self.tx_index {
            conditions.push(format!("tx_index = {}", tx_index));
        }

        if let Some(output_index) = &self.output_index {
            conditions.push(format!("output_index = {}", output_index));
        }

        if let Some(output_type) = &self.output_type {
            conditions.push(format!("type = '{}'", output_type));
        }

        if let Some(block_height) = &self.block_height {
            conditions.push(format!("block_height = {}", block_height));
        }

        if let Some(to_address) = &self.to_address {
            conditions.push(format!("to_address = '{}'", to_address));
        }

        if let Some(asset_id) = &self.asset_id {
            conditions.push(format!("asset_id = '{}'", asset_id));
        }

        if let Some(contract_id) = &self.contract_id {
            conditions.push(format!("contract_id = '{}'", contract_id));
        }

        if let Some(address) = &self.address {
            conditions.push(format!("to_address = '{}'", address));
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

impl HasPagination for OutputsQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}
