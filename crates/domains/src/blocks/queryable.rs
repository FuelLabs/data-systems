use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

use chrono::{DateTime, Duration, Utc};
use fuel_streams_subject::subject::*;
use fuel_streams_types::{BlockTimestamp, *};
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

use super::{types::*, BlockDbItem};
use crate::queryable::Queryable;

#[derive(
    Debug,
    Default,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    utoipa::ToSchema,
)]
pub enum TimeRange {
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "12h")]
    TwelveHours,
    #[serde(rename = "1d")]
    OneDay,
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "30d")]
    ThirtyDays,
    #[serde(rename = "90d")]
    NinetyDays,
    #[serde(rename = "180d")]
    OneEightyDays,
    #[serde(rename = "1y")]
    OneYear,
    #[default]
    #[serde(rename = "all")]
    All,
}

impl TimeRange {
    pub fn time_since_now(&self) -> DateTime<Utc> {
        let now = Utc::now();
        if let Some(duration) = self.to_duration() {
            now - duration
        } else {
            now // For TimeRange::All
        }
    }

    pub fn to_duration(&self) -> Option<Duration> {
        match self {
            Self::OneHour => Some(Duration::hours(1)),
            Self::TwelveHours => Some(Duration::hours(12)),
            Self::OneDay => Some(Duration::days(1)),
            Self::SevenDays => Some(Duration::days(7)),
            Self::ThirtyDays => Some(Duration::days(30)),
            Self::NinetyDays => Some(Duration::days(90)),
            Self::OneEightyDays => Some(Duration::days(180)),
            Self::OneYear => Some(Duration::days(365)),
            Self::All => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneHour => "1h",
            Self::TwelveHours => "12h",
            Self::OneDay => "1d",
            Self::SevenDays => "7d",
            Self::ThirtyDays => "30d",
            Self::NinetyDays => "90d",
            Self::OneEightyDays => "180d",
            Self::OneYear => "1y",
            Self::All => "all",
        }
    }
}

impl Display for TimeRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for TimeRange {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "1h" => Ok(Self::OneHour),
            "12h" => Ok(Self::TwelveHours),
            "1d" => Ok(Self::OneDay),
            "7d" => Ok(Self::SevenDays),
            "30d" => Ok(Self::ThirtyDays),
            "90d" => Ok(Self::NinetyDays),
            "180d" => Ok(Self::OneEightyDays),
            "1y" => Ok(Self::OneYear),
            "all" => Ok(Self::All),
            _ => Err(format!("Invalid TimeRange: {}", s)),
        }
    }
}

#[allow(dead_code)]
#[derive(Iden)]
enum Blocks {
    #[iden = "blocks"]
    Table,
    #[iden = "producer_address"]
    Producer,
    #[iden = "block_height"]
    Height,
    #[iden = "timestamp"]
    Timestamp,
    #[iden = "value"]
    Value,
}

#[derive(
    Debug,
    Clone,
    Default,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct BlocksQuery {
    pub producer: Option<Address>,
    pub height: Option<BlockHeight>,
    pub timestamp: Option<BlockTimestamp>,
    pub time_range: Option<TimeRange>,
    pub after: Option<i32>,
    pub before: Option<i32>,
    pub first: Option<i32>,
    pub last: Option<i32>,
}

impl From<&Block> for BlocksQuery {
    fn from(block: &Block) -> Self {
        BlocksQuery {
            producer: Some(block.producer.to_owned()),
            height: Some(block.height.to_owned()),
            timestamp: Some(BlockTimestamp::from(&block.header)),
            time_range: Some(TimeRange::default()),
            ..Default::default()
        }
    }
}

impl BlocksQuery {
    pub fn get_sql_and_values(&self) -> (String, sea_query::Values) {
        self.build_query().build(PostgresQueryBuilder)
    }

