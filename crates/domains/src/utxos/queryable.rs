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

use super::UtxoDbItem;
use crate::{inputs::InputType, queryable::Queryable};

#[allow(dead_code)]
#[derive(Iden)]
enum Utxos {
    #[iden = "utxos"]
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
    #[iden = "utxo_type"]
    UtxoType,
    #[iden = "utxo_id"]
    UtxoId,
    #[iden = "value"]
    Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UtxosQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<i32>,
    pub utxo_type: Option<InputType>,
    pub block_height: Option<BlockHeight>,
    pub utxo_id: Option<HexData>,
    pub after: Option<i32>,
    pub before: Option<i32>,
    pub first: Option<i32>,
    pub last: Option<i32>,
    pub address: Option<String>,
}

impl UtxosQuery {
    pub fn set_address(&mut self, address: String) {
        self.address = Some(address);
    }

    pub fn set_block_height(&mut self, height: u64) {
        self.block_height = Some(height.into());
    }

    pub fn set_tx_id(&mut self, tx_id: &str) {
        self.tx_id = Some(tx_id.into());
    }

    pub fn set_utxo_type(&mut self, utxo_type: Option<InputType>) {
        self.utxo_type = utxo_type;
    }

    pub fn get_sql_and_values(&self) -> (String, sea_query::Values) {
        self.build_query().build(PostgresQueryBuilder)
    }

    fn build_condition(&self) -> Condition {
        let mut condition = Condition::all();

        // handle address query
        // TODO: extend db fields with separate sender, recipient as per utxo type Input::Message
        if let Some(address) = &self.address {
            match self.utxo_type {
                Some(InputType::Coin) => {
                    condition = condition
                        .add(Expr::col(Utxos::UtxoId).eq(address.clone()));
                }
                Some(InputType::Contract) => {
                    condition = condition
                        .add(Expr::col(Utxos::UtxoId).eq(address.clone()));
                }
                Some(InputType::Message) => {
                    condition = condition
                        .add(Expr::col(Utxos::UtxoId).eq(address.clone()));
                }
                _ => {
                    condition = condition
                        .add(Expr::col(Utxos::UtxoId).eq(address.clone()));
                }
            }
        }

        if let Some(block_height) = &self.block_height {
            condition = condition.add(
                Expr::col(Utxos::BlockHeight).eq(block_height.to_string()),
            );
        }

        if let Some(tx_id) = &self.tx_id {
            condition =
                condition.add(Expr::col(Utxos::TxId).eq(tx_id.to_string()));
        }

        if let Some(tx_index) = &self.tx_index {
            condition = condition.add(Expr::col(Utxos::TxIndex).eq(*tx_index));
        }

        if let Some(input_index) = &self.input_index {
            condition =
                condition.add(Expr::col(Utxos::InputIndex).eq(*input_index));
        }

        if let Some(utxo_type) = &self.utxo_type {
            condition = condition
                .add(Expr::col(Utxos::UtxoType).eq(utxo_type.to_string()));
        }

        // unique conditions
        if let Some(utxo_id) = &self.utxo_id {
            condition =
                condition.add(Expr::col(Utxos::UtxoId).eq(utxo_id.to_string()));
        }

        condition
    }

    pub fn build_query(&self) -> SelectStatement {
        let mut condition = self.build_condition();

        // Add after/before conditions
        if let Some(after) = self.after {
            condition = condition.add(Expr::col(Utxos::BlockHeight).gt(after));
        }

        if let Some(before) = self.before {
            condition = condition.add(Expr::col(Utxos::BlockHeight).lt(before));
        }

        let mut query_builder = Query::select();
        let mut query = query_builder
            .column(Asterisk)
            .from(Utxos::Table)
            .cond_where(condition);

        // Add first/last conditions
        if let Some(first) = self.first {
            query = query
                .order_by(Utxos::BlockHeight, Order::Asc)
                .limit(first as u64);
        } else if let Some(last) = self.last {
            query = query
                .order_by(Utxos::BlockHeight, Order::Desc)
                .limit(last as u64);
        }

        query.to_owned()
    }
}

#[async_trait::async_trait]
impl Queryable for UtxosQuery {
    type Record = UtxoDbItem;

    fn query_to_string(&self) -> String {
        self.build_query().to_string(PostgresQueryBuilder)
    }

    async fn execute<'c, E>(
        &self,
        executor: E,
    ) -> Result<Vec<UtxoDbItem>, sqlx::Error>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        let sql = self.build_query().to_string(PostgresQueryBuilder);

        sqlx::query_as::<_, UtxoDbItem>(&sql)
            .fetch_all(executor)
            .await
    }
}
