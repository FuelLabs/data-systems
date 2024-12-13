use fuel_streams_storage::DeliverPolicy;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionType {
    Stream(String),
}

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionPayload {
    pub topic: SubscriptionType,
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
    Update(Vec<u8>),
    Error(String),
}

#[cfg(test)]
mod tests {
    use fuel_streams_storage::DeliverPolicy;

    use super::{ClientMessage, SubscriptionPayload, SubscriptionType};

    #[test]
    fn test_sub_ser() {
        let stream_topic_wildcard = "blocks.*.*".to_owned();
        let msg = ClientMessage::Subscribe(SubscriptionPayload {
            topic: SubscriptionType::Stream(stream_topic_wildcard.clone()),
            deliver_policy: DeliverPolicy::All,
        });
        let ser_str_value = serde_json::to_string(&msg).unwrap();
        println!("Ser value {:?}", ser_str_value);
        let expected_value = serde_json::json!({
            "subscribe": {
                "topic": {
                    "stream": stream_topic_wildcard,
                    "deliver_policy": "all"
                }
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
