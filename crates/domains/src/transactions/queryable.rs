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
    pub address: Option<Address>,
}

impl TransactionsQuery {
    pub fn set_address(&mut self, address: &str) {
        self.address = Some(Address::from(address));
    }

    pub fn set_block_height(&mut self, height: u64) {
        self.block_height = Some(height.into());
    }

    pub fn get_sql_and_values(&self) -> (String, sea_query::Values) {
        self.build_query().build(PostgresQueryBuilder)
    }

    fn build_condition(&self) -> Condition {
        let mut condition = Condition::all();

        // TODO: handle address query
        if let Some(_address) = &self.address {
            match self.tx_type {
                Some(TransactionType::Blob) => {}
                Some(TransactionType::Create) => {}
                Some(TransactionType::Mint) => {}
                Some(TransactionType::Script) => {}
                Some(TransactionType::Upgrade) => {}
                Some(TransactionType::Upload) => {}
                _ => {}
            }
        }

        if let Some(block_height) = &self.block_height {
            condition = condition
                .add(Expr::col(Transactions::BlockHeight).eq(**block_height));
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

#[cfg(test)]
mod test {
    use fuel_streams_types::{
        BlockHeight,
        TransactionStatus,
        TransactionType,
        TxId,
    };
    use pretty_assertions::assert_eq;

    use crate::{
        queryable::Queryable,
        transactions::queryable::TransactionsQuery,
    };

    // Test constants
    const AFTER_POINTER: i32 = 10000;
    const BEFORE_POINTER: i32 = 20000;
    const FIRST_POINTER: i32 = 300;
    const LAST_POINTER: i32 = 400;
    const TEST_BLOCK_HEIGHT: i32 = 55;
    const TEST_TX_INDEX: u32 = 3;
    const TEST_TX_ID: &str =
        "0x0101010101010101010101010101010101010101010101010101010101010101";

    #[test]
    fn test_sql_with_fixed_conds() {
        // Test 1: basic query with tx_id, block_height and tx_status
        let query = TransactionsQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: Some(BlockHeight::from(TEST_BLOCK_HEIGHT)),
            tx_status: Some(TransactionStatus::Success),
            tx_type: None,
            tx_index: None,
            after: None,
            before: None,
            first: None,
            last: None,
            address: None,
        };

        assert_eq!(
            query.query_to_string(),
            format!("SELECT * FROM \"transactions\" WHERE \"block_height\" = {} AND \"tx_id\" = '{}' AND \"tx_status\" = 'success'",
                TEST_BLOCK_HEIGHT, TEST_TX_ID)
        );

        // Test 2: query with tx_type and tx_index
        let type_query = TransactionsQuery {
            tx_id: None,
            block_height: None,
            tx_status: None,
            tx_type: Some(TransactionType::Script),
            tx_index: Some(TEST_TX_INDEX),
            after: None,
            before: None,
            first: Some(FIRST_POINTER),
            last: None,
            address: None,
        };

        assert_eq!(
            type_query.query_to_string(),
            format!("SELECT * FROM \"transactions\" WHERE \"tx_index\" = {} AND \"type\" = 'script' ORDER BY \"block_height\" ASC LIMIT {}",
                TEST_TX_INDEX, FIRST_POINTER)
        );

        // Test 3: query with block height and range conditions
        let range_query = TransactionsQuery {
            tx_id: None,
            block_height: Some(BlockHeight::from(TEST_BLOCK_HEIGHT)),
            tx_status: None,
            tx_type: None,
            tx_index: None,
            after: Some(AFTER_POINTER),
            before: None,
            first: None,
            last: Some(LAST_POINTER),
            address: None,
        };

        assert_eq!(
            range_query.query_to_string(),
            format!("SELECT * FROM \"transactions\" WHERE \"block_height\" = {} AND \"block_height\" > {} ORDER BY \"block_height\" DESC LIMIT {}",
                TEST_BLOCK_HEIGHT, AFTER_POINTER, LAST_POINTER)
        );

        // Test 4: query with tx_status, tx_type and before condition
        let status_type_query = TransactionsQuery {
            tx_id: None,
            block_height: None,
            tx_status: Some(TransactionStatus::Failed),
            tx_type: Some(TransactionType::Create),
            tx_index: None,
            after: None,
            before: Some(BEFORE_POINTER),
            first: Some(FIRST_POINTER),
            last: None,
            address: None,
        };

        assert_eq!(
            status_type_query.query_to_string(),
            format!("SELECT * FROM \"transactions\" WHERE \"type\" = 'create' AND \"tx_status\" = 'failed' AND \"block_height\" < {} ORDER BY \"block_height\" ASC LIMIT {}",
                BEFORE_POINTER, FIRST_POINTER)
        );

        // Test 5: query with only tx_id
        let tx_id_query = TransactionsQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: None,
            tx_status: None,
            tx_type: None,
            tx_index: None,
            after: None,
            before: None,
            first: None,
            last: None,
            address: None,
        };

        assert_eq!(
            tx_id_query.query_to_string(),
            format!(
                "SELECT * FROM \"transactions\" WHERE \"tx_id\" = '{}'",
                TEST_TX_ID
            )
        );
    }

    #[test]
    fn test_transactions_query_from_query_string() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&txIndex={}&txStatus=success&type=script&blockHeight={}&after={}&before={}&first={}&last={}",
            TEST_TX_ID,
            TEST_TX_INDEX,
            TEST_BLOCK_HEIGHT,
            AFTER_POINTER,
            BEFORE_POINTER,
            FIRST_POINTER,
            LAST_POINTER
        );

        let query: TransactionsQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, Some(TEST_TX_INDEX));
        assert_eq!(query.tx_status, Some(TransactionStatus::Success));
        assert_eq!(query.tx_type, Some(TransactionType::Script));
        assert_eq!(
            query.block_height,
            Some(BlockHeight::from(TEST_BLOCK_HEIGHT))
        );
        assert_eq!(query.after, Some(AFTER_POINTER));
        assert_eq!(query.before, Some(BEFORE_POINTER));
        assert_eq!(query.first, Some(FIRST_POINTER));
        assert_eq!(query.last, Some(LAST_POINTER));
    }

    #[test]
    fn test_transactions_query_from_query_string_partial() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&txStatus=submitted&type=mint&after={}&first={}",
            TEST_TX_ID, AFTER_POINTER, FIRST_POINTER
        );

        let query: TransactionsQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, None);
        assert_eq!(query.tx_status, Some(TransactionStatus::Submitted));
        assert_eq!(query.tx_type, Some(TransactionType::Mint));
        assert_eq!(query.block_height, None);
        assert_eq!(query.after, Some(AFTER_POINTER));
        assert_eq!(query.before, None);
        assert_eq!(query.first, Some(FIRST_POINTER));
        assert_eq!(query.last, None);
    }

    #[test]
    fn test_set_block_height() {
        let mut query = TransactionsQuery::default();

        // Test set_block_height
        query.set_block_height(TEST_BLOCK_HEIGHT as u64);
        assert_eq!(
            query.block_height,
            Some(BlockHeight::from(TEST_BLOCK_HEIGHT))
        );

        // Test query building after setting block height
        assert_eq!(
            query.query_to_string(),
            format!(
                "SELECT * FROM \"transactions\" WHERE \"block_height\" = {}",
                TEST_BLOCK_HEIGHT
            )
        );
    }
}
