use std::sync::Arc;

use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Predicate, PredicateDbItem, PredicatesQuery};
use crate::infra::{
    repository::{Repository, RepositoryResult},
    Db,
    DbItem,
};

impl Predicate {
    pub async fn upsert_as_relation(
        db: &Arc<Db>,
        db_item: &mut PredicateDbItem,
    ) -> RepositoryResult<PredicateDbItem> {
        let created_at = BlockTimestamp::now();
        db_item.created_at = created_at;

        let predicate_id = match sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO predicates (
                blob_id,
                predicate_address,
                block_time,
                created_at
            )
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (predicate_address) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(&db_item.blob_id)
        .bind(&db_item.predicate_address)
        .bind(db_item.block_time())
        .bind(db_item.created_at)
        .fetch_one(db.pool_ref())
        .await
        {
            Ok(id) => id,
            Err(sqlx::Error::RowNotFound) => sqlx::query_scalar::<_, i32>(
                r#" SELECT id FROM predicates WHERE predicate_address = $1 "#,
            )
            .bind(&db_item.predicate_address)
            .fetch_one(db.pool_ref())
            .await?,
            Err(e) => return Err(e.into()), // Propagate other errors
        };

        sqlx::query(
            r#"
            INSERT INTO predicate_transactions (
                predicate_id,
                cursor,
                subject,
                block_height,
                block_time,
                tx_id,
                tx_index,
                input_index,
                asset_id,
                bytecode
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(predicate_id)
        .bind(db_item.cursor().to_string())
        .bind(&db_item.subject)
        .bind(db_item.block_height.into_inner() as i64)
        .bind(db_item.block_time)
        .bind(&db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.input_index)
        .bind(&db_item.asset_id)
        .bind(&db_item.bytecode)
        .execute(db.pool_ref())
        .await?;

        Ok(db_item.to_owned())
    }
}

#[async_trait]
impl Repository for Predicate {
    type Item = PredicateDbItem;
    type QueryParams = PredicatesQuery;

    async fn insert<'e, 'c: 'e, E>(
        _executor: E,
        _db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        unreachable!("This method is not usable for predicate")
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fuel_streams_types::primitives::*;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        infra::{
            Db,
            DbConnectionOpts,
            DbItem,
            OrderBy,
            QueryOptions,
            QueryParamsBuilder,
            RecordPointer,
        },
        inputs::Input,
        mocks::{MockInput, MockTransaction},
        predicates::DynPredicateSubject,
    };

    async fn setup_db() -> anyhow::Result<(Arc<Db>, String)> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let namespace = QueryOptions::random_namespace();
        Ok((db, namespace))
    }

    fn assert_result(result: &PredicateDbItem, expected: &PredicateDbItem) {
        assert_eq!(result.cursor(), expected.cursor());
        assert_eq!(result.subject, expected.subject);
        assert_eq!(result.block_height, expected.block_height);
        assert_eq!(result.tx_id, expected.tx_id);
        assert_eq!(result.tx_index, expected.tx_index);
        assert_eq!(result.input_index, expected.input_index);
        assert_eq!(result.blob_id, expected.blob_id);
        assert_eq!(result.predicate_address, expected.predicate_address);
        assert_eq!(result.asset_id, expected.asset_id);
        assert_eq!(result.bytecode, expected.bytecode);
    }

    async fn insert_predicate(
        db: &Arc<Db>,
        input: Option<Input>,
        height: BlockHeight,
        block_time: BlockTimestamp,
        namespace: &str,
    ) -> anyhow::Result<(PredicateDbItem, Input, DynPredicateSubject)> {
        let input = input.unwrap_or_else(MockInput::coin_predicate);
        let tx =
            MockTransaction::script(vec![input.to_owned()], vec![], vec![]);
        let subject =
            DynPredicateSubject::new(&input, &height, &tx.id, 0, 0).unwrap();
        let packet = subject
            .build_packet(block_time, RecordPointer {
                block_height: height,
                tx_id: Some(tx.id.to_owned()),
                tx_index: Some(0_u32),
                input_index: Some(0_u32),
                ..Default::default()
            })
            .with_namespace(namespace);

        let mut db_item = PredicateDbItem::try_from(&packet)?;
        let result = Predicate::upsert_as_relation(db, &mut db_item).await?;
        assert_result(&result, &db_item);

        Ok((db_item, input, subject))
    }

