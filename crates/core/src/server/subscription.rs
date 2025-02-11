use fuel_streams_subject::subject::SubjectPayload;
use fuel_web_utils::server::middlewares::api_key::ApiKey;
use serde::{Deserialize, Serialize};

use super::DeliverPolicy;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub deliver_policy: DeliverPolicy,
    pub payload: SubjectPayload,
}

impl Subscription {
    pub fn new(
        api_key: &ApiKey,
        deliver_policy: &DeliverPolicy,
        payload: &SubjectPayload,
    ) -> Self {
        Self {
            id: Self::create_subscription_id(api_key, payload),
            deliver_policy: deliver_policy.to_owned(),
            payload: payload.to_owned(),
        }
    }

    fn create_subscription_id(
        api_key: &ApiKey,
        payload: &SubjectPayload,
    ) -> String {
        format!("{}-{}-{}", api_key.id(), api_key.user(), payload)
    }
}

impl std::fmt::Display for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}
