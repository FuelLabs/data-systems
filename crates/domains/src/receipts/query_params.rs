use fuel_streams_types::{
    Address,
    AssetId,
    BlockHeight,
    Bytes32,
    ContractId,
    TxId,
};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use super::ReceiptType;
use crate::infra::{
    repository::{HasPagination, QueryPagination, QueryParamsBuilder},
    QueryOptions,
};

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptsQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<i32>,
    pub receipt_type: Option<ReceiptType>,
    pub block_height: Option<BlockHeight>,
    pub from: Option<ContractId>,
    pub to: Option<ContractId>,
    pub contract: Option<ContractId>,
    pub asset: Option<AssetId>,
    pub sender: Option<Address>,
    pub recipient: Option<Address>,
    pub sub_id: Option<Bytes32>,
    pub address: Option<Address>, // for the accounts endpoint
    #[serde(flatten)]
    pub pagination: QueryPagination,
    #[serde(flatten)]
    pub options: QueryOptions,
}

impl ReceiptsQuery {
    pub fn set_address(&mut self, address: &str) {
        self.address = Some(Address::from(address));
    }

    pub fn set_receipt_type(&mut self, receipt_type: Option<ReceiptType>) {
        self.receipt_type = receipt_type;
    }

    pub fn set_block_height(&mut self, height: u64) {
        self.block_height = Some(height.into());
    }

    pub fn set_tx_id(&mut self, tx_id: &str) {
        self.tx_id = Some(tx_id.into());
    }
}

impl QueryParamsBuilder for ReceiptsQuery {
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
        query_builder.push("SELECT * FROM receipts");

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

        if let Some(receipt_index) = &self.receipt_index {
            conditions.push("receipt_index = ");
            query_builder.push_bind(*receipt_index);
            query_builder.push(" ");
        }

        if let Some(receipt_type) = &self.receipt_type {
            conditions.push("receipt_type = ");
            query_builder.push_bind(receipt_type.to_string()); // Use string representation
            query_builder.push(" ");
        }

        if let Some(block_height) = &self.block_height {
            conditions.push("block_height = ");
            query_builder.push_bind(*block_height);
            query_builder.push(" ");
        }

        if let Some(from) = &self.from {
            conditions.push("from_contract_id = ");
            query_builder.push_bind(from.to_string());
            query_builder.push(" ");
        }

        if let Some(to) = &self.to {
            conditions.push("to_contract_id = ");
            query_builder.push_bind(to.to_string());
            query_builder.push(" ");
        }

        if let Some(contract) = &self.contract {
            conditions.push("contract_id = ");
            query_builder.push_bind(contract.to_string());
            query_builder.push(" ");
        }

        if let Some(asset) = &self.asset {
            conditions.push("asset_id = ");
            query_builder.push_bind(asset.to_string());
            query_builder.push(" ");
        }

        if let Some(sender) = &self.sender {
            conditions.push("sender_address = ");
            query_builder.push_bind(sender.to_string());
            query_builder.push(" ");
        }

        if let Some(recipient) = &self.recipient {
            conditions.push("recipient_address = ");
            query_builder.push_bind(recipient.to_string());
            query_builder.push(" ");
        }

        if let Some(sub_id) = &self.sub_id {
            conditions.push("sub_id = ");
            query_builder.push_bind(sub_id.to_string());
            query_builder.push(" ");
        }

        if let Some(address) = &self.address {
            conditions.push("(sender_address = ");
            query_builder.push_bind(address.to_string());
            query_builder.push(" OR recipient_address = ");
            query_builder.push_bind(address.to_string());
            query_builder.push(") ");
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

        self.pagination
            .apply_on_query(&mut query_builder, "block_height");

        query_builder
    }
}

impl HasPagination for ReceiptsQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}
