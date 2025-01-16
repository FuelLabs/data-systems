use fuel_streams_core::DeliverPolicy;
use fuel_streams_domains::{SubjectPayload, SubjectPayloadError};
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionPayload {
    pub deliver_policy: DeliverPolicy,
    pub subject: String,
    pub params: serde_json::Value,
}
impl TryFrom<SubscriptionPayload> for SubjectPayload {
    type Error = SubjectPayloadError;
    fn try_from(payload: SubscriptionPayload) -> Result<Self, Self::Error> {
        SubjectPayload::new(payload.subject, payload.params)
    }
}
impl std::fmt::Display for SubscriptionPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{s}")
    }
}
impl TryFrom<String> for SubscriptionPayload {
    type Error = serde_json::Error;
    fn try_from(subscription_id: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&subscription_id)
    }
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
    Error(String),
    Response(ResponseMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub subject: String,
    pub payload: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::{ClientMessage, DeliverPolicy, SubscriptionPayload};

    #[test]
    fn test_sub_ser() {
        let stream_topic_wildcard = "blocks.*.*".to_owned();
        let msg = ClientMessage::Subscribe(SubscriptionPayload {
            subject: stream_topic_wildcard.clone(),
            params: serde_json::Value::Null,
            deliver_policy: DeliverPolicy::New,
        });
        let ser_str_value = serde_json::to_string(&msg).unwrap();
        println!("Ser value {:?}", ser_str_value);
        let expected_value = serde_json::json!({
            "subscribe": {
                "subject": stream_topic_wildcard,
                "params": serde_json::Value::Null,
                "deliverPolicy": "new"
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
