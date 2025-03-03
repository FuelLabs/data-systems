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

use super::TransactionDbItem;
use crate::queryable::Queryable;

#[allow(dead_code)]
#[derive(Iden)]
enum Transactions {
    #[iden = "transactions"]
    Table,
    #[iden = "subject"]
    Subject,
    #[iden = "block_height"]
    BlockHeight,
    #[iden = "tx_id"]
    TxId,
    #[iden = "tx_index"]
    TxIndex,
    #[iden = "tx_status"]
    TxStatus,
    #[iden = "type"]
    Type,
    #[iden = "value"]
    Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionsQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub tx_status: Option<TransactionStatus>,
    #[serde(rename = "type")]
    pub tx_type: Option<TransactionType>,
    pub block_height: Option<BlockHeight>,
    pub after: Option<i32>,
    pub before: Option<i32>,
    pub first: Option<i32>,
    pub last: Option<i32>,
    // TODO: double-check
    pub contract_id: Option<ContractId>,
}

impl TransactionsQuery {
    pub fn set_contract_id(&mut self, contract_id: &str) {
        self.contract_id = Some(ContractId::from(contract_id));
    }

    pub fn set_block_height(&mut self, height: u64) {
        self.block_height = Some(height.into());
    }

    pub fn get_sql_and_values(&self) -> (String, sea_query::Values) {
        self.build_query().build(PostgresQueryBuilder)
    }

    fn build_condition(&self) -> Condition {
        let mut condition = Condition::all();

        if let Some(block_height) = &self.block_height {
            condition = condition.add(
                Expr::col(Transactions::BlockHeight)
                    .eq(block_height.to_string()),
            );
        }

        if let Some(tx_id) = &self.tx_id {
            condition = condition
                .add(Expr::col(Transactions::TxId).eq(tx_id.to_string()));
        }

        if let Some(tx_index) = &self.tx_index {
            condition =
                condition.add(Expr::col(Transactions::TxIndex).eq(*tx_index));
        }

        if let Some(tx_kind) = &self.tx_type {
            condition = condition
                .add(Expr::col(Transactions::Type).eq(tx_kind.to_string()));
        }

        if let Some(tx_status) = &self.tx_status {
            condition = condition.add(
                Expr::col(Transactions::TxStatus).eq(tx_status.to_string()),
            );
        }

        condition
    }

    pub fn build_query(&self) -> SelectStatement {
        let mut condition = self.build_condition();

        // Add after/before conditions
        if let Some(after) = self.after {
            condition =
                condition.add(Expr::col(Transactions::BlockHeight).gt(after));
        }

        if let Some(before) = self.before {
            condition =
                condition.add(Expr::col(Transactions::BlockHeight).lt(before));
        }

        let mut query_builder = Query::select();
        let mut query = query_builder
            .column(Asterisk)
            .from(Transactions::Table)
            .cond_where(condition);

        // Add first/last conditions
        if let Some(first) = self.first {
            query = query
                .order_by(Transactions::BlockHeight, Order::Asc)
                .limit(first as u64);
        } else if let Some(last) = self.last {
            query = query
                .order_by(Transactions::BlockHeight, Order::Desc)
                .limit(last as u64);
        }

        query.to_owned()
    }
}

#[async_trait::async_trait]
impl Queryable for TransactionsQuery {
    type Record = TransactionDbItem;

    fn query_to_string(&self) -> String {
        self.build_query().to_string(PostgresQueryBuilder)
    }

    async fn execute<'c, E>(
        &self,
        executor: E,
    ) -> Result<Vec<TransactionDbItem>, sqlx::Error>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        let sql = self.build_query().to_string(PostgresQueryBuilder);

        sqlx::query_as::<_, TransactionDbItem>(&sql)
            .fetch_all(executor)
            .await
    }
}
