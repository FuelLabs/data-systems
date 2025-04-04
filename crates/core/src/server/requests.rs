use fuel_streams_subject::subject::SubjectPayload;
use fuel_web_utils::api_key::ApiKey;
use serde::{Deserialize, Serialize};

use crate::types::*;

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SubscribeRequest {
    pub deliver_policy: DeliverPolicy,
    pub subscribe: Vec<SubjectPayload>,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UnsubscribeRequest {
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
    Subscribe(SubscribeRequest),
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
