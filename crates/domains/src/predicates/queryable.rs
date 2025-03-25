use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use sea_query::{Condition, Expr, Iden, JoinType, Query, SelectStatement};
use serde::{Deserialize, Serialize};

use super::PredicateDbItem;
use crate::queryable::{HasPagination, QueryPagination, Queryable};

#[allow(dead_code)]
#[derive(Iden)]
pub enum Predicates {
    #[iden = "predicates"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "blob_id"]
    BlobId,
    #[iden = "predicate_address"]
    PredicateAddress,
    #[iden = "value"]
    Value,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "published_at"]
    PublishedAt,
}

#[allow(dead_code)]
#[derive(Iden)]
pub enum PredicateTransactions {
    #[iden = "predicate_transactions"]
    Table,
    #[iden = "predicate_id"]
    PredicateId,
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
}

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct PredicatesQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<i32>,
    pub block_height: Option<BlockHeight>,
    pub blob_id: Option<String>,
    pub predicate_address: Option<Address>,
    #[serde(flatten)]
    pub pagination: QueryPagination,
}

impl PredicatesQuery {
    pub fn set_block_height(&mut self, height: u64) {
        self.block_height = Some(height.into());
    }

    pub fn set_tx_id(&mut self, tx_id: &str) {
        self.tx_id = Some(tx_id.into());
    }

    pub fn set_predicate_address(&mut self, address: &str) {
        self.predicate_address = Some(Address::from(address));
    }

    pub fn set_blob_id(&mut self, blob_id: String) {
        self.blob_id = Some(blob_id);
    }

    pub fn set_input_index(&mut self, index: i32) {
        self.input_index = Some(index);
    }
}

#[async_trait::async_trait]
impl Queryable for PredicatesQuery {
    type Record = PredicateDbItem;
    type Table = Predicates;
    type PaginationColumn = PredicateTransactions;

    fn table() -> Self::Table {
        Predicates::Table
    }

    fn pagination_column() -> Self::PaginationColumn {
        PredicateTransactions::BlockHeight
    }

    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }

    fn build_query(&self) -> SelectStatement {
        let mut query = Query::select();

        // Select all fields needed for PredicateDbItem
        query
            .column((
                PredicateTransactions::Table,
                PredicateTransactions::Subject,
            ))
            .column((
                PredicateTransactions::Table,
                PredicateTransactions::BlockHeight,
            ))
            .column((PredicateTransactions::Table, PredicateTransactions::TxId))
            .column((
                PredicateTransactions::Table,
                PredicateTransactions::TxIndex,
            ))
            .column((
                PredicateTransactions::Table,
                PredicateTransactions::InputIndex,
            ))
            .column((Predicates::Table, Predicates::BlobId))
            .column((Predicates::Table, Predicates::PredicateAddress))
            .column((Predicates::Table, Predicates::Value))
            .column((Predicates::Table, Predicates::CreatedAt))
            .column((Predicates::Table, Predicates::PublishedAt))
            .from(Predicates::Table)
            .join(
                JoinType::InnerJoin,
                PredicateTransactions::Table,
                Expr::col((
                    PredicateTransactions::Table,
                    PredicateTransactions::PredicateId,
                ))
                .equals((Predicates::Table, Predicates::Id)),
            );

        // Apply conditions
        let condition = self.build_condition();
        if !condition.is_empty() {
            query.cond_where(condition);
        }

        // Apply pagination
        if let Some(after) = self.pagination.after() {
            query.and_where(
                Expr::col((
                    PredicateTransactions::Table,
                    PredicateTransactions::BlockHeight,
                ))
                .gt(after),
            );
        }
        if let Some(before) = self.pagination.before() {
            query.and_where(
                Expr::col((
                    PredicateTransactions::Table,
                    PredicateTransactions::BlockHeight,
                ))
                .lt(before),
            );
        }
        if let Some(first) = self.pagination.first() {
            query
                .order_by(
                    (
                        PredicateTransactions::Table,
                        PredicateTransactions::BlockHeight,
                    ),
                    sea_query::Order::Asc,
                )
                .limit(first as u64);
        }
        if let Some(last) = self.pagination.last() {
            query
                .order_by(
                    (
                        PredicateTransactions::Table,
                        PredicateTransactions::BlockHeight,
                    ),
                    sea_query::Order::Desc,
                )
                .limit(last as u64);
        }

        query
    }

    fn build_condition(&self) -> Condition {
        let mut condition = Condition::all();

        if let Some(block_height) = &self.block_height {
            condition = condition.add(
                Expr::col((
                    PredicateTransactions::Table,
                    PredicateTransactions::BlockHeight,
                ))
                .eq(**block_height),
            );
        }

        if let Some(tx_id) = &self.tx_id {
            condition = condition.add(
                Expr::col((
                    PredicateTransactions::Table,
                    PredicateTransactions::TxId,
                ))
                .eq(tx_id.to_string()),
            );
        }

        if let Some(tx_index) = &self.tx_index {
            condition = condition.add(
                Expr::col((
                    PredicateTransactions::Table,
                    PredicateTransactions::TxIndex,
                ))
                .eq(*tx_index),
            );
        }

        if let Some(input_index) = &self.input_index {
            condition = condition.add(
                Expr::col((
                    PredicateTransactions::Table,
                    PredicateTransactions::InputIndex,
                ))
                .eq(*input_index),
            );
        }

        if let Some(blob_id) = &self.blob_id {
            condition = condition.add(
                Expr::col((Predicates::Table, Predicates::BlobId)).eq(blob_id),
            );
        }

        if let Some(predicate_address) = &self.predicate_address {
            condition = condition.add(
                Expr::col((Predicates::Table, Predicates::PredicateAddress))
                    .eq(predicate_address.to_string()),
            );
        }

        condition
    }
}

