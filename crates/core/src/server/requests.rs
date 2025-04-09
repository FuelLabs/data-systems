use fuel_streams_subject::subject::SubjectPayload;
use fuel_web_utils::api_key::ApiKey;
use serde::{Deserialize, Serialize};

use crate::types::*;

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SubscribeRequest {
    #[serde(alias = "deliverPolicy")]
    pub deliver_policy: DeliverPolicy,
    pub subscribe: Vec<SubjectPayload>,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UnsubscribeRequest {
    #[serde(alias = "deliverPolicy")]
    pub deliver_policy: DeliverPolicy,
    pub unsubscribe: Vec<SubjectPayload>,
}

#[derive(Debug, thiserror::Error)]
pub enum ServerRequestError {
    #[error("Invalid request: {0}")]
    InvalidRequest(#[source] serde_json::Error),
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerRequest {
    #[serde(alias = "subscribe")]
    Subscribe(SubscribeRequest),
    #[serde(alias = "unsubscribe")]
    Unsubscribe(UnsubscribeRequest),
}

impl ServerRequest {
    pub fn subscriptions(&self, api_key: &ApiKey) -> Vec<Subscription> {
        let payload = match self {
            ServerRequest::Subscribe(req) => &req.subscribe,
            ServerRequest::Unsubscribe(req) => &req.unsubscribe,
        };
        let deliver_policy = match self {
            ServerRequest::Subscribe(req) => req.deliver_policy,
            ServerRequest::Unsubscribe(req) => req.deliver_policy,
        };

        let subjects = payload.clone();
        if subjects.is_empty() {
            tracing::debug!("No subscriptions requested");
            return vec![];
        }

        subjects
            .into_iter()
            .map(|payload| {
                Subscription::new(api_key, &deliver_policy, &payload)
            })
            .collect()
    }
}

impl TryFrom<&[u8]> for ServerRequest {
    type Error = ServerRequestError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        match serde_json::from_slice::<SubscribeRequest>(bytes) {
            Ok(subscribe_request) => {
                Ok(ServerRequest::Subscribe(subscribe_request))
            }
            Err(_) => {
                match serde_json::from_slice::<UnsubscribeRequest>(bytes) {
                    Ok(unsubscribe_request) => {
                        Ok(ServerRequest::Unsubscribe(unsubscribe_request))
                    }
                    Err(err) => Err(ServerRequestError::InvalidRequest(err)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_subscribe_request_snake_case() {
        let json = r#"{
            "deliver_policy": "new",
            "subscribe": []
        }"#;

        let request: SubscribeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.deliver_policy, DeliverPolicy::New);
        assert!(request.subscribe.is_empty());
    }

    #[test]
    fn test_subscribe_request_camel_case() {
        let json = r#"{
            "deliverPolicy": "new",
            "subscribe": []
        }"#;

        let request: SubscribeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.deliver_policy, DeliverPolicy::New);
        assert!(request.subscribe.is_empty());
    }

    #[test]
    fn test_unsubscribe_request_snake_case() {
        let json = r#"{
            "deliver_policy": "new",
            "unsubscribe": []
        }"#;

        let request: UnsubscribeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.deliver_policy, DeliverPolicy::New);
        assert!(request.unsubscribe.is_empty());
    }

    #[test]
    fn test_unsubscribe_request_camel_case() {
        let json = r#"{
            "deliverPolicy": "new",
            "unsubscribe": []
        }"#;

        let request: UnsubscribeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.deliver_policy, DeliverPolicy::New);
        assert!(request.unsubscribe.is_empty());
    }

    #[test]
    fn test_server_request_try_from_subscribe() {
        // Test snake_case
        let json = r#"{
            "deliver_policy": "new",
            "subscribe": []
        }"#;
        let request = ServerRequest::try_from(json.as_bytes()).unwrap();
        assert!(matches!(request, ServerRequest::Subscribe(_)));

        // Test camelCase
        let json = r#"{
            "deliverPolicy": "new",
            "subscribe": []
        }"#;
        let request = ServerRequest::try_from(json.as_bytes()).unwrap();
        assert!(matches!(request, ServerRequest::Subscribe(_)));
    }

    #[test]
    fn test_server_request_try_from_unsubscribe() {
        // Test snake_case
        let json = r#"{
            "deliver_policy": "new",
            "unsubscribe": []
        }"#;
        let request = ServerRequest::try_from(json.as_bytes()).unwrap();
        assert!(matches!(request, ServerRequest::Unsubscribe(_)));

        // Test camelCase
        let json = r#"{
            "deliverPolicy": "new",
            "unsubscribe": []
        }"#;
        let request = ServerRequest::try_from(json.as_bytes()).unwrap();
        assert!(matches!(request, ServerRequest::Unsubscribe(_)));
    }

    #[test]
    fn test_invalid_request() {
        let json = r#"{
            "invalid_field": "value"
        }"#;
        let result = ServerRequest::try_from(json.as_bytes());
        assert!(matches!(result, Err(ServerRequestError::InvalidRequest(_))));
    }
}
