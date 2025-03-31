use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use super::types::*;
use crate::infra::{record::QueryOptions, repository::SubjectQueryBuilder};

#[derive(
    Subject, Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq,
)]
#[subject(id = "blocks")]
#[subject(entity = "Block")]
#[subject(query_all = "blocks.>")]
#[subject(format = "blocks.{producer}.{da_height}.{height}")]
pub struct BlocksSubject {
    #[subject(
        sql_column = "producer_address",
        description = "The address of the producer that created the block"
    )]
    pub producer: Option<Address>,
    #[subject(
        sql_column = "block_da_height",
        description = "The height of the DA block as unsigned 64 bit integer"
    )]
    pub da_height: Option<DaBlockHeight>,
    #[subject(
        sql_column = "block_height",
        description = "The height of the block as unsigned 64 bit integer"
    )]
    pub height: Option<BlockHeight>,
}

impl From<&Block> for BlocksSubject {
    fn from(block: &Block) -> Self {
        BlocksSubject {
            producer: Some(block.producer.to_owned()),
            da_height: Some(block.header.da_height.to_owned()),
            height: Some(block.height.to_owned()),
        }
    }
}

impl SubjectQueryBuilder for BlocksSubject {
    fn query_builder(
        &self,
        options: Option<&QueryOptions>,
    ) -> QueryBuilder<'static, Postgres> {
        let mut conditions = Vec::new();
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();
        query_builder.push("SELECT * FROM blocks");

        if let Some(where_clause) = self.to_sql_where() {
            conditions.push(where_clause);
        }
        if let Some(block) = options.map(|o| o.from_block.unwrap_or_default()) {
            conditions.push(format!("block_height >= {}", block));
        }

        if !conditions.is_empty() {
            query_builder.push(" WHERE ");
            query_builder.push(conditions.join(" AND "));
        }

        query_builder.push(" ORDER BY block_height ASC");
        if let Some(opts) = options {
            opts.apply_limit_offset(&mut query_builder);
        }

        query_builder
    }
}
