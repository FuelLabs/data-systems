use fuel_streams_types::{Address, BlockHeight, ContractId, TxId};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use super::{TransactionStatus, TransactionType};
use crate::infra::{
    repository::{HasPagination, QueryPagination, QueryParamsBuilder},
    QueryOptions,
};

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct TransactionsQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub tx_status: Option<TransactionStatus>,
    #[serde(rename = "type")]
    pub tx_type: Option<TransactionType>,
    pub block_height: Option<BlockHeight>,
    pub blob_id: Option<String>,
    pub contract_id: Option<ContractId>, // for the contracts endpoint
    pub address: Option<Address>,        // for the accounts endpoint
    #[serde(flatten)]
    pub pagination: QueryPagination,
    #[serde(flatten)]
    pub options: QueryOptions,
}

impl TransactionsQuery {
    pub fn set_address(&mut self, address: &str) {
        self.address = Some(Address::from(address));
    }

    pub fn set_contract_id(&mut self, contract_id: &str) {
        self.contract_id = Some(ContractId::from(contract_id));
    }

    pub fn set_block_height(&mut self, height: u64) {
        self.block_height = Some(height.into());
    }
}

impl QueryParamsBuilder for TransactionsQuery {
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
        query_builder.push("SELECT * FROM transactions");

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

        if let Some(tx_status) = &self.tx_status {
            conditions.push("tx_status = ");
            query_builder.push_bind(tx_status.to_string()); // Use string representation
            query_builder.push(" ");
        }

        if let Some(tx_type) = &self.tx_type {
            conditions.push("type = "); // Match column name from schema
            query_builder.push_bind(tx_type.to_string()); // Use string representation
            query_builder.push(" ");
        }

        if let Some(block_height) = &self.block_height {
            conditions.push("block_height = ");
            query_builder.push_bind(*block_height);
            query_builder.push(" ");
        }

        if let Some(blob_id) = &self.blob_id {
            conditions.push("blob_id = ");
            query_builder.push_bind(blob_id.clone());
            query_builder.push(" ");
        }

        if let Some(contract_id) = &self.contract_id {
            conditions.push("contract_id = ");
            query_builder.push_bind(contract_id.to_string());
            query_builder.push(" ");
        }

        if let Some(address) = &self.address {
            conditions.push("address = ");
            query_builder.push_bind(address.to_string());
            query_builder.push(" ");
        }

        let options = &self.options;
        if let Some(from_block) = options.from_block {
            conditions.push("block_height >= ");
            query_builder.push_bind(from_block);
            query_builder.push(" ");
        }
        #[cfg(any(test, feature = "test-helpers"))]
        if let Some(ns) = &options.namespace {
            conditions.push("subject LIKE ");
            query_builder.push_bind(format!("{}%", ns));
            query_builder.push(" ");
        }

        if !conditions.is_empty() {
            query_builder.push(" WHERE ");
            query_builder.push(conditions.join(" AND "));
        }

        // Apply pagination using block_height as cursor
        self.pagination
            .apply_on_query(&mut query_builder, "block_height");

        query_builder
    }
}

impl HasPagination for TransactionsQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}
