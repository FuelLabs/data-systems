use fuel_streams_types::{Address, AssetId, BlockHeight, ContractId, TxId};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use super::types::*;
use crate::infra::repository::{
    HasPagination,
    QueryPagination,
    QueryParamsBuilder,
};

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
    pub address: Option<Address>,  // for the accounts endpoint
    #[serde(flatten)]
    pub pagination: QueryPagination,
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
    fn query_builder(&self) -> QueryBuilder<'static, Postgres> {
        let mut conditions = Vec::new();
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();
        query_builder.push("SELECT * FROM inputs");

        if let Some(tx_id) = &self.tx_id {
            conditions.push("tx_id = ");
            query_builder.push_bind(tx_id.to_string());
            query_builder.push(" ");
        }

        if let Some(tx_index) = &self.tx_index {
            conditions.push("tx_index = ");
            query_builder.push_bind(*tx_index as i32);
            query_builder.push(" ");
        }

        if let Some(input_index) = &self.input_index {
            conditions.push("input_index = ");
            query_builder.push_bind(*input_index);
            query_builder.push(" ");
        }

        if let Some(input_type) = &self.input_type {
            conditions.push("input_type = ");
            query_builder.push_bind(input_type.clone() as i32);
            query_builder.push(" ");
        }

        if let Some(block_height) = &self.block_height {
            conditions.push("block_height = ");
            query_builder.push_bind(*block_height);
            query_builder.push(" ");
        }

        if let Some(owner_id) = &self.owner_id {
            conditions.push("owner_id = ");
            query_builder.push_bind(owner_id.to_string());
            query_builder.push(" ");
        }

        if let Some(asset_id) = &self.asset_id {
            conditions.push("asset_id = ");
            query_builder.push_bind(asset_id.to_string());
            query_builder.push(" ");
        }

        if let Some(contract_id) = &self.contract_id {
            conditions.push("contract_id = ");
            query_builder.push_bind(contract_id.to_string());
            query_builder.push(" ");
        }

        if let Some(sender_address) = &self.sender_address {
            conditions.push("sender_address = ");
            query_builder.push_bind(sender_address.to_string());
            query_builder.push(" ");
        }

        if let Some(recipient_address) = &self.recipient_address {
            conditions.push("recipient_address = ");
            query_builder.push_bind(recipient_address.to_string());
            query_builder.push(" ");
        }

        if let Some(address) = &self.address {
            conditions.push("(owner_id = ");
            query_builder.push_bind(address.to_string());
            query_builder.push(" OR sender_address = ");
            query_builder.push_bind(address.to_string());
            query_builder.push(" OR recipient_address = ");
            query_builder.push_bind(address.to_string());
            query_builder.push(") ");
        }

        if !conditions.is_empty() {
            query_builder.push(" WHERE ");
            query_builder.push(conditions.join(" AND "));
        }

        // Apply pagination using block_height as cursor
        self.pagination
            .apply_pagination(&mut query_builder, "block_height");

        query_builder
    }
}

impl HasPagination for InputsQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}
