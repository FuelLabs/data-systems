use fuel_streams_nats::NatsDeliverPolicy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[serde(rename_all = "camelCase")]
pub enum DeliverPolicy {
    All,
    Last,
    New,
    ByStartSequence {
        #[serde(rename = "optStartSeq")]
        start_sequence: u64,
    },
    ByStartTime {
        #[serde(rename = "optStartTime")]
        start_time: time::OffsetDateTime,
    },
    LastPerSubject,
}

impl From<DeliverPolicy> for NatsDeliverPolicy {
    fn from(policy: DeliverPolicy) -> Self {
        match policy {
            DeliverPolicy::All => NatsDeliverPolicy::All,
            DeliverPolicy::Last => NatsDeliverPolicy::Last,
            DeliverPolicy::New => NatsDeliverPolicy::New,
            DeliverPolicy::ByStartSequence { start_sequence } => {
                NatsDeliverPolicy::ByStartSequence { start_sequence }
            }
            DeliverPolicy::ByStartTime { start_time } => {
                NatsDeliverPolicy::ByStartTime { start_time }
            }
            DeliverPolicy::LastPerSubject => NatsDeliverPolicy::LastPerSubject,
        }
    }
}

impl From<NatsDeliverPolicy> for DeliverPolicy {
    fn from(policy: NatsDeliverPolicy) -> Self {
        match policy {
            NatsDeliverPolicy::All => DeliverPolicy::All,
            NatsDeliverPolicy::Last => DeliverPolicy::Last,
            NatsDeliverPolicy::New => DeliverPolicy::New,
            NatsDeliverPolicy::ByStartSequence { start_sequence } => {
                DeliverPolicy::ByStartSequence { start_sequence }
            }
            NatsDeliverPolicy::ByStartTime { start_time } => {
                DeliverPolicy::ByStartTime { start_time }
            }
            NatsDeliverPolicy::LastPerSubject => DeliverPolicy::LastPerSubject,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionPayload {
    pub wildcard: String,
    pub deliver_policy: DeliverPolicy,
}

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ClientMessage {
    Subscribe(SubscriptionPayload),
    Unsubscribe(SubscriptionPayload),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ServerMessage {
    Subscribed(SubscriptionPayload),
    Unsubscribed(SubscriptionPayload),
    Response(serde_json::Value),
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::{ClientMessage, DeliverPolicy, SubscriptionPayload};

    #[test]
    fn test_sub_ser() {
        let stream_topic_wildcard = "blocks.*.*".to_owned();
        let msg = ClientMessage::Subscribe(SubscriptionPayload {
            wildcard: stream_topic_wildcard.clone(),
            deliver_policy: DeliverPolicy::All,
        });
        let ser_str_value = serde_json::to_string(&msg).unwrap();
        println!("Ser value {:?}", ser_str_value);
        let expected_value = serde_json::json!({
            "subscribe": {
                "wildcard": stream_topic_wildcard,
                "deliverPolicy": "all"
            }
        });
        let deser_msg_val =
            serde_json::from_value::<ClientMessage>(expected_value).unwrap();
        assert!(msg.eq(&deser_msg_val));

        let deser_msg_str =
            serde_json::from_str::<ClientMessage>(&ser_str_value).unwrap();
        assert!(msg.eq(&deser_msg_str));
    }
}