impl HasPagination for PredicatesQuery {
    fn pagination(&self) -> &QueryPagination {
        &self.pagination
    }
}

#[cfg(test)]
mod test {
    use fuel_streams_types::{Address, BlockHeight, TxId};
    use pretty_assertions::assert_eq;
    use sea_query::PostgresQueryBuilder;

    use super::{PredicatesQuery, Queryable};

    // Test constants
    const AFTER_POINTER: i32 = 10000;
    const BEFORE_POINTER: i32 = 20000;
    const FIRST_POINTER: i32 = 100;
    const LAST_POINTER: i32 = 100;
    const TEST_BLOCK_HEIGHT: i32 = 55;
    const TEST_TX_INDEX: u32 = 3;
    const TEST_INPUT_INDEX: i32 = 7;
    const TEST_TX_ID: &str =
        "0x0101010101010101010101010101010101010101010101010101010101010101";
    const TEST_BLOB_ID: &str =
        "0x0202020202020202020202020202020202020202020202020202020202020202";
    const TEST_ADDRESS: &str =
        "0x0303030303030303030303030303030303030303030303030303030303030303";

    #[test]
    fn test_sql_with_fixed_conditions() {
        // Test 1: Basic query with tx_id and block_height
        let query = PredicatesQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: Some(BlockHeight::from(TEST_BLOCK_HEIGHT)),
            tx_index: None,
            input_index: None,
            blob_id: None,
            predicate_address: None,
            pagination: Default::default(),
        };

        let sql = query.build_query().to_string(PostgresQueryBuilder);
        assert_eq!(
            sql,
            format!(
                "SELECT \
                    \"predicate_transactions\".\"subject\", \
                    \"predicate_transactions\".\"block_height\", \
                    \"predicate_transactions\".\"tx_id\", \
                    \"predicate_transactions\".\"tx_index\", \
                    \"predicate_transactions\".\"input_index\", \
                    \"predicates\".\"blob_id\", \
                    \"predicates\".\"predicate_address\", \
                    \"predicates\".\"value\", \
                    \"predicates\".\"created_at\", \
                    \"predicates\".\"published_at\" \
                FROM \"predicates\" \
                INNER JOIN \"predicate_transactions\" \
                    ON \"predicate_transactions\".\"predicate_id\" = \"predicates\".\"id\" \
                WHERE \"predicate_transactions\".\"block_height\" = {} \
                    AND \"predicate_transactions\".\"tx_id\" = '{}'",
                TEST_BLOCK_HEIGHT, TEST_TX_ID
            )
        );

