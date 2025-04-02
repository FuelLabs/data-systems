use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Predicate, PredicateDbItem, PredicatesQuery};
use crate::infra::{
    repository::{Repository, RepositoryError, RepositoryResult},
    DbItem,
};

#[async_trait]
impl Repository for Predicate {
    type Item = PredicateDbItem;
    type QueryParams = PredicatesQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, PredicateDbItem>(
            r#"
            WITH inserted_predicate AS (
                INSERT INTO predicates (
                    blob_id,
                    predicate_address,
                    created_at,
                    published_at
                )
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (predicate_address) DO UPDATE
                SET blob_id = EXCLUDED.blob_id,
                    created_at = EXCLUDED.created_at,
                    published_at = EXCLUDED.published_at
                RETURNING id, blob_id, predicate_address, created_at, published_at
            ),
            inserted_transaction AS (
                INSERT INTO predicate_transactions (
                    predicate_id,
                    cursor,
                    subject,
                    block_height,
                    tx_id,
                    tx_index,
                    input_index,
                    asset_id,
                    bytecode
                )
                SELECT
                    id,
                    $5,
                    $6,
                    $7,
                    $8,
                    $9,
                    $10,
                    $11,
                    $12
                FROM inserted_predicate
                RETURNING predicate_id
            )
            SELECT
                p.id,
                p.blob_id,
                p.predicate_address,
                p.created_at,
                p.published_at,
                $5 AS cursor,
                $6 AS subject,
                $7 AS block_height,
                $8 AS tx_id,
                $9 AS tx_index,
                $10 AS input_index,
                $11 AS asset_id,
                $12 AS bytecode
            FROM inserted_predicate p
            "#,
        )
        .bind(&db_item.blob_id)
        .bind(&db_item.predicate_address)
        .bind(db_item.created_at)
        .bind(published_at)
        .bind(db_item.cursor().to_string())
        .bind(&db_item.subject)
        .bind(db_item.block_height)
        .bind(&db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.input_index)
        .bind(&db_item.asset_id)
        .bind(&db_item.bytecode)
        .fetch_one(executor)
        .await
        .map_err(|e| {
            eprintln!("SQL error inserting predicate: {:?}", e);
            RepositoryError::Insert(e)
        })?;

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
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
        },
        inputs::Input,
        mocks::{MockInput, MockTransaction},
        predicates::DynPredicateSubject,
    };

    async fn test_predicate(input: &Input) -> anyhow::Result<()> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let tx =
            MockTransaction::script(vec![input.to_owned()], vec![], vec![]);
        let namespace = QueryOptions::random_namespace();

        // Create predicate subject
        let subject =
            DynPredicateSubject::new(input, &1.into(), &tx.id, 0, 0).unwrap();

        let timestamps = BlockTimestamp::default();
        let packet =
            subject.build_packet(timestamps).with_namespace(&namespace);

        let db_item = PredicateDbItem::try_from(&packet)?;
        let result = Predicate::insert(db.pool_ref(), &db_item).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(db_item.cursor(), result.cursor());
        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.tx_id, db_item.tx_id);
        assert_eq!(result.tx_index, db_item.tx_index);
        assert_eq!(result.input_index, db_item.input_index);
        assert_eq!(result.blob_id, db_item.blob_id);
        assert_eq!(result.predicate_address, db_item.predicate_address);
        assert_eq!(result.asset_id, db_item.asset_id);
        assert_eq!(result.bytecode, db_item.bytecode);
        assert_eq!(result.created_at, db_item.created_at);

        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_predicate_with_blob_id() -> anyhow::Result<()> {
        let input = MockInput::coin_predicate();
        test_predicate(&input).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_predicate_without_blob_id() -> anyhow::Result<()> {
        let input = MockInput::coin_signed();
        test_predicate(&input).await?;
        Ok(())
    }

    async fn create_predicates(
        namespace: &str,
        db: &Db,
        count: u32,
    ) -> Vec<PredicateDbItem> {
        let mut predicates = Vec::with_capacity(count as usize);
        for height in 1..=count {
            let input = MockInput::coin_predicate();
            let tx =
                MockTransaction::script(vec![input.to_owned()], vec![], vec![]);
            let subject =
                DynPredicateSubject::new(&input, &height.into(), &tx.id, 0, 0)
                    .unwrap();
            let timestamps = BlockTimestamp::default();
            let packet =
                subject.build_packet(timestamps).with_namespace(namespace);
            let db_item = PredicateDbItem::try_from(&packet).unwrap();

            Predicate::insert(db.pool_ref(), &db_item).await.unwrap();
            predicates.push(db_item);
        }
        predicates
    }

    #[tokio::test]
    async fn test_find_one_predicate() -> anyhow::Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;

        let input = MockInput::coin_predicate();
        let tx =
            MockTransaction::script(vec![input.to_owned()], vec![], vec![]);
        let subject =
            DynPredicateSubject::new(&input, &1.into(), &tx.id, 0, 0).unwrap();
        let timestamps = BlockTimestamp::default();
        let packet =
            subject.build_packet(timestamps).with_namespace(&namespace);
        let db_item = PredicateDbItem::try_from(&packet)?;

        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));

        Predicate::insert(db.pool_ref(), &db_item).await?;
        let result = Predicate::find_one(db.pool_ref(), &query).await?;

        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.tx_id, db_item.tx_id);
        assert_eq!(result.blob_id, db_item.blob_id);
        assert_eq!(result.predicate_address, db_item.predicate_address);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_predicates_basic_query() -> anyhow::Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let predicates = create_predicates(&namespace, &db, 3).await;

        let mut query = PredicatesQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Predicate::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three predicates");
        assert_eq!(results[0].subject, predicates[0].subject);
        assert_eq!(results[1].subject, predicates[1].subject);
        assert_eq!(results[2].subject, predicates[2].subject);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_predicates_with_cursor_based_pagination_after(
    ) -> anyhow::Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let predicates = create_predicates(&namespace, &db, 5).await;

        let mut query = PredicatesQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_after(Some(predicates[1].cursor()));
        query.with_first(Some(2));

        let results = Predicate::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 predicates after cursor"
        );
        assert_eq!(results[0].cursor(), predicates[2].cursor());
        assert_eq!(results[1].cursor(), predicates[3].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_predicates_with_cursor_based_pagination_before(
    ) -> anyhow::Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let predicates = create_predicates(&namespace, &db, 5).await;

        let mut query = PredicatesQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_before(Some(predicates[3].cursor()));
        query.with_last(Some(2));

        let results = Predicate::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 predicates before cursor"
        );

        assert_eq!(results[0].cursor(), predicates[2].cursor());
        assert_eq!(results[1].cursor(), predicates[1].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_predicates_with_limit_offset_pagination(
    ) -> anyhow::Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let predicates = create_predicates(&namespace, &db, 5).await;

        let mut query = PredicatesQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_limit(Some(2));
        query.with_offset(Some(1));
        query.with_order_by(OrderBy::Asc);

        let results = Predicate::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 2, "Should return exactly 2 predicates");
        assert_eq!(results[0].cursor(), predicates[1].cursor());
        assert_eq!(results[1].cursor(), predicates[2].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_predicates_with_different_order(
    ) -> anyhow::Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let predicates = create_predicates(&namespace, &db, 3).await;
        let mut query = PredicatesQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Predicate::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_eq!(asc_results[0].cursor(), predicates[0].cursor());
        assert_eq!(asc_results[2].cursor(), predicates[2].cursor());

        query.with_order_by(OrderBy::Desc);
        let desc_results = Predicate::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_eq!(desc_results[0].cursor(), predicates[2].cursor());
        assert_eq!(desc_results[2].cursor(), predicates[0].cursor());

        Ok(())
    }
}