    async fn create_predicates(
        db: &Arc<Db>,
        namespace: &str,
        count: u32,
    ) -> anyhow::Result<Vec<PredicateDbItem>> {
        let mut predicates = Vec::with_capacity(count as usize);
        for height in 1..=count {
            let (db_item, _, _) = insert_predicate(
                db,
                None,
                height.into(),
                BlockTimestamp::now(),
                namespace,
            )
            .await?;
            predicates.push(db_item);
        }
        Ok(predicates)
    }

    #[tokio::test]
    async fn test_inserting_predicate_with_blob_id() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_predicate(
            &db,
            Some(MockInput::coin_predicate()),
            1.into(),
            BlockTimestamp::now(),
            &namespace,
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_predicate_without_blob_id() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_predicate(
            &db,
            Some(MockInput::coin_signed(None)),
            1.into(),
            BlockTimestamp::now(),
            &namespace,
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_one_predicate() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let (db_item, _, subject) = insert_predicate(
            &db,
            None,
            1.into(),
            BlockTimestamp::now(),
            &namespace,
        )
        .await?;

        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));

        let result = Predicate::find_one(db.pool_ref(), &query).await?;
        assert_result(&result, &db_item);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_predicates_basic_query() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let predicates = create_predicates(&db, &namespace, 3).await?;

        let mut query = PredicatesQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Predicate::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three predicates");
        assert_result(&results[0], &predicates[0]);
        assert_result(&results[1], &predicates[1]);
        assert_result(&results[2], &predicates[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_predicates_with_cursor_based_pagination_after(
    ) -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let predicates = create_predicates(&db, &namespace, 5).await?;

        let mut query = PredicatesQuery::default();
        query.with_namespace(Some(namespace));
        query.with_after(Some(predicates[1].cursor()));
        query.with_first(Some(2));

        let results = Predicate::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 predicates after cursor"
        );
        assert_result(&results[0], &predicates[2]);
        assert_result(&results[1], &predicates[3]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_predicates_with_cursor_based_pagination_before(
    ) -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let predicates = create_predicates(&db, &namespace, 5).await?;

        let mut query = PredicatesQuery::default();
        query.with_namespace(Some(namespace));
        query.with_before(Some(predicates[3].cursor()));
        query.with_last(Some(2));

        let results = Predicate::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 predicates before cursor"
        );
        assert_result(&results[0], &predicates[2]);
        assert_result(&results[1], &predicates[1]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_predicates_with_limit_offset_pagination(
    ) -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let predicates = create_predicates(&db, &namespace, 5).await?;

        let mut query = PredicatesQuery::default();
        query.with_namespace(Some(namespace));
        query.with_limit(Some(2));
        query.with_offset(Some(1));
        query.with_order_by(OrderBy::Asc);

        let results = Predicate::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 2, "Should return exactly 2 predicates");
        assert_result(&results[0], &predicates[1]);
        assert_result(&results[1], &predicates[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_predicates_with_different_order(
    ) -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let predicates = create_predicates(&db, &namespace, 3).await?;

        let mut query = PredicatesQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Predicate::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_result(&asc_results[0], &predicates[0]);
        assert_result(&asc_results[2], &predicates[2]);

        query.with_order_by(OrderBy::Desc);
        let desc_results = Predicate::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_result(&desc_results[0], &predicates[2]);
        assert_result(&desc_results[2], &predicates[0]);

        Ok(())
    }
}