        // Test 2: Query with blob_id and predicate_address
        let query = PredicatesQuery {
            tx_id: None,
            block_height: None,
            tx_index: None,
            input_index: None,
            blob_id: Some(TEST_BLOB_ID.to_string()),
            predicate_address: Some(Address::from(TEST_ADDRESS)),
            pagination: (None, None, Some(FIRST_POINTER), None).into(),
        };

        let sql = query.build_query().to_string(PostgresQueryBuilder);
        assert_eq!(
            sql,
            format!(
                "SELECT \
                    \"predicate_transactions\".\"subject\", \
                    \"predicate_transactions\".\"block_height\", \
                    \"predicate_transactions\".\"tx_id\", \
                    \"predicate_transactions\".\"tx_index\", \
                    \"predicate_transactions\".\"input_index\", \
                    \"predicates\".\"blob_id\", \
                    \"predicates\".\"predicate_address\", \
                    \"predicates\".\"value\", \
                    \"predicates\".\"created_at\", \
                    \"predicates\".\"published_at\" \
                FROM \"predicates\" \
                INNER JOIN \"predicate_transactions\" \
                    ON \"predicate_transactions\".\"predicate_id\" = \"predicates\".\"id\" \
                WHERE \"predicates\".\"blob_id\" = '{}' \
                    AND \"predicates\".\"predicate_address\" = '{}' \
                ORDER BY \"predicate_transactions\".\"block_height\" ASC \
                LIMIT {}",
                TEST_BLOB_ID, TEST_ADDRESS, FIRST_POINTER
            )
        );

