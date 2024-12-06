use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionTopic {
    Stream(String),
}

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionPayload {
    pub topic: SubscriptionTopic,
}

// -------------------CLIENT--------------------------//
#[derive(Eq, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ClientMessage {
    Ping,
    Subscribe(SubscriptionPayload),
    Unsubscribe(SubscriptionPayload),
}

// -------------------SERVER--------------------------//

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ServerMessage<T> {
    Pong,
    Subscribed(SubscriptionPayload),
    Unsubscribed(SubscriptionPayload),
    Update(T),
}

#[cfg(test)]
mod tests {
    use super::{ClientMessage, SubscriptionPayload, SubscriptionTopic};

    #[test]
    fn test_ping_ser() {
        let msg = ClientMessage::Ping;
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized =
            serde_json::from_str::<ClientMessage>(&serialized).unwrap();
        assert!(msg.eq(&deserialized));
    }

    #[test]
    fn test_sub_ser() {
        let stream_topic_wildcard = "blocks.*.*".to_owned();
        let msg = ClientMessage::Subscribe(SubscriptionPayload {
            topic: SubscriptionTopic::Stream(stream_topic_wildcard.clone()),
        });
        let ser_str_value = serde_json::to_string(&msg).unwrap();
        println!("Ser value {:?}", ser_str_value);
        let expected_value = serde_json::json!({
            "subscribe": {
                "topic": {
                    "stream": stream_topic_wildcard
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