    fn build_condition(&self) -> Condition {
        let mut condition = Condition::all();

        if let Some(producer) = &self.producer {
            condition = condition
                .add(Expr::col(Blocks::Producer).eq(producer.to_string()));
        }

        if let Some(height) = &self.height {
            condition =
                condition.add(Expr::col(Blocks::Height).eq(*height.deref()));
        }

        if let Some(timestamp) = &self.timestamp {
            condition = condition.add(
                Expr::col(Blocks::Timestamp).gte(timestamp.unix_timestamp()),
            );
        }

        // Add time range condition
        if let Some(time_range) = &self.time_range {
            let start_time = time_range.time_since_now();
            condition = condition
                .add(Expr::col(Blocks::Timestamp).gte(start_time.timestamp()));
        }

        condition
    }

    pub fn build_query(&self) -> SelectStatement {
        let mut condition = self.build_condition();

        // Add after/before conditions
        if let Some(after) = self.after {
            condition = condition.add(Expr::col(Blocks::Height).gt(after));
        }

        if let Some(before) = self.before {
            condition = condition.add(Expr::col(Blocks::Height).lt(before));
        }

        let mut query_builder = Query::select();
        let mut query = query_builder
            .column(Asterisk)
            .from(Blocks::Table)
            .cond_where(condition);

        // Add first/last conditions
        if let Some(first) = self.first {
            query = query
                .order_by(Blocks::Height, Order::Asc)
                .limit(first as u64);
        } else if let Some(last) = self.last {
            query = query
                .order_by(Blocks::Height, Order::Desc)
                .limit(last as u64);
        }

        query.to_owned()
    }
}

#[async_trait::async_trait]
impl Queryable for BlocksQuery {
    type Record = BlockDbItem;

    fn query_to_string(&self) -> String {
        self.build_query().to_string(PostgresQueryBuilder)
    }

    async fn execute<'c, E>(
        &self,
        executor: E,
    ) -> Result<Vec<BlockDbItem>, sqlx::Error>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        let sql = self.build_query().to_string(PostgresQueryBuilder);

        sqlx::query_as::<_, BlockDbItem>(&sql)
            .fetch_all(executor)
            .await
    }
}

#[cfg(test)]
mod test {
    use chrono::Utc;
    use fuel_streams_types::{Address, BlockHeight, BlockTimestamp};
    use pretty_assertions::assert_eq;

    use crate::{
        blocks::queryable::{BlocksQuery, TimeRange},
        queryable::Queryable,
    };

    const AFTER_POINTER: i32 = 10000;
    const BEFORE_POINTER: i32 = 20000;
    const FIRST_POINTER: i32 = 300;
    const LAST_POINTER: i32 = 400;
    const TEST_TIMESTAMP: i64 = 1739974057;
    const TEST_BLOCK_HEIGHT: i32 = 55;
    const TEST_PRODUCER_ADDRESS: &str =
        "0x0101010101010101010101010101010101010101010101010101010101010101";

