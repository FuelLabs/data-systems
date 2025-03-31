use fuel_streams_types::{Address, BlockHeight, ContractId, TxId};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use super::{TransactionStatus, TransactionType};
use crate::infra::repository::{
    HasPagination,
    QueryPagination,
    QueryParamsBuilder,
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
            query_builder.push_bind(tx_status.clone() as i32);
            query_builder.push(" ");
        }

        if let Some(tx_type) = &self.tx_type {
            conditions.push("tx_type = ");
            query_builder.push_bind(tx_type.clone() as i32);
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

impl HasPagination for TransactionsQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}
