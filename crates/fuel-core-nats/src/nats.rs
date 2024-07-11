mod subjects;
pub use subjects::*;

use anyhow::Context;
use async_nats::jetstream::stream;
use tracing::info;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Nats {
    pub stream_name: String,
    pub sandbox_id: String,
    pub jetstream: async_nats::jetstream::Context,
    /// Messages published to jetstream
    pub jetstream_messages: async_nats::jetstream::stream::Stream,
    /// Max publishing payload in connected NATS server
    max_payload_size: usize,
    subjects: Vec<String>,
}

impl Nats {
    pub async fn connect(
        nats_url: &str,
        nats_nkey: Option<String>,
        sandbox_id: &Option<String>,
    ) -> anyhow::Result<Self> {
        let sandbox_id = &(sandbox_id.clone().unwrap_or_default());
        let config = stream::Config {
            name: format!("{sandbox_id}fuel"),
            subjects: subjects::get_all_in_sandbox(sandbox_id),
            storage: async_nats::jetstream::stream::StorageType::File,
            ..Default::default()
        };

        let client = match nats_nkey {
            Some(nkey) => async_nats::connect_with_options(
                nats_url,
                async_nats::ConnectOptions::with_nkey(nkey),
            )
            .await
            .context(format!("Connecting to {nats_url}"))?,
            None => async_nats::connect(nats_url)
                .await
                .context(format!("Connecting to {nats_url}"))?,
        };

        let max_payload_size = client.server_info().max_payload;
        info!("NATS Publisher: max_payload_size={max_payload_size}");

        // Create a JetStream context
        let jetstream = async_nats::jetstream::new(client);

        let subjects = config.subjects.clone();
        let stream_name = config.name.clone();
        let jetstream_messages = jetstream.get_or_create_stream(config).await?;

        Ok(Self {
            stream_name,
            sandbox_id: sandbox_id.to_string(),
            jetstream,
            jetstream_messages,
            max_payload_size,
            subjects,
        })
    }

    /// A wrapper around JetStream::publish() that also checks that the payload size does not exceed NATS server's max_payload_size.
    pub async fn publish(
        &self,
        subject: Subject,
        payload: bytes::Bytes,
        sandbox_id: &Option<String>,
    ) -> anyhow::Result<()> {
        let subject = &subject.get_value(sandbox_id);
        info!("NATS: Publishing: {subject}");

        // Check message size
        let payload_size = payload.len();
        if payload_size > self.max_payload_size {
            anyhow::bail!(
                "{subject} payload size={payload_size} exceeds max_payload_size={}",
                self.max_payload_size
            )
        }
        // Publish
        let ack_future = self.jetstream.publish(subject.to_string(), payload).await?;
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

#[cfg(test)]
pub fn random_sandbox_id() -> String {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let random_int: u16 = rng.gen();
    format!("sandbox-{random_int}")
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use async_nats::jetstream::stream::LastRawMessageErrorKind;

    #[tokio::test]
    async fn returns_authorization_error_without_nkey() {
        assert!(Nats::connect(NATS_URL, None, &Some(random_sandbox_id()))
            .await
            .is_err_and(|e| {
                e.source()
                    .expect("An error source must exist")
                    .to_string()
                    .contains("authorization violation: nats: authorization violation")
            }));
    }

    #[tokio::test]
    async fn connects_to_nats_with_nkey() {
        setup_env();

        let nats = Nats::connect(NATS_URL, nkey(), &Some(random_sandbox_id()))
            .await
            .expect(&format!("Ensure NATS server is running at {NATS_URL}"));

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
        setup_env();

        let nats = Nats::connect(NATS_URL, nkey(), &Some(random_sandbox_id()))
            .await
            .expect(&format!("Ensure NATS server is running at {NATS_URL}"));

        assert_eq!(nats.max_payload_size, 8_388_608)
    }

    pub async fn get_nats_connection(sandbox_id: &str) -> Nats {
        setup_env();

        Nats::connect(NATS_URL, nkey(), &Some(sandbox_id.to_string()))
            .await
            .expect(&format!("Ensure NATS server is running at {NATS_URL}"))
    }

    const NATS_URL: &str = "nats://localhost:4222";
    fn setup_env() {
        dotenvy::dotenv().ok();
    }
    fn nkey() -> Option<String> {
        std::env::var("NATS_NKEY").ok()
    }
}
