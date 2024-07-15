mod subjects;
pub use subjects::*;

use anyhow::Context;
use async_nats::jetstream::stream;
use tracing::info;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NatsConnection {
    pub id: String,
    pub stream_name: String,
    pub jetstream: async_nats::jetstream::Context,
    /// Messages published to jetstream
    pub jetstream_messages: async_nats::jetstream::stream::Stream,
    /// Max publishing payload in connected NATS server
    max_payload_size: usize,
    subjects: Vec<String>,
}

impl NatsConnection {
    /// A wrapper around JetStream::publish() that also checks that the payload size does not exceed NATS server's max_payload_size.
    pub async fn publish(
        &self,
        subject: Subject,
        payload: bytes::Bytes,
    ) -> anyhow::Result<()> {
        let subject = subject.get_string(&self.id);
        info!("NATS: Publishing: {subject}");

        // Check message size
        let payload_size = payload.len();
        if payload_size > self.max_payload_size {
            let subject = &subject;
            anyhow::bail!(
                "{subject} payload size={payload_size} exceeds max_payload_size={}",
                self.max_payload_size
            )
        }
        // Publish
        let ack_future = self.jetstream.publish(subject, payload).await?;
        // Wait for an ACK
        ack_future.await?;
        Ok(())
    }

    #[cfg(test)]
    pub async fn has_no_message(&self) -> bool {
        let raw_messages_by_all_subjects =
            self.get_last_raw_messages_by_all_subjects().await;

        raw_messages_by_all_subjects.iter().all(|result| {
            result.as_ref().is_err_and(|e| {
                e.kind() == stream::LastRawMessageErrorKind::NoMessageFound
            })
        })
    }

    #[cfg(test)]
    pub async fn get_last_raw_messages_by_all_subjects(
        &self,
    ) -> Vec<
        Result<
            stream::RawMessage,
            async_nats::error::Error<stream::LastRawMessageErrorKind>,
        >,
    > {
        let mut results = vec![];

        for subject in &self.subjects {
            let result = self
                .jetstream_messages
                .get_last_raw_message_by_subject(subject)
                .await;

            results.push(result);
        }

        results
    }
}

pub async fn connect(
    nats_url: &str,
    nats_nkey: &str,
    connection_id: Option<String>,
) -> anyhow::Result<NatsConnection> {
    let connection_id = &connection_id.unwrap_or_default();
    let config = stream::Config {
        name: format!("{connection_id}fuel"),
        subjects: subjects::get_all_in_connection(connection_id),
        storage: async_nats::jetstream::stream::StorageType::File,
        ..Default::default()
    };

    let client = async_nats::connect_with_options(
        nats_url,
        async_nats::ConnectOptions::with_nkey(nats_nkey.to_string()),
    )
    .await
    .context(format!("Connecting to {nats_url}"))?;

    let max_payload_size = client.server_info().max_payload;
    info!("NATS Publisher: max_payload_size={max_payload_size}");

    // Create a JetStream context
    let jetstream = async_nats::jetstream::new(client);

    let id = connection_id.clone();
    let subjects = config.subjects.clone();
    let stream_name = config.name.clone();
    let jetstream_messages = jetstream.get_or_create_stream(config).await?;

    Ok(NatsConnection {
        id,
        stream_name,
        jetstream,
        jetstream_messages,
        max_payload_size,
        subjects,
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use async_nats::jetstream::stream::LastRawMessageErrorKind;

    #[tokio::test]
    async fn returns_signature_error_empty_nkey() {
        assert!(connect(&get_url(), "", Some(get_random_connection_id()))
            .await
            .is_err_and(|e| {
                e.source()
                    .expect("An error source must exist")
                    .to_string()
                    .contains("failed signing nonce")
            }));
    }

    #[tokio::test]
    async fn returns_authorization_error_invalid_nkey() {
        assert!(connect(
            &get_url(),
            "some-invalid-nkey",
            Some(get_random_connection_id())
        )
        .await
        .is_err_and(|e| {
            e.source()
                .expect("An error source must exist")
                .to_string()
                .contains("failed signing nonce")
        }));
    }

    #[tokio::test]
    async fn connects_to_nats_with_nkey() {
        let nats = get_nats_connection(&get_random_connection_id()).await;

        assert!(nats
            .get_last_raw_messages_by_all_subjects()
            .await
            .iter()
            .all(|result| {
                result.as_ref().is_err_and(|err| {
                    err.kind() == LastRawMessageErrorKind::NoMessageFound
                })
            }));
    }

    #[tokio::test]
    async fn returns_max_payload_size_allowed_on_the_connection() {
        let nats = get_nats_connection(&get_random_connection_id()).await;

        assert_eq!(nats.max_payload_size, 8_388_608)
    }

    pub fn get_random_connection_id() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let connection_id: u32 = rng.gen();
        format!("connection-{connection_id}")
    }

    pub async fn get_nats_connection(connection_id: &str) -> NatsConnection {
        let url = &get_url();

        connect(url, &get_nkey(), Some(connection_id.to_string()))
            .await
            .expect(&format!("Ensure NATS server is running at {url}"))
    }
    fn get_url() -> String {
        dotenvy::var("NATS_URL").unwrap_or("nats://localhost:4222".to_string())
    }
    fn get_nkey() -> String {
        dotenvy::var("NATS_NKEY_SEED").unwrap()
    }
}
