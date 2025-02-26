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

use super::{ReceiptDbItem, ReceiptType};
use crate::queryable::Queryable;

#[allow(dead_code)]
#[derive(Iden)]
enum Receipts {
    #[iden = "receipts"]
    Table,
    #[iden = "subject"]
    Subject,
    #[iden = "block_height"]
    BlockHeight,
    #[iden = "tx_id"]
    TxId,
    #[iden = "tx_index"]
    TxIndex,
    #[iden = "receipt_index"]
    ReceiptIndex,
    #[iden = "receipt_type"]
    ReceiptType,
    #[iden = "from_contract_id"]
    FromContractId,
    #[iden = "to_contract_id"]
    ToContractId,
    #[iden = "to_address"]
    ToAddress,
    #[iden = "asset_id"]
    ReceiptAssetId,
    #[iden = "contract_id"]
    ReceiptContractId,
    #[iden = "sub_id"]
    ReceiptSubId,
    #[iden = "sender_address"]
    ReceiptSenderAddress,
    #[iden = "recipient_address"]
    ReceiptRecipientAddress,
    #[iden = "value"]
    Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptsQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<i32>,
    pub receipt_type: Option<ReceiptType>,
    pub block_height: Option<BlockHeight>,
    pub after: Option<i32>,
    pub before: Option<i32>,
    pub first: Option<i32>,
    pub last: Option<i32>,
}

impl ReceiptsQuery {
    pub fn set_block_height(&mut self, height: u64) {
        self.block_height = Some(height.into());
    }

    pub fn set_tx_id(&mut self, tx_id: &str) {
        self.tx_id = Some(tx_id.into());
    }

    pub fn get_sql_and_values(&self) -> (String, sea_query::Values) {
        self.build_query().build(PostgresQueryBuilder)
    }

    fn build_condition(&self) -> Condition {
        let mut condition = Condition::all();

        if let Some(block_height) = &self.block_height {
            condition = condition.add(
                Expr::col(Receipts::BlockHeight).eq(block_height.to_string()),
            );
        }

        if let Some(tx_id) = &self.tx_id {
            condition =
                condition.add(Expr::col(Receipts::TxId).eq(tx_id.to_string()));
        }

        if let Some(tx_index) = &self.tx_index {
            condition =
                condition.add(Expr::col(Receipts::TxIndex).eq(*tx_index));
        }

        if let Some(receipt_index) = &self.receipt_index {
            condition = condition
                .add(Expr::col(Receipts::ReceiptIndex).eq(*receipt_index));
        }

        if let Some(receipt_type) = &self.receipt_type {
            condition = condition.add(
                Expr::col(Receipts::ReceiptType).eq(receipt_type.to_string()),
            );
        }

        condition
    }

    pub fn build_query(&self) -> SelectStatement {
        let mut condition = self.build_condition();

        // Add after/before conditions
        if let Some(after) = self.after {
            condition =
                condition.add(Expr::col(Receipts::BlockHeight).gt(after));
        }

        if let Some(before) = self.before {
            condition =
                condition.add(Expr::col(Receipts::BlockHeight).lt(before));
        }

        let mut query_builder = Query::select();
        let mut query = query_builder
            .column(Asterisk)
            .from(Receipts::Table)
            .cond_where(condition);

        // Add first/last conditions
        if let Some(first) = self.first {
            query = query
                .order_by(Receipts::BlockHeight, Order::Asc)
                .limit(first as u64);
        } else if let Some(last) = self.last {
            query = query
                .order_by(Receipts::BlockHeight, Order::Desc)
                .limit(last as u64);
        }

        query.to_owned()
    }
}

#[async_trait::async_trait]
impl Queryable for ReceiptsQuery {
    type Record = ReceiptDbItem;

    fn query_to_string(&self) -> String {
        self.build_query().to_string(PostgresQueryBuilder)
    }

    async fn execute<'c, E>(
        &self,
        executor: E,
    ) -> Result<Vec<ReceiptDbItem>, sqlx::Error>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        let sql = self.build_query().to_string(PostgresQueryBuilder);

        sqlx::query_as::<_, ReceiptDbItem>(&sql)
            .fetch_all(executor)
            .await
    }
}
