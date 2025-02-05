use serde::{Deserialize, Serialize};

use super::{StreamMessage, SubscriptionPayload};

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ServerRequest {
    Subscriptions(Vec<SubscriptionPayload>),
    Subscribe(SubscriptionPayload),
    Unsubscribe(SubscriptionPayload),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ServerResponse {
    Subscribed(SubscriptionPayload),
    Unsubscribed(SubscriptionPayload),
    Error(String),
    Response(StreamMessage),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        server::{DeliverPolicy, ServerRequest, SubscriptionPayload},
        subjects::*,
    };

    #[test]
    fn test_sub_ser() {
        let stream_topic_subject = "blocks.*.*".to_owned();
        let msg = ServerRequest::Subscribe(SubscriptionPayload {
            subject: stream_topic_subject.clone(),
            params: json!({}).to_string(),
            deliver_policy: DeliverPolicy::New,
        });
        let ser_str_value = serde_json::to_string(&msg).unwrap();
        println!("Ser value {:?}", ser_str_value);
        let expected_value = serde_json::json!({
            "subscribe": {
                "subject": stream_topic_subject,
                "params": json!({}).to_string(),
                "deliverPolicy": "new"
            }
        });
        let deser_msg_val =
            serde_json::from_value::<ServerRequest>(expected_value).unwrap();
        assert!(msg.eq(&deser_msg_val));

        let deser_msg_str =
            serde_json::from_str::<ServerRequest>(&ser_str_value).unwrap();
        assert!(msg.eq(&deser_msg_str));
    }
}