    #[test]
    fn test_sql_with_fixed_conds() {
        // Test 1: classical basic query
        let query = BlocksQuery {
            producer: Some(Address::from([1u8; 32])),
            height: Some(BlockHeight::from(TEST_BLOCK_HEIGHT)),
            timestamp: None,
            time_range: None,
            after: None,
            before: None,
            first: None,
            last: None,
        };

        assert_eq!(
            query.query_to_string(),
            format!("SELECT * FROM \"blocks\" WHERE \"producer_address\" = '{}' AND \"block_height\" = {}", TEST_PRODUCER_ADDRESS, TEST_BLOCK_HEIGHT)
        );

        // Test 2: all blocks after a given block_height, first items only
        let after_height_query = BlocksQuery {
            producer: None,
            height: None,
            timestamp: None,
            time_range: None,
            after: Some(TEST_BLOCK_HEIGHT),
            before: None,
            first: Some(FIRST_POINTER),
            last: None,
        };

        assert_eq!(
            after_height_query.query_to_string(),
            format!("SELECT * FROM \"blocks\" WHERE \"block_height\" > {} ORDER BY \"block_height\" ASC LIMIT {}", TEST_BLOCK_HEIGHT, FIRST_POINTER)
        );

        // Test 3: all blocks after a given timestamp, first items only
        let after_timestamp_query = BlocksQuery {
            producer: None,
            height: None,
            timestamp: Some(BlockTimestamp::from_secs(TEST_TIMESTAMP)),
            time_range: None,
            after: None,
            before: None,
            first: Some(FIRST_POINTER),
            last: None,
        };
        assert_eq!(
            after_timestamp_query.query_to_string(),
            format!("SELECT * FROM \"blocks\" WHERE \"timestamp\" >= {} ORDER BY \"block_height\" ASC LIMIT {}", TEST_TIMESTAMP, FIRST_POINTER)
        );

        // Test 4: all blocks before a given timestamp, last items only
        let before_timestamp_query = BlocksQuery {
            producer: None,
            height: None,
            timestamp: Some(BlockTimestamp::from_secs(TEST_TIMESTAMP)),
            time_range: None,
            after: None,
            before: None,
            last: Some(LAST_POINTER),
            first: None,
        };
        assert_eq!(
            before_timestamp_query.query_to_string(),
            format!("SELECT * FROM \"blocks\" WHERE \"timestamp\" >= {} ORDER BY \"block_height\" DESC LIMIT {}", TEST_TIMESTAMP, LAST_POINTER)
        );

        // Test 5: all blocks in the last 90 days
        let ninety_days_query = BlocksQuery {
            producer: None,
            height: None,
            timestamp: None,
            time_range: Some(TimeRange::NinetyDays),
            after: None,
            before: None,
            first: None,
            last: None,
        };
        let now = Utc::now();
        let ninety_days_ago = now - chrono::Duration::days(90);
        assert_eq!(
            ninety_days_query.query_to_string(),
            format!(
                "SELECT * FROM \"blocks\" WHERE \"timestamp\" >= {}",
                ninety_days_ago.timestamp()
            )
        );
    }

    #[test]
    fn test_blocks_query_from_query_string() {
        use serde_urlencoded;

        let query_string = format!("height={}&producer=0x0101010101010101010101010101010101010101010101010101010101010101&timeRange={}&timestamp={}&after={}&before={}&first={}&last={}",
            TEST_BLOCK_HEIGHT,
            TimeRange::OneHour,
            TEST_TIMESTAMP,
            AFTER_POINTER,
            BEFORE_POINTER,
            FIRST_POINTER,
            LAST_POINTER
        );
        let query: BlocksQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.height, Some(BlockHeight::from(TEST_BLOCK_HEIGHT)));
        assert_eq!(query.producer, Some(Address::from([1u8; 32])));
        assert_eq!(query.time_range, Some(TimeRange::OneHour));
        assert_eq!(
            query.timestamp,
            Some(BlockTimestamp::from_secs(TEST_TIMESTAMP))
        );
        assert_eq!(query.after, Some(AFTER_POINTER));
        assert_eq!(query.before, Some(BEFORE_POINTER));
        assert_eq!(query.first, Some(FIRST_POINTER));
        assert_eq!(query.last, Some(LAST_POINTER));
    }

    #[test]
    fn test_blocks_query_from_query_string_partial() {
        use serde_urlencoded;

        let query_string = format!("producer=0x0101010101010101010101010101010101010101010101010101010101010101&timeRange={}&after={}&before={}&first={}",
            TimeRange::OneHour,
            AFTER_POINTER,
            BEFORE_POINTER,
            FIRST_POINTER,
        );
        let query: BlocksQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.height, None);
        assert_eq!(query.producer, Some(Address::from([1u8; 32])));
        assert_eq!(query.time_range, Some(TimeRange::OneHour));
        assert_eq!(query.timestamp, None);
        assert_eq!(query.after, Some(AFTER_POINTER));
        assert_eq!(query.before, Some(BEFORE_POINTER));
        assert_eq!(query.first, Some(FIRST_POINTER));
        assert_eq!(query.last, None);
    }
}
