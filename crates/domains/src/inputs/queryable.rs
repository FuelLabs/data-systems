use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use sea_query::{
    Asterisk,
    Condition,
    Expr,
    Iden,
    Order,
    PostgresQueryBuilder,
    Query,
    SelectStatement,
};
use serde::{Deserialize, Serialize};

use super::{types::*, InputDbItem};
use crate::queryable::Queryable;

#[allow(dead_code)]
#[derive(Iden)]
enum Inputs {
    #[iden = "inputs"]
    Table,
    #[iden = "subject"]
    Subject,
    #[iden = "block_height"]
    BlockHeight,
    #[iden = "tx_id"]
    TxId,
    #[iden = "tx_index"]
    TxIndex,
    #[iden = "input_index"]
    InputIndex,
    #[iden = "input_type"]
    InputType,
    #[iden = "owner_id"]
    InputOwnerId,
    #[iden = "asset_id"]
    InputAssetId,
    #[iden = "contract_id"]
    InputContractId,
    #[iden = "sender_address"]
    InputSenderAddress,
    #[iden = "recipient_address"]
    InputRecipientAddress,
    #[iden = "value"]
    Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InputsQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<i32>,
    pub input_type: Option<InputType>,
    pub block_height: Option<BlockHeight>,
    pub owner_id: Option<String>,  // for coin inputs
    pub asset_id: Option<AssetId>, // for coin inputs
    pub contract_id: Option<ContractId>, // for contract inputs
    pub sender_address: Option<Address>, // for message inputs
    pub recipient_address: Option<Address>, // for message inputs
    pub after: Option<i32>,
    pub before: Option<i32>,
    pub first: Option<i32>,
    pub last: Option<i32>,
}

impl InputsQuery {
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

    pub fn get_sql_and_values(&self) -> (String, sea_query::Values) {
        self.build_query().build(PostgresQueryBuilder)
    }

    fn build_condition(&self) -> Condition {
        let mut condition = Condition::all();

        if let Some(block_height) = &self.block_height {
            condition = condition.add(
                Expr::col(Inputs::BlockHeight).eq(block_height.to_string()),
            );
        }

        if let Some(tx_id) = &self.tx_id {
            condition =
                condition.add(Expr::col(Inputs::TxId).eq(tx_id.to_string()));
        }

        if let Some(tx_index) = &self.tx_index {
            condition = condition.add(Expr::col(Inputs::TxIndex).eq(*tx_index));
        }

        if let Some(input_index) = &self.input_index {
            condition =
                condition.add(Expr::col(Inputs::InputIndex).eq(*input_index));
        }

        if let Some(input_type) = &self.input_type {
            condition = condition
                .add(Expr::col(Inputs::InputType).eq(input_type.to_string()));
        }

        // unique conditions
        if let Some(owner_id) = &self.owner_id {
            condition = condition
                .add(Expr::col(Inputs::InputOwnerId).eq(owner_id.clone()));
        }

        if let Some(asset_id) = &self.asset_id {
            condition = condition
                .add(Expr::col(Inputs::InputAssetId).eq(asset_id.to_string()));
        }

        if let Some(contract_id) = &self.contract_id {
            condition = condition.add(
                Expr::col(Inputs::InputContractId).eq(contract_id.to_string()),
            );
        }

        if let Some(sender_address) = &self.sender_address {
            condition = condition.add(
                Expr::col(Inputs::InputSenderAddress)
                    .eq(sender_address.to_string()),
            );
        }

        if let Some(recipient_address) = &self.recipient_address {
            condition = condition.add(
                Expr::col(Inputs::InputRecipientAddress)
                    .eq(recipient_address.to_string()),
            );
        }

        condition
    }

    pub fn build_query(&self) -> SelectStatement {
        let mut condition = self.build_condition();

        // Add after/before conditions
        if let Some(after) = self.after {
            condition = condition.add(Expr::col(Inputs::BlockHeight).gt(after));
        }

        if let Some(before) = self.before {
            condition =
                condition.add(Expr::col(Inputs::BlockHeight).lt(before));
        }

        let mut query_builder = Query::select();
        let mut query = query_builder
            .column(Asterisk)
            .from(Inputs::Table)
            .cond_where(condition);

        // Add first/last conditions
        if let Some(first) = self.first {
            query = query
                .order_by(Inputs::BlockHeight, Order::Asc)
                .limit(first as u64);
        } else if let Some(last) = self.last {
            query = query
                .order_by(Inputs::BlockHeight, Order::Desc)
                .limit(last as u64);
        }

        query.to_owned()
    }
}

#[async_trait::async_trait]
impl Queryable for InputsQuery {
    type Record = InputDbItem;

    fn query_to_string(&self) -> String {
        self.build_query().to_string(PostgresQueryBuilder)
    }

    async fn execute<'c, E>(
        &self,
        executor: E,
    ) -> Result<Vec<InputDbItem>, sqlx::Error>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        let sql = self.build_query().to_string(PostgresQueryBuilder);

        sqlx::query_as::<_, InputDbItem>(&sql)
            .fetch_all(executor)
            .await
    }
}
