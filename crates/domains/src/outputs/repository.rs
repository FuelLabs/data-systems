use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Output, OutputDbItem, OutputsQuery};
use crate::infra::repository::{Repository, RepositoryError, RepositoryResult};

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
                    subject, value, block_height, tx_id, tx_index,
                    output_index, output_type, to_address, asset_id, contract_id,
                    created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    output_index = EXCLUDED.output_index,
                    output_type = EXCLUDED.output_type,
                    to_address = EXCLUDED.to_address,
                    asset_id = EXCLUDED.asset_id,
                    contract_id = EXCLUDED.contract_id,
                    created_at = EXCLUDED.created_at,
                    published_at = $12
                RETURNING *
            )
            SELECT * FROM upsert",
        )
        .bind(db_item.subject.clone())
        .bind(db_item.value.to_owned())
        .bind(db_item.block_height)
        .bind(db_item.tx_id.to_owned())
        .bind(db_item.tx_index)
        .bind(db_item.output_index)
        .bind(db_item.output_type.to_owned())
        .bind(db_item.to_address.to_owned())
        .bind(db_item.asset_id.to_owned())
        .bind(db_item.contract_id.to_owned())
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
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        infra::{Db, DbConnectionOpts, QueryOptions, ToPacket},
        mocks::{MockOutput, MockTransaction},
        outputs::DynOutputSubject,
    };

    async fn test_output(output: &Output) -> anyhow::Result<()> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let tx =
            MockTransaction::script(vec![], vec![output.to_owned()], vec![]);
        let namespace = QueryOptions::random_namespace();
        let subject = DynOutputSubject::from((
            output,
            1.into(),
            tx.id.clone(),
            0,
            0,
            &tx,
        ));
        let timestamps = BlockTimestamp::default();
        let packet = output
            .to_packet(&subject.into(), timestamps)
            .with_namespace(&namespace);

        let db_item = OutputDbItem::try_from(&packet)?;
        let result = Output::insert(db.pool_ref(), &db_item).await;
        assert!(result.is_ok());

        let result = result.unwrap();
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
    async fn test_inserting_output_coin() -> anyhow::Result<()> {
        test_output(&MockOutput::coin(100)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_contract() -> anyhow::Result<()> {
        test_output(&MockOutput::contract()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_change() -> anyhow::Result<()> {
        test_output(&MockOutput::change(50)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_variable() -> anyhow::Result<()> {
        test_output(&MockOutput::variable(75)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_contract_created() -> anyhow::Result<()> {
        test_output(&MockOutput::contract_created()).await?;
        Ok(())
    }
}
