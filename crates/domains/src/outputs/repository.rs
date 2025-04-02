use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Output, OutputDbItem, OutputsQuery};
use crate::infra::{
    repository::{Repository, RepositoryError, RepositoryResult},
    DbItem,
};

#[async_trait]
impl Repository for Output {
    type Item = OutputDbItem;
    type QueryParams = OutputsQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, OutputDbItem>(
            "WITH upsert AS (
                INSERT INTO outputs (
                    subject, value, cursor, block_height, tx_id, tx_index,
                    output_index, output_type, to_address, asset_id, contract_id,
                    created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    cursor = EXCLUDED.cursor,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    output_index = EXCLUDED.output_index,
                    output_type = EXCLUDED.output_type,
                    to_address = EXCLUDED.to_address,
                    asset_id = EXCLUDED.asset_id,
                    contract_id = EXCLUDED.contract_id,
                    created_at = EXCLUDED.created_at,
                    published_at = $13
                RETURNING *
            )
            SELECT * FROM upsert",
        )
        .bind(&db_item.subject)
        .bind(&db_item.value)
        .bind(db_item.cursor().to_string())
        .bind(db_item.block_height)
        .bind(&db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.output_index)
        .bind(&db_item.output_type)
        .bind(&db_item.to_address)
        .bind(&db_item.asset_id)
        .bind(&db_item.contract_id)
        .bind(db_item.created_at)
        .bind(published_at)
        .fetch_one(executor)
        .await
        .map_err(RepositoryError::Insert)?;

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
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
        mocks::{MockOutput, MockTransaction},
        outputs::DynOutputSubject,
    };

    async fn test_output(output: &Output) -> Result<()> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let tx =
            MockTransaction::script(vec![], vec![output.to_owned()], vec![]);
        let namespace = QueryOptions::random_namespace();
        let subject =
            DynOutputSubject::new(output, 1.into(), tx.id.clone(), 0, 0, &tx);
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(output, timestamps)
            .with_namespace(&namespace);

        let db_item = OutputDbItem::try_from(&packet)?;
        let result = Output::insert(db.pool_ref(), &db_item).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(db_item.cursor(), result.cursor());
        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.value, db_item.value);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.tx_id, db_item.tx_id);
        assert_eq!(result.tx_index, db_item.tx_index);
        assert_eq!(result.output_index, db_item.output_index);
        assert_eq!(result.output_type, db_item.output_type);
        assert_eq!(result.to_address, db_item.to_address);
        assert_eq!(result.asset_id, db_item.asset_id);
        assert_eq!(result.contract_id, db_item.contract_id);
        assert_eq!(result.created_at, db_item.created_at);

        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_coin() -> Result<()> {
        test_output(&MockOutput::coin(100)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_contract() -> Result<()> {
        test_output(&MockOutput::contract()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_change() -> Result<()> {
        test_output(&MockOutput::change(50)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_variable() -> Result<()> {
        test_output(&MockOutput::variable(75)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_contract_created() -> Result<()> {
        test_output(&MockOutput::contract_created()).await?;
        Ok(())
    }

    async fn create_test_output(
        height: u32,
        namespace: &str,
    ) -> (OutputDbItem, Output, DynOutputSubject) {
        let output = MockOutput::coin(100);
        let tx =
            MockTransaction::script(vec![], vec![output.to_owned()], vec![]);
        let subject = DynOutputSubject::new(
            &output,
            height.into(),
            tx.id.clone(),
            0,
            0,
            &tx,
        );
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(&output, timestamps)
            .with_namespace(namespace);
        let db_item = OutputDbItem::try_from(&packet).unwrap();
        (db_item, output, subject)
    }

    async fn create_outputs(
        namespace: &str,
        db: &Db,
        count: u32,
    ) -> Vec<OutputDbItem> {
        let mut outputs = Vec::with_capacity(count as usize);
        for height in 1..=count {
            let (db_item, _, _) = create_test_output(height, namespace).await;
            Output::insert(db.pool_ref(), &db_item).await.unwrap();
            outputs.push(db_item);
        }
        outputs
    }

    #[tokio::test]
    async fn test_find_one_output() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let (db_item, _, subject) = create_test_output(1, &namespace).await;
        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));

        Output::insert(db.pool_ref(), &db_item).await?;
        let result = Output::find_one(db.pool_ref(), &query).await?;
        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.value, db_item.value);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.tx_id, db_item.tx_id);
        assert_eq!(result.output_type, db_item.output_type);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_basic_query() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let outputs = create_outputs(&namespace, &db, 3).await;
        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three outputs");
        assert_eq!(results[0].subject, outputs[0].subject);
        assert_eq!(results[1].subject, outputs[1].subject);
        assert_eq!(results[2].subject, outputs[2].subject);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_with_cursor_based_pagination_after(
    ) -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let outputs = create_outputs(&namespace, &db, 5).await;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_after(Some(outputs[1].cursor()));
        query.with_first(Some(2));

        let results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 outputs after cursor"
        );
        assert_eq!(results[0].cursor(), outputs[2].cursor());
        assert_eq!(results[1].cursor(), outputs[3].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_with_cursor_based_pagination_before(
    ) -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let outputs = create_outputs(&namespace, &db, 5).await;
        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_before(Some(outputs[4].cursor()));
        query.with_last(Some(2));

        let results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 outputs before cursor"
        );
        assert_eq!(results[0].cursor(), outputs[3].cursor());
        assert_eq!(results[1].cursor(), outputs[2].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_with_limit_offset_pagination() -> Result<()>
    {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let outputs = create_outputs(&namespace, &db, 5).await;

        // Test first page
        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_limit(Some(2));
        query.with_offset(Some(0));
        query.with_order_by(OrderBy::Asc);

        let first_page = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(first_page.len(), 2, "First page should have 2 outputs");
        assert_eq!(first_page[0].cursor(), outputs[0].cursor());
        assert_eq!(first_page[1].cursor(), outputs[1].cursor());

        // Test second page
        query.with_offset(Some(2));
        let second_page = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(second_page.len(), 2, "Second page should have 2 outputs");
        assert_eq!(second_page[0].cursor(), outputs[2].cursor());
        assert_eq!(second_page[1].cursor(), outputs[3].cursor());

        // Test last page
        query.with_offset(Some(4));
        let last_page = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(last_page.len(), 1, "Last page should have 1 output");
        assert_eq!(last_page[0].cursor(), outputs[4].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_with_different_order() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let outputs = create_outputs(&namespace, &db, 3).await;

        // Test ascending order
        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_eq!(asc_results[0].cursor(), outputs[0].cursor());
        assert_eq!(asc_results[2].cursor(), outputs[2].cursor());

        // Test descending order
        query.with_order_by(OrderBy::Desc);
        let desc_results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_eq!(desc_results[0].cursor(), outputs[2].cursor());
        assert_eq!(desc_results[2].cursor(), outputs[0].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_cursor_pagination_ignores_order_by() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let outputs = create_outputs(&namespace, &db, 5).await;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_after(Some(outputs[1].cursor()));
        query.with_first(Some(2));

        let results_default = Output::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Asc);

        let results_asc = Output::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Desc);

        let results_desc = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results_default, results_asc);
        assert_eq!(results_default, results_desc);

        Ok(())
    }
}
