use fuel_streams_types::{Address, BlobId, BlockHeight, ContractId, TxId};
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
pub struct TransactionsQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<i32>,
    pub status: Option<TransactionStatus>,
    pub r#type: Option<TransactionType>,
    pub block_height: Option<BlockHeight>,
    pub blob_id: Option<BlobId>,
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
        if self.address.is_some() || self.contract_id.is_some() {
            self.query_with_joins()
        } else {
            self.query_without_joins()
        }
    }
}

impl TransactionsQuery {
    fn query_with_joins(&self) -> QueryBuilder<'static, Postgres> {
        let mut conditions = Vec::new();
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();

        query_builder.push(
            "SELECT DISTINCT t.* FROM transactions t
            LEFT JOIN inputs i ON t.tx_id = i.tx_id
            LEFT JOIN outputs o ON t.tx_id = o.tx_id",
        );

        if let Some(tx_id) = &self.tx_id {
            conditions.push(format!("t.tx_id = '{}'", tx_id));
        }
        if let Some(tx_index) = &self.tx_index {
            conditions.push(format!("t.tx_index = {}", tx_index));
        }
        if let Some(status) = &self.status {
            conditions.push(format!("t.status = '{}'", status));
        }
        if let Some(tx_type) = &self.r#type {
            conditions.push(format!("t.type = '{}'", tx_type));
        }
        if let Some(block_height) = &self.block_height {
            conditions.push(format!("t.block_height = {}", block_height));
        }
        if let Some(blob_id) = &self.blob_id {
            conditions.push(format!("t.blob_id = '{}'", blob_id));
        }
        if let Some(address) = &self.address {
            conditions.push(format!(
                "(i.sender_address = '{0}' OR i.recipient_address = '{0}' OR i.owner_id = '{0}' OR o.to_address = '{0}')",
                address
            ));
        }
        if let Some(contract_id) = &self.contract_id {
            conditions.push(format!(
                "(i.contract_id = '{0}' OR o.contract_id = '{0}')",
                contract_id
            ));
        }

        Self::apply_conditions(
            &mut query_builder,
            &mut conditions,
            &self.options,
            &self.pagination,
            "cursor",
            Some("t."),
        );

        Self::apply_pagination(
            &mut query_builder,
            &self.pagination,
            "cursor",
            Some("t."),
        );

        query_builder
    }

    fn query_without_joins(&self) -> QueryBuilder<'static, Postgres> {
        let mut conditions = Vec::new();
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();

        query_builder.push("SELECT * FROM transactions");

        if let Some(tx_id) = &self.tx_id {
            conditions.push(format!("tx_id = '{}'", tx_id));
        }
        if let Some(tx_index) = &self.tx_index {
            conditions.push(format!("tx_index = {}", tx_index));
        }
        if let Some(status) = &self.status {
            conditions.push(format!("status = '{}'", status));
        }
        if let Some(tx_type) = &self.r#type {
            conditions.push(format!("type = '{}'", tx_type));
        }
        if let Some(block_height) = &self.block_height {
            conditions.push(format!("block_height = {}", block_height));
        }
        if let Some(blob_id) = &self.blob_id {
            conditions.push(format!("blob_id = '{}'", blob_id));
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

impl HasPagination for TransactionsQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}
