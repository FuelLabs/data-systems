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
            condition = condition
                .add(Expr::col(Receipts::BlockHeight).eq(**block_height));
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

#[cfg(test)]
mod test {
    use fuel_streams_types::{BlockHeight, TxId};
    use pretty_assertions::assert_eq;

    use crate::{
        queryable::Queryable,
        receipts::queryable::{ReceiptType, ReceiptsQuery},
    };

    // Test constants
    const AFTER_POINTER: i32 = 10000;
    const BEFORE_POINTER: i32 = 20000;
    const FIRST_POINTER: i32 = 300;
    const LAST_POINTER: i32 = 400;
    const TEST_BLOCK_HEIGHT: i32 = 55;
    const TEST_TX_INDEX: u32 = 3;
    const TEST_RECEIPT_INDEX: i32 = 7;
    const TEST_TX_ID: &str =
        "0x0101010101010101010101010101010101010101010101010101010101010101";

    #[test]
    fn test_sql_with_fixed_conds() {
        // Test 1: basic query with tx_id, block_height and receipt_type
        let query = ReceiptsQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: Some(BlockHeight::from(TEST_BLOCK_HEIGHT)),
            receipt_type: Some(ReceiptType::Call),
            tx_index: None,
            receipt_index: None,
            after: None,
            before: None,
            first: None,
            last: None,
        };

        assert_eq!(
            query.query_to_string(),
            format!("SELECT * FROM \"receipts\" WHERE \"block_height\" = {} AND \"tx_id\" = '{}' AND \"receipt_type\" = 'call'",
                TEST_BLOCK_HEIGHT, TEST_TX_ID)
        );

        // Test 2: query with receipt indices and first pagination
        let indices_query = ReceiptsQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: None,
            receipt_type: None,
            tx_index: Some(TEST_TX_INDEX),
            receipt_index: Some(TEST_RECEIPT_INDEX),
            after: None,
            before: None,
            first: Some(FIRST_POINTER),
            last: None,
        };

        assert_eq!(
            indices_query.query_to_string(),
            format!("SELECT * FROM \"receipts\" WHERE \"tx_id\" = '{}' AND \"tx_index\" = {} AND \"receipt_index\" = {} ORDER BY \"block_height\" ASC LIMIT {}",
                TEST_TX_ID, TEST_TX_INDEX, TEST_RECEIPT_INDEX, FIRST_POINTER)
        );

        // Test 3: query with receipt_type and last pagination with range
        let range_query = ReceiptsQuery {
            tx_id: None,
            block_height: None,
            receipt_type: Some(ReceiptType::Return),
            tx_index: None,
            receipt_index: None,
            after: Some(AFTER_POINTER),
            before: None,
            first: None,
            last: Some(LAST_POINTER),
        };

        assert_eq!(
            range_query.query_to_string(),
            format!("SELECT * FROM \"receipts\" WHERE \"receipt_type\" = 'return' AND \"block_height\" > {} ORDER BY \"block_height\" DESC LIMIT {}",
                AFTER_POINTER, LAST_POINTER)
        );

        // Test 4: query with block height and before/after conditions
        let block_range_query = ReceiptsQuery {
            tx_id: None,
            block_height: Some(BlockHeight::from(TEST_BLOCK_HEIGHT)),
            receipt_type: None,
            tx_index: None,
            receipt_index: None,
            after: Some(AFTER_POINTER),
            before: Some(BEFORE_POINTER),
            first: Some(FIRST_POINTER),
            last: None,
        };

        assert_eq!(
            block_range_query.query_to_string(),
            format!("SELECT * FROM \"receipts\" WHERE \"block_height\" = {} AND \"block_height\" > {} AND \"block_height\" < {} ORDER BY \"block_height\" ASC LIMIT {}",
                TEST_BLOCK_HEIGHT, AFTER_POINTER, BEFORE_POINTER, FIRST_POINTER)
        );
    }

    #[test]
    fn test_receipts_query_from_query_string() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&txIndex={}&receiptIndex={}&receiptType=Call&blockHeight={}&after={}&before={}&first={}&last={}",
            TEST_TX_ID,
            TEST_TX_INDEX,
            TEST_RECEIPT_INDEX,
            TEST_BLOCK_HEIGHT,
            AFTER_POINTER,
            BEFORE_POINTER,
            FIRST_POINTER,
            LAST_POINTER
        );

        let query: ReceiptsQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, Some(TEST_TX_INDEX));
        assert_eq!(query.receipt_index, Some(TEST_RECEIPT_INDEX));
        assert_eq!(query.receipt_type, Some(ReceiptType::Call));
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
    fn test_receipts_query_from_query_string_partial() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&receiptType=Revert&after={}&first={}",
            TEST_TX_ID, AFTER_POINTER, FIRST_POINTER
        );

        let query: ReceiptsQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, None);
        assert_eq!(query.receipt_index, None);
        assert_eq!(query.receipt_type, Some(ReceiptType::Revert));
        assert_eq!(query.block_height, None);
        assert_eq!(query.after, Some(AFTER_POINTER));
        assert_eq!(query.before, None);
        assert_eq!(query.first, Some(FIRST_POINTER));
        assert_eq!(query.last, None);
    }

    #[test]
    fn test_set_methods() {
        let mut query = ReceiptsQuery::default();

        // Test set_block_height
        query.set_block_height(TEST_BLOCK_HEIGHT as u64);
        assert_eq!(
            query.block_height,
            Some(BlockHeight::from(TEST_BLOCK_HEIGHT))
        );

        // Test set_tx_id
        query.set_tx_id(TEST_TX_ID);
        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));

        // Test query building after setting values
        assert_eq!(
            query.query_to_string(),
            format!("SELECT * FROM \"receipts\" WHERE \"block_height\" = {} AND \"tx_id\" = '{}'",
                TEST_BLOCK_HEIGHT, TEST_TX_ID)
        );
    }
}
