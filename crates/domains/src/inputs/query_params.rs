use fuel_streams_types::{
    Address,
    AssetId,
    BlockHeight,
    ContractId,
    InputType,
    TxId,
};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use crate::infra::{
    repository::{HasPagination, QueryPagination, QueryParamsBuilder},
    QueryOptions,
};

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
pub struct InputsQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<i32>,
    pub input_index: Option<i32>,
    pub input_type: Option<InputType>,
    pub block_height: Option<BlockHeight>,
    pub owner_id: Option<Address>, // for coin inputs
    pub asset_id: Option<AssetId>, // for coin inputs
    pub contract_id: Option<ContractId>, // for contract inputs
    pub sender_address: Option<Address>, // for message inputs
    pub recipient_address: Option<Address>, // for message inputs
    pub address: Option<Address>,  // for the accounts endpoint
    #[serde(flatten)]
    pub pagination: QueryPagination,
    #[serde(flatten)]
    pub options: QueryOptions,
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

impl QueryParamsBuilder for InputsQuery {
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
        query_builder.push("SELECT * FROM inputs");

        if let Some(tx_id) = &self.tx_id {
            conditions.push(format!("tx_id = '{}'", tx_id));
        }

        if let Some(tx_index) = &self.tx_index {
            conditions.push(format!("tx_index = {}", tx_index));
        }

        if let Some(input_index) = &self.input_index {
            conditions.push(format!("input_index = {}", input_index));
        }

        if let Some(input_type) = &self.input_type {
            conditions.push(format!("type = '{}'", input_type));
        }

        if let Some(block_height) = &self.block_height {
            conditions.push(format!("block_height = {}", block_height));
        }

        if let Some(owner_id) = &self.owner_id {
            conditions.push(format!("owner_id = '{}'", owner_id));
        }

        if let Some(asset_id) = &self.asset_id {
            conditions.push(format!("asset_id = '{}'", asset_id));
        }

        if let Some(contract_id) = &self.contract_id {
            conditions.push(format!("contract_id = '{}'", contract_id));
        }

        if let Some(sender_address) = &self.sender_address {
            conditions.push(format!("sender_address = '{}'", sender_address));
        }

        if let Some(recipient_address) = &self.recipient_address {
            conditions
                .push(format!("recipient_address = '{}'", recipient_address));
        }

        if let Some(address) = &self.address {
            conditions.push(format!(
                "(owner_id = '{}' OR sender_address = '{}' OR recipient_address = '{}')",
                address,
                address,
                address
            ));
        }

        Self::apply_conditions(
            &mut query_builder,
            &mut conditions,
            &self.options,
            &self.pagination,
            "cursor",
            None,
        );

        Self::apply_pagination(&mut query_builder, &self.pagination, &[
            "block_height",
            "tx_index",
            "input_index",
        ]);

        query_builder
    }
}

impl HasPagination for InputsQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}
