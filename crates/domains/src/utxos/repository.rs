use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Utxo, UtxoDbItem, UtxosQuery};
use crate::infra::{
    repository::{Repository, RepositoryError, RepositoryResult},
    DbItem,
};

#[async_trait]
impl Repository for Utxo {
    type Item = UtxoDbItem;
    type QueryParams = UtxosQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let created_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, UtxoDbItem>(
            r#"
            INSERT INTO utxos (
                subject,
                value,
                block_height,
                tx_id,
                tx_index,
                output_index,
                cursor,
                utxo_id,
                type,
                status,
                amount,
                asset_id,
                from_address,
                to_address,
                nonce,
                contract_id,
                block_time,
                created_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9::utxo_type, $10::utxo_status,
                $11, $12, $13, $14, $15, $16, $17, $18
            )
            ON CONFLICT (utxo_id)
            DO UPDATE SET
                subject = EXCLUDED.subject,
                value = EXCLUDED.value,
                block_height = EXCLUDED.block_height,
                tx_id = EXCLUDED.tx_id,
                tx_index = EXCLUDED.tx_index,
                output_index = EXCLUDED.output_index,
                cursor = EXCLUDED.cursor,
                type = EXCLUDED.type,
                status = EXCLUDED.status,
                amount = EXCLUDED.amount,
                asset_id = EXCLUDED.asset_id,
                from_address = EXCLUDED.from_address,
                to_address = EXCLUDED.to_address,
                nonce = EXCLUDED.nonce,
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
        .bind(&db_item.utxo_id)
        .bind(db_item.r#type)
        .bind(db_item.status)
        .bind(db_item.amount)
        .bind(&db_item.asset_id)
        .bind(&db_item.from_address)
        .bind(&db_item.to_address)
        .bind(&db_item.nonce)
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
    use std::{str::FromStr, sync::Arc};

    use anyhow::Result;
    use fuel_streams_types::{BlockHeight, UtxoId, UtxoStatus};
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
        inputs::Input,
        mocks::MockInput,
        outputs::{MockOutput, Output},
        transactions::{
            repository::tests::insert_transaction,
            DynTransactionSubject,
            MockTransaction,
            Transaction,
            TransactionDbItem,
        },
        utxos::DynUtxoSubject,
    };

    async fn setup_db() -> Result<(Arc<Db>, String)> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let namespace = QueryOptions::random_namespace();
        Ok((db, namespace))
    }

