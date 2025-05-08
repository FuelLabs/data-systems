use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::*;
use crate::infra::repository::{Repository, RepositoryError, RepositoryResult};

#[async_trait]
impl Repository for Message {
    type Item = MessageDbItem;
    type QueryParams = MessagesQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let created_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, MessageDbItem>(
            r#"
            INSERT INTO messages (
                subject,
                value,
                block_height,
                message_index,
                cursor,
                type,
                sender,
                recipient,
                nonce,
                amount,
                data,
                da_height,
                block_time,
                created_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14
            )
            ON CONFLICT (subject) DO UPDATE SET
                value = EXCLUDED.value,
                block_height = EXCLUDED.block_height,
                message_index = EXCLUDED.message_index,
                cursor = EXCLUDED.cursor,
                type = EXCLUDED.type,
                sender = EXCLUDED.sender,
                recipient = EXCLUDED.recipient,
                nonce = EXCLUDED.nonce,
                amount = EXCLUDED.amount,
                data = EXCLUDED.data,
                da_height = EXCLUDED.da_height,
                block_time = EXCLUDED.block_time,
                created_at = EXCLUDED.created_at
            RETURNING *
            "#,
        )
        .bind(&db_item.subject)
        .bind(&db_item.value)
        .bind(db_item.block_height.into_inner() as i64)
        .bind(db_item.message_index)
        .bind(db_item.cursor.to_string())
        .bind(db_item.r#type)
        .bind(&db_item.sender)
        .bind(&db_item.recipient)
        .bind(&db_item.nonce)
        .bind(db_item.amount)
        .bind(&db_item.data)
        .bind(db_item.da_height.into_inner() as i64)
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

    use fuel_streams_types::{BlockHeight, BlockTimestamp};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        blocks::repository::tests::insert_block,
        infra::{
            Db,
            DbConnectionOpts,
            OrderBy,
            QueryOptions,
            QueryParamsBuilder,
            RecordPointer,
        },
        messages::packets::DynMessageSubject,
        mocks::MockMessage,
    };

    async fn setup_db() -> anyhow::Result<(Arc<Db>, String)> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let namespace = QueryOptions::random_namespace();
        Ok((db, namespace))
    }

    fn assert_result(result: &MessageDbItem, expected: &MessageDbItem) {
        assert_eq!(result.subject, expected.subject);
        assert_eq!(result.value, expected.value);
        assert_eq!(result.block_height, expected.block_height);
        assert_eq!(result.message_index, expected.message_index);
        assert_eq!(result.cursor, expected.cursor);
        assert_eq!(result.r#type, expected.r#type);
        assert_eq!(result.sender, expected.sender);
        assert_eq!(result.recipient, expected.recipient);
        assert_eq!(result.nonce, expected.nonce);
        assert_eq!(result.amount, expected.amount);
        assert_eq!(result.data, expected.data);
        assert_eq!(result.da_height, expected.da_height);
    }

    pub async fn insert_message(
        db: &Arc<Db>,
        message: Option<Message>,
        height: BlockHeight,
        namespace: &str,
    ) -> anyhow::Result<(MessageDbItem, Message, DynMessageSubject)> {
        insert_block(db, height, namespace).await?;
        let message = message.unwrap_or_else(MockMessage::imported);
        let subject = DynMessageSubject::new(&message, height, 0);
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(&message, timestamps, RecordPointer {
                block_height: height,
                ..Default::default()
            })
            .with_namespace(namespace);

        let db_item = MessageDbItem::try_from(&packet)?;
        let result = Message::insert(db.pool_ref(), &db_item).await?;
        assert_result(&result, &db_item);

        Ok((db_item, message, subject))
    }

    async fn create_messages(
        db: &Arc<Db>,
        namespace: &str,
        count: u32,
    ) -> anyhow::Result<Vec<MessageDbItem>> {
        let mut messages = Vec::with_capacity(count as usize);
        for height in 1..=count {
            let (db_item, _, _) =
                insert_message(db, None, height.into(), namespace).await?;
            messages.push(db_item);
        }
        Ok(messages)
    }

    #[tokio::test]
    async fn test_inserting_imported_message() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let message = MockMessage::imported();
        let (db_item, _, _) =
            insert_message(&db, Some(message), 1.into(), &namespace).await?;
        assert_result(&db_item, &db_item);
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_consumed_message() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let message = MockMessage::consumed();
        let (db_item, _, _) =
            insert_message(&db, Some(message), 1.into(), &namespace).await?;
        assert_result(&db_item, &db_item);
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_all_message_types() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        for message in MockMessage::all() {
            let (db_item, _, _) =
                insert_message(&db, Some(message), 1.into(), &namespace)
                    .await?;
            assert_result(&db_item, &db_item);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_find_one_message() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let (db_item, _, subject) =
            insert_message(&db, None, 1.into(), &namespace).await?;
        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));
        let result = Message::find_one(db.pool_ref(), &query).await?;
        assert_result(&result, &db_item);
        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_messages_basic_query() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let messages = create_messages(&db, &namespace, 3).await?;

        let mut query = MessagesQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Message::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three messages");
        assert_result(&results[0], &messages[0]);
        assert_result(&results[1], &messages[1]);
        assert_result(&results[2], &messages[2]);

        Ok(())
    }
}
