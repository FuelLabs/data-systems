use fuel_streams_types::*;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use super::MessageType;
use crate::infra::{
    repository::{HasPagination, QueryPagination, QueryParamsBuilder},
    QueryOptions,
};

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
pub struct MessagesQuery {
    pub block_height: Option<BlockHeight>,
    pub message_index: Option<i32>,
    pub message_type: Option<MessageType>,
    pub sender: Option<Address>,
    pub recipient: Option<Address>,
    pub nonce: Option<Nonce>,
    pub da_height: Option<DaBlockHeight>,
    pub address: Option<Address>, // for the accounts endpoint
    #[serde(flatten)]
    pub pagination: QueryPagination,
    #[serde(flatten)]
    pub options: QueryOptions,
}

impl MessagesQuery {
    pub fn set_address(&mut self, address: &str) {
        self.address = Some(Address::from(address));
    }

    pub fn set_block_height(&mut self, height: u64) {
        self.block_height = Some(height.into());
    }

    pub fn set_message_type(&mut self, message_type: Option<MessageType>) {
        self.message_type = message_type;
    }
}

impl QueryParamsBuilder for MessagesQuery {
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
        query_builder.push("SELECT * FROM messages");

        if let Some(block_height) = &self.block_height {
            conditions.push(format!("block_height = {}", block_height));
        }

        if let Some(message_index) = &self.message_index {
            conditions.push(format!("message_index = {}", message_index));
        }

        if let Some(message_type) = &self.message_type {
            conditions.push(format!("type = '{}'", message_type));
        }

        if let Some(sender) = &self.sender {
            conditions.push(format!("sender = '{}'", sender));
        }

        if let Some(recipient) = &self.recipient {
            conditions.push(format!("recipient = '{}'", recipient));
        }

        if let Some(nonce) = &self.nonce {
            conditions.push(format!("nonce = '{}'", nonce));
        }

        if let Some(da_height) = &self.da_height {
            conditions.push(format!("da_height = {}", da_height));
        }

        if let Some(address) = &self.address {
            let addr_str = address.to_string();
            conditions.push(format!(
                "(sender = '{}' OR recipient = '{}')",
                addr_str, addr_str
            ));
        }

        let cursor_fields = &["block_height", "message_index"];

        Self::apply_conditions(
            &mut query_builder,
            &mut conditions,
            &self.options,
            &self.pagination,
            cursor_fields,
            None,
        );

        Self::apply_pagination(
            &mut query_builder,
            &self.pagination,
            cursor_fields,
            None,
        );

        query_builder
    }
}

impl HasPagination for MessagesQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}