    fn assert_result(result: &UtxoDbItem, expected: &UtxoDbItem) {
        assert_eq!(result.cursor(), expected.cursor());
        assert_eq!(result.subject, expected.subject);
        assert_eq!(result.value, expected.value);
        assert_eq!(result.block_height, expected.block_height);
        assert_eq!(result.tx_id, expected.tx_id);
        assert_eq!(result.tx_index, expected.tx_index);
        assert_eq!(result.output_index, expected.output_index);
        assert_eq!(result.utxo_id, expected.utxo_id);
        assert_eq!(result.r#type, expected.r#type);
        assert_eq!(result.status, expected.status);
        assert_eq!(result.amount, expected.amount);
        assert_eq!(result.asset_id, expected.asset_id);
        assert_eq!(result.from_address, expected.from_address);
        assert_eq!(result.to_address, expected.to_address);
        assert_eq!(result.nonce, expected.nonce);
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
        insert_block(db, height, namespace).await
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

    async fn insert_utxo(
        db: &Arc<Db>,
        tx: &Transaction,
        input: Option<&Input>,
        output: Option<&Output>,
        height: BlockHeight,
        namespace: &str,
        indices: (i32, i32),
    ) -> Result<(UtxoDbItem, DynUtxoSubject)> {
        let (tx_index, output_index) = indices;
        let subject = if let Some(input) = input {
            DynUtxoSubject::from_input(input, height, tx.id.clone(), tx_index)
                .unwrap()
        } else if let Some(output) = output {
            DynUtxoSubject::from_output(
                output,
                height,
                tx.id.clone(),
                tx_index,
                output_index,
            )
            .unwrap()
        } else {
            panic!("Either input or output must be provided");
        };

        let timestamps = BlockTimestamp::default();
        let pointer = RecordPointer {
            block_height: height,
            tx_id: Some(tx.id.clone()),
            tx_index: Some(tx_index as u32),
            output_index: Some(output_index as u32),
            ..Default::default()
        };
        let packet = subject
            .build_packet(timestamps, pointer)
            .with_namespace(namespace);
        let db_item = UtxoDbItem::try_from(&packet)?;
        let result = Utxo::insert(db.pool_ref(), &db_item).await?;
        assert_result(&result, &db_item);

        Ok((db_item, subject))
    }

    async fn create_utxos(
        db: &Arc<Db>,
        namespace: &str,
        count: u32,
    ) -> Result<Vec<UtxoDbItem>> {
        let mut utxos = Vec::with_capacity(count as usize);
        let height = BlockHeight::random();
        let outputs: Vec<Output> =
            (0..count).map(|_| MockOutput::coin(100)).collect();
        let tx = MockTransaction::script(vec![], outputs.clone(), vec![]);
        insert_tx(db, &tx, height, namespace).await?;

        for (output_index, output) in outputs.iter().enumerate() {
            let (db_item, _) = insert_utxo(
                db,
                &tx,
                None,
                Some(output),
                height,
                namespace,
                (0, output_index as i32),
            )
            .await?;
            utxos.push(db_item);
        }

        utxos.sort_by_key(|i| i.cursor());
        Ok(utxos)
    }

    #[tokio::test]
    async fn test_find_one_utxo() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let output = MockOutput::coin(100);
        let tx = MockTransaction::script(vec![], vec![output.clone()], vec![]);
        insert_tx(&db, &tx, height, &namespace).await?;

        let (db_item, subject) = insert_utxo(
            &db,
            &tx,
            None,
            Some(&output),
            height,
            &namespace,
            (0, 0),
        )
        .await?;

        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));
        let result = Utxo::find_one(db.pool_ref(), &query).await?;
        assert_result(&result, &db_item);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_utxos_basic_query() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let utxos = create_utxos(&db, &namespace, 3).await?;
        let mut query = UtxosQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three UTXOs");
        assert_result(&results[0], &utxos[0]);
        assert_result(&results[1], &utxos[1]);
        assert_result(&results[2], &utxos[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_utxos_with_cursor_based_pagination_after(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let utxos = create_utxos(&db, &namespace, 5).await?;
        let mut query = UtxosQuery::default();
        query.with_namespace(Some(namespace));
        query.with_after(Some(utxos[1].cursor()));
        query.with_first(Some(2));

        let results = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 UTXOs after cursor"
        );
        assert_result(&results[0], &utxos[2]);
        assert_result(&results[1], &utxos[3]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_utxos_with_cursor_based_pagination_before(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let utxos = create_utxos(&db, &namespace, 5).await?;
        let mut query = UtxosQuery::default();
        query.with_namespace(Some(namespace));
        query.with_before(Some(utxos[4].cursor()));
        query.with_last(Some(2));

        let results = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 UTXOs before cursor"
        );
        assert_result(&results[0], &utxos[3]);
        assert_result(&results[1], &utxos[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_utxos_with_limit_offset_pagination() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let utxos = create_utxos(&db, &namespace, 5).await?;

        let mut query = UtxosQuery::default();
        query.with_namespace(Some(namespace));
        query.with_limit(Some(2));
        query.with_offset(Some(1));
        query.with_order_by(OrderBy::Asc);

        let results = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 2, "Should return exactly 2 UTXOs");
        assert_result(&results[0], &utxos[1]);
        assert_result(&results[1], &utxos[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_utxos_with_different_order() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let utxos = create_utxos(&db, &namespace, 3).await?;

        let mut query = UtxosQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_result(&asc_results[0], &utxos[0]);
        assert_result(&asc_results[2], &utxos[2]);

        query.with_order_by(OrderBy::Desc);
        let desc_results = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_result(&desc_results[0], &utxos[2]);
        assert_result(&desc_results[2], &utxos[0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_cursor_pagination_ignores_order_by() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let utxos = create_utxos(&db, &namespace, 5).await?;

        let mut query = UtxosQuery::default();
        query.with_namespace(Some(namespace));
        query.with_after(Some(utxos[1].cursor()));
        query.with_first(Some(2));

        let results_default = Utxo::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Asc);
        let results_asc = Utxo::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Desc);
        let results_desc = Utxo::find_many(db.pool_ref(), &query).await?;

        assert_eq!(results_default, results_asc);
        assert_eq!(results_default, results_desc);

        Ok(())
    }

    #[tokio::test]
    async fn test_utxo_being_spent() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let output = MockOutput::coin(100);
        let tx = MockTransaction::script(vec![], vec![output.clone()], vec![]);
        insert_tx(&db, &tx, height, &namespace).await?;

        let (db_item_1, _) = insert_utxo(
            &db,
            &tx,
            None,
            Some(&output),
            height,
            &namespace,
            (0, 0),
        )
        .await?;

        let mut query = UtxosQuery::default();
        let utxo_id = UtxoId::from_str(&db_item_1.utxo_id).unwrap();
        query.set_utxo_id(Some(utxo_id.clone()));
        query.with_namespace(Some(namespace.clone()));

        let result = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status, UtxoStatus::Unspent);

        let input = MockInput::coin_signed(Some(utxo_id.clone()));
        let (db_item_2, _) = insert_utxo(
            &db,
            &tx,
            Some(&input),
            None,
            height,
            &namespace,
            (0, 0),
        )
        .await?;

        let mut query = UtxosQuery::default();
        query.set_utxo_id(Some(utxo_id.clone()));
        query.with_namespace(Some(namespace.clone()));

        let result = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status, UtxoStatus::Spent);
        assert_eq!(db_item_1.utxo_id, db_item_2.utxo_id);

        Ok(())
    }
}
