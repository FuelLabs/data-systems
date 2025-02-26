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

use super::{OutputDbItem, OutputType};
use crate::queryable::Queryable;

#[allow(dead_code)]
#[derive(Iden)]
enum Outputs {
    #[iden = "outputs"]
    Table,
    #[iden = "subject"]
    Subject,
    #[iden = "block_height"]
    BlockHeight,
    #[iden = "tx_id"]
    TxId,
    #[iden = "tx_index"]
    TxIndex,
    #[iden = "output_index"]
    OutputIndex,
    #[iden = "output_type"]
    OutputType,
    #[iden = "to_address"]
    OutputToAddress,
    #[iden = "asset_id"]
    OutputAssetId,
    #[iden = "contract_id"]
    OutputContractId,
    #[iden = "value"]
    Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OutputsQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub output_index: Option<i32>,
    pub output_type: Option<OutputType>,
    pub block_height: Option<BlockHeight>,
    pub to_address: Option<String>, // for coin, change, and variable outputs
    pub asset_id: Option<String>,   // for coin, change, and variable outputs
    pub contract_id: Option<String>, /* for contract and contract_created outputs */
    pub after: Option<i32>,
    pub before: Option<i32>,
    pub first: Option<i32>,
    pub last: Option<i32>,
}

impl OutputsQuery {
    pub fn set_block_height(&mut self, height: u64) {
        self.block_height = Some(height.into());
    }

    pub fn set_tx_id(&mut self, tx_id: &str) {
        self.tx_id = Some(tx_id.into());
    }

    pub fn set_output_type(&mut self, output_type: Option<OutputType>) {
        self.output_type = output_type;
    }

    pub fn get_sql_and_values(&self) -> (String, sea_query::Values) {
        self.build_query().build(PostgresQueryBuilder)
    }

    fn build_condition(&self) -> Condition {
        let mut condition = Condition::all();

        if let Some(block_height) = &self.block_height {
            condition = condition.add(
                Expr::col(Outputs::BlockHeight).eq(block_height.to_string()),
            );
        }

        if let Some(tx_id) = &self.tx_id {
            condition =
                condition.add(Expr::col(Outputs::TxId).eq(tx_id.to_string()));
        }

        if let Some(tx_index) = &self.tx_index {
            condition =
                condition.add(Expr::col(Outputs::TxIndex).eq(*tx_index));
        }

        if let Some(output_index) = &self.output_index {
            condition = condition
                .add(Expr::col(Outputs::OutputIndex).eq(*output_index));
        }

        if let Some(output_type) = &self.output_type {
            condition = condition.add(
                Expr::col(Outputs::OutputType).eq(output_type.to_string()),
            );
        }

        // unique conditions
        if let Some(to_address) = &self.to_address {
            condition = condition.add(
                Expr::col(Outputs::OutputToAddress).eq(to_address.clone()),
            );
        }

        if let Some(asset_id) = &self.asset_id {
            condition = condition
                .add(Expr::col(Outputs::OutputAssetId).eq(asset_id.clone()));
        }

        if let Some(contract_id) = &self.contract_id {
            condition = condition.add(
                Expr::col(Outputs::OutputContractId).eq(contract_id.clone()),
            );
        }

        condition
    }

    pub fn build_query(&self) -> SelectStatement {
        let mut condition = self.build_condition();

        // Add after/before conditions
        if let Some(after) = self.after {
            condition =
                condition.add(Expr::col(Outputs::BlockHeight).gt(after));
        }

        if let Some(before) = self.before {
            condition =
                condition.add(Expr::col(Outputs::BlockHeight).lt(before));
        }

        let mut query_builder = Query::select();
        let mut query = query_builder
            .column(Asterisk)
            .from(Outputs::Table)
            .cond_where(condition);

        // Add first/last conditions
        if let Some(first) = self.first {
            query = query
                .order_by(Outputs::BlockHeight, Order::Asc)
                .limit(first as u64);
        } else if let Some(last) = self.last {
            query = query
                .order_by(Outputs::BlockHeight, Order::Desc)
                .limit(last as u64);
        }

        query.to_owned()
    }
}

#[async_trait::async_trait]
impl Queryable for OutputsQuery {
    type Record = OutputDbItem;

    fn query_to_string(&self) -> String {
        self.build_query().to_string(PostgresQueryBuilder)
    }

    async fn execute<'c, E>(
        &self,
        executor: E,
    ) -> Result<Vec<OutputDbItem>, sqlx::Error>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        let sql = self.build_query().to_string(PostgresQueryBuilder);

        sqlx::query_as::<_, OutputDbItem>(&sql)
            .fetch_all(executor)
            .await
    }
}
