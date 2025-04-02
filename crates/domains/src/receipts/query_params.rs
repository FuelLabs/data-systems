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
#[serde(rename_all = "camelCase")]
pub struct ReceiptsQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<i32>,
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
            conditions.push(format!("tx_id = '{}'", tx_id));
        }

        if let Some(tx_index) = &self.tx_index {
            conditions.push(format!("tx_index = {}", tx_index));
        }

        if let Some(receipt_index) = &self.receipt_index {
            conditions.push(format!("receipt_index = {}", receipt_index));
        }

        if let Some(receipt_type) = &self.receipt_type {
            conditions.push(format!("type = '{}'", receipt_type));
        }

        if let Some(block_height) = &self.block_height {
            conditions.push(format!("block_height = {}", block_height));
        }

        if let Some(from) = &self.from {
            conditions.push(format!("from_contract_id = '{}'", from));
        }

        if let Some(to) = &self.to {
            conditions.push(format!("to_contract_id = '{}'", to));
        }

        if let Some(contract) = &self.contract {
            conditions.push(format!("contract_id = '{}'", contract));
        }

        if let Some(asset) = &self.asset {
            conditions.push(format!("asset_id = '{}'", asset));
        }

        if let Some(sender) = &self.sender {
            conditions.push(format!("sender_address = '{}'", sender));
        }

        if let Some(recipient) = &self.recipient {
            conditions.push(format!("recipient_address = '{}'", recipient));
        }

        if let Some(sub_id) = &self.sub_id {
            conditions.push(format!("sub_id = '{}'", sub_id));
        }

        if let Some(address) = &self.address {
            let addr_str = address.to_string();
            conditions.push(format!(
                "(sender_address = '{}' OR recipient_address = '{}')",
                addr_str, addr_str
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

        Self::apply_pagination(
            &mut query_builder,
            &self.pagination,
            "cursor",
            None,
        );

        query_builder
    }
}

impl HasPagination for ReceiptsQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}
