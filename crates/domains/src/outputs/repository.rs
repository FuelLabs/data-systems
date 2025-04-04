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
        let created_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, OutputDbItem>(
            r#"
            INSERT INTO outputs (
                subject,
                value,
                block_height,
                tx_id,
                tx_index,
                output_index,
                cursor,
                type,
                amount,
                asset_id,
                to_address,
                state_root,
                balance_root,
                input_index,
                contract_id,
                block_time,
                created_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8::output_type, $9, $10,
                $11, $12, $13, $14, $15, $16, $17
            )
            ON CONFLICT (subject) DO UPDATE SET
                value = EXCLUDED.value,
                block_height = EXCLUDED.block_height,
                tx_id = EXCLUDED.tx_id,
                tx_index = EXCLUDED.tx_index,
                output_index = EXCLUDED.output_index,
                cursor = EXCLUDED.cursor,
                type = EXCLUDED.type,
                amount = EXCLUDED.amount,
                asset_id = EXCLUDED.asset_id,
                to_address = EXCLUDED.to_address,
                state_root = EXCLUDED.state_root,
                balance_root = EXCLUDED.balance_root,
                input_index = EXCLUDED.input_index,
                contract_id = EXCLUDED.contract_id,
                block_time = EXCLUDED.block_time,
                created_at = EXCLUDED.created_at
            RETURNING *
            "#,
        )
        .bind(&db_item.subject)
        .bind(&db_item.value)
        .bind(db_item.block_height.into_inner() as i64)
        .bind(&db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.output_index)
        .bind(db_item.cursor().to_string())
        .bind(db_item.r#type)
        .bind(db_item.amount)
        .bind(&db_item.asset_id)
        .bind(&db_item.to_address)
        .bind(&db_item.state_root)
        .bind(&db_item.balance_root)
        .bind(db_item.input_index)
        .bind(&db_item.contract_id)
        .bind(db_item.block_time)
        .bind(created_at)
        .fetch_one(executor)
        .await
        .map_err(RepositoryError::Insert)?;

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;
    use fuel_streams_types::BlockHeight;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        blocks::{
            packets::DynBlockSubject,
            repository::tests::insert_block,
            Block,
            BlockDbItem,
        },
        infra::{
            Db,
            DbConnectionOpts,
            OrderBy,
            QueryOptions,
            QueryParamsBuilder,
            RecordPointer,
        },
        mocks::{MockOutput, MockTransaction},
        outputs::DynOutputSubject,
        transactions::{
            repository::tests::insert_transaction,
            DynTransactionSubject,
            Transaction,
            TransactionDbItem,
        },
    };

    async fn setup_db() -> anyhow::Result<(Arc<Db>, String)> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let namespace = QueryOptions::random_namespace();
        Ok((db, namespace))
    }

    fn assert_result(result: &OutputDbItem, expected: &OutputDbItem) {
        assert_eq!(result.cursor(), expected.cursor());
        assert_eq!(result.subject, expected.subject);
        assert_eq!(result.value, expected.value);
        assert_eq!(result.block_height, expected.block_height);
        assert_eq!(result.tx_id, expected.tx_id);
        assert_eq!(result.tx_index, expected.tx_index);
        assert_eq!(result.output_index, expected.output_index);
        assert_eq!(result.r#type, expected.r#type);
        assert_eq!(result.amount, expected.amount);
        assert_eq!(result.asset_id, expected.asset_id);
        assert_eq!(result.to_address, expected.to_address);
        assert_eq!(result.state_root, expected.state_root);
        assert_eq!(result.balance_root, expected.balance_root);
        assert_eq!(result.input_index, expected.input_index);
        assert_eq!(result.contract_id, expected.contract_id);
        assert_eq!(
            result.block_time.into_inner().to_rfc3339(),
            expected.block_time.into_inner().to_rfc3339()
        );
    }

    async fn insert_random_block(
        db: &Arc<Db>,
        height: BlockHeight,
        namespace: &str,
    ) -> Result<(BlockDbItem, Block, DynBlockSubject)> {
        let (db_item, block, subject) =
            insert_block(db, height, namespace).await?;
        Ok((db_item, block, subject))
    }

    async fn insert_tx(
        db: &Arc<Db>,
        tx: &Transaction,
        height: BlockHeight,
        namespace: &str,
    ) -> Result<(TransactionDbItem, Transaction, DynTransactionSubject)> {
        let _ = insert_random_block(db, height, namespace).await?;
        insert_transaction(db, Some(tx.clone()), height, namespace).await
    }

    async fn insert_output(
        db: &Arc<Db>,
        tx: &Transaction,
        output: &Output,
        height: BlockHeight,
        namespace: &str,
        (tx_index, output_index): (i32, i32),
    ) -> Result<(OutputDbItem, Output, DynOutputSubject)> {
        let subject = DynOutputSubject::new(
            output,
            height,
            tx.id.to_owned(),
            tx_index,
            output_index,
            tx,
        );
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(output, timestamps, RecordPointer {
                block_height: height,
                tx_id: Some(tx.id.to_owned()),
                tx_index: Some(tx_index as u32),
                output_index: Some(output_index as u32),
                ..Default::default()
            })
            .with_namespace(namespace);

        let db_item = OutputDbItem::try_from(&packet)?;
        let result = Output::insert(db.pool_ref(), &db_item).await?;
        assert_result(&result, &db_item);

        Ok((db_item, output.clone(), subject))
    }

    async fn create_outputs(
        db: &Arc<Db>,
        namespace: &str,
        count: u32,
    ) -> Result<Vec<OutputDbItem>> {
        let mut outputs = Vec::with_capacity(count as usize);
        for _ in 0..count {
            outputs.push(MockOutput::coin(100))
        }

        let height = BlockHeight::random();
        let tx = MockTransaction::script(vec![], outputs.clone(), vec![]);
        insert_tx(db, &tx, height, namespace).await?;

        let mut db_items = Vec::with_capacity(count as usize);
        for (index, output) in outputs.iter().enumerate() {
            let (db_item, _, _) = insert_output(
                db,
                &tx,
                output,
                height,
                namespace,
                (0, index as i32),
            )
            .await?;
            db_items.push(db_item);
        }
        db_items.sort_by_key(|i| i.cursor());
        Ok(db_items)
    }

    #[tokio::test]
    async fn test_inserting_output_coin() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let output1 = MockOutput::coin(100);
        let output2 = MockOutput::coin(200);
        let tx = MockTransaction::script(
            vec![],
            vec![output1.clone(), output2.clone()],
            vec![],
        );
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_output(&db, &tx, &output1, height, &namespace, (0, 0)).await?;
        insert_output(&db, &tx, &output2, height, &namespace, (0, 1)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_contract() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let output = MockOutput::contract();
        let tx = MockTransaction::script(vec![], vec![output.clone()], vec![]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_output(&db, &tx, &output, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_change() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let output = MockOutput::change(50);
        let tx = MockTransaction::script(vec![], vec![output.clone()], vec![]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_output(&db, &tx, &output, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_variable() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let output = MockOutput::variable(75);
        let tx = MockTransaction::script(vec![], vec![output.clone()], vec![]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_output(&db, &tx, &output, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_contract_created() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let output = MockOutput::contract_created();
        let tx = MockTransaction::script(vec![], vec![output.clone()], vec![]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_output(&db, &tx, &output, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_one_output() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let output = MockOutput::coin(100);
        let tx = MockTransaction::script(vec![], vec![output.clone()], vec![]);
        insert_tx(&db, &tx, height, &namespace).await?;

        let (db_item, _, subject) =
            insert_output(&db, &tx, &output, height, &namespace, (0, 0))
                .await?;
        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));

        let result = Output::find_one(db.pool_ref(), &query).await?;
        assert_result(&result, &db_item);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_basic_query() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let outputs = create_outputs(&db, &namespace, 3).await?;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three outputs");
        assert_result(&results[0], &outputs[0]);
        assert_result(&results[1], &outputs[1]);
        assert_result(&results[2], &outputs[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_with_cursor_based_pagination_after(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let outputs = create_outputs(&db, &namespace, 5).await?;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_after(Some(outputs[1].cursor()));
        query.with_first(Some(2));

        let results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 outputs after cursor"
        );
        assert_result(&results[0], &outputs[2]);
        assert_result(&results[1], &outputs[3]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_with_cursor_based_pagination_before(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let outputs = create_outputs(&db, &namespace, 5).await?;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_before(Some(outputs[4].cursor()));
        query.with_last(Some(2));

        let results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 outputs before cursor"
        );
        assert_result(&results[0], &outputs[3]);
        assert_result(&results[1], &outputs[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_with_limit_offset_pagination() -> Result<()>
    {
        let (db, namespace) = setup_db().await?;
        let outputs = create_outputs(&db, &namespace, 5).await?;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_limit(Some(2));
        query.with_offset(Some(1));
        query.with_order_by(OrderBy::Asc);

        let results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 2, "Should return exactly 2 outputs");
        assert_result(&results[0], &outputs[1]);
        assert_result(&results[1], &outputs[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_with_different_order() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let outputs = create_outputs(&db, &namespace, 3).await?;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_result(&asc_results[0], &outputs[0]);
        assert_result(&asc_results[2], &outputs[2]);

        query.with_order_by(OrderBy::Desc);
        let desc_results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_result(&desc_results[0], &outputs[2]);
        assert_result(&desc_results[2], &outputs[0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_cursor_pagination_ignores_order_by() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let outputs = create_outputs(&db, &namespace, 5).await?;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace));
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