        // Test 3: Query with transaction indices and pagination
        let query = PredicatesQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: None,
            tx_index: Some(TEST_TX_INDEX),
            input_index: Some(TEST_INPUT_INDEX),
            blob_id: None,
            predicate_address: None,
            pagination: (Some(AFTER_POINTER), None, None, Some(LAST_POINTER))
                .into(),
        };

        let sql = query.build_query().to_string(PostgresQueryBuilder);
        assert_eq!(
            sql,
            format!(
                "SELECT \
                    \"predicate_transactions\".\"subject\", \
                    \"predicate_transactions\".\"block_height\", \
                    \"predicate_transactions\".\"tx_id\", \
                    \"predicate_transactions\".\"tx_index\", \
                    \"predicate_transactions\".\"input_index\", \
                    \"predicates\".\"blob_id\", \
                    \"predicates\".\"predicate_address\", \
                    \"predicates\".\"value\", \
                    \"predicates\".\"created_at\", \
                    \"predicates\".\"published_at\" \
                FROM \"predicates\" \
                INNER JOIN \"predicate_transactions\" \
                    ON \"predicate_transactions\".\"predicate_id\" = \"predicates\".\"id\" \
                WHERE \"predicate_transactions\".\"tx_id\" = '{}' \
                    AND \"predicate_transactions\".\"tx_index\" = {} \
                    AND \"predicate_transactions\".\"input_index\" = {} \
                    AND \"predicate_transactions\".\"block_height\" > {} \
                ORDER BY \"predicate_transactions\".\"block_height\" DESC \
                LIMIT {}",
                TEST_TX_ID, TEST_TX_INDEX, TEST_INPUT_INDEX, AFTER_POINTER, LAST_POINTER
            )
        );

        // Test 4: Query with all conditions and before pagination
        let query = PredicatesQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: Some(BlockHeight::from(TEST_BLOCK_HEIGHT)),
            tx_index: Some(TEST_TX_INDEX),
            input_index: Some(TEST_INPUT_INDEX),
            blob_id: Some(TEST_BLOB_ID.to_string()),
            predicate_address: Some(Address::from(TEST_ADDRESS)),
            pagination: (None, Some(BEFORE_POINTER), Some(FIRST_POINTER), None)
                .into(),
        };

        let sql = query.build_query().to_string(PostgresQueryBuilder);
        assert_eq!(
            sql,
            format!(
                "SELECT \
                    \"predicate_transactions\".\"subject\", \
                    \"predicate_transactions\".\"block_height\", \
                    \"predicate_transactions\".\"tx_id\", \
                    \"predicate_transactions\".\"tx_index\", \
                    \"predicate_transactions\".\"input_index\", \
                    \"predicates\".\"blob_id\", \
                    \"predicates\".\"predicate_address\", \
                    \"predicates\".\"value\", \
                    \"predicates\".\"created_at\", \
                    \"predicates\".\"published_at\" \
                FROM \"predicates\" \
                INNER JOIN \"predicate_transactions\" \
                    ON \"predicate_transactions\".\"predicate_id\" = \"predicates\".\"id\" \
                WHERE \"predicate_transactions\".\"block_height\" = {} \
                    AND \"predicate_transactions\".\"tx_id\" = '{}' \
                    AND \"predicate_transactions\".\"tx_index\" = {} \
                    AND \"predicate_transactions\".\"input_index\" = {} \
                    AND \"predicates\".\"blob_id\" = '{}' \
                    AND \"predicates\".\"predicate_address\" = '{}' \
                    AND \"predicate_transactions\".\"block_height\" < {} \
                ORDER BY \"predicate_transactions\".\"block_height\" ASC \
                LIMIT {}",
                TEST_BLOCK_HEIGHT, TEST_TX_ID, TEST_TX_INDEX, TEST_INPUT_INDEX,
                TEST_BLOB_ID, TEST_ADDRESS, BEFORE_POINTER, FIRST_POINTER
            )
        );
    }

    #[test]
    fn test_predicates_query_from_query_string() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&txIndex={}&inputIndex={}&blockHeight={}&blobId={}&predicateAddress={}&after={}&before={}&first={}&last={}",
            TEST_TX_ID, TEST_TX_INDEX, TEST_INPUT_INDEX, TEST_BLOCK_HEIGHT,
            TEST_BLOB_ID, TEST_ADDRESS, AFTER_POINTER, BEFORE_POINTER, FIRST_POINTER, LAST_POINTER
        );

        let query: PredicatesQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, Some(TEST_TX_INDEX));
        assert_eq!(query.input_index, Some(TEST_INPUT_INDEX));
        assert_eq!(
            query.block_height,
            Some(BlockHeight::from(TEST_BLOCK_HEIGHT))
        );
        assert_eq!(query.blob_id, Some(TEST_BLOB_ID.to_string()));
        assert_eq!(query.predicate_address, Some(Address::from(TEST_ADDRESS)));
        assert_eq!(query.pagination.after(), Some(AFTER_POINTER));
        assert_eq!(query.pagination.before(), Some(BEFORE_POINTER));
        assert_eq!(query.pagination.first(), Some(FIRST_POINTER));
        assert_eq!(query.pagination.last(), Some(LAST_POINTER));
    }

    #[test]
    fn test_predicates_query_from_query_string_partial() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&blobId={}&after={}&first={}",
            TEST_TX_ID, TEST_BLOB_ID, AFTER_POINTER, FIRST_POINTER
        );

        let query: PredicatesQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, None);
        assert_eq!(query.input_index, None);
        assert_eq!(query.block_height, None);
        assert_eq!(query.blob_id, Some(TEST_BLOB_ID.to_string()));
        assert_eq!(query.predicate_address, None);
        assert_eq!(query.pagination.after(), Some(AFTER_POINTER));
        assert_eq!(query.pagination.before(), None);
        assert_eq!(query.pagination.first(), Some(FIRST_POINTER));
        assert_eq!(query.pagination.last(), None);
    }
}
