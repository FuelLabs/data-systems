use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use dashmap::DashMap;

use super::{ApiKeyError, ApiKeyId, ApiKeyRole};

#[derive(Clone, Debug)]
struct RateLimiter {
    current_subscriptions: Arc<AtomicU64>,
    requests_this_minute: Arc<AtomicU64>,
    minute_start_time: Arc<AtomicU64>,
}

impl RateLimiter {
    fn new() -> Self {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        RateLimiter {
            current_subscriptions: Arc::new(AtomicU64::new(0)),
            requests_this_minute: Arc::new(AtomicU64::new(0)),
            minute_start_time: Arc::new(AtomicU64::new(current_time)),
        }
    }

    pub fn current_subscriptions(&self) -> u64 {
        self.current_subscriptions.load(Ordering::Relaxed)
    }

    pub fn minute_start_time(&self) -> u64 {
        self.minute_start_time.load(Ordering::Relaxed)
    }

    pub fn record_request(&self) -> u64 {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        let minute_start = self.minute_start_time();
        // If the current time is greater than the minute start time plus 60 seconds,
        // reset the requests this minute counter and update the minute start time
        if current_time >= minute_start + 60 {
            self.requests_this_minute.store(1, Ordering::Relaxed);
            self.minute_start_time
                .store(current_time, Ordering::Relaxed);
            1
        } else {
            // Otherwise, increment the requests this minute counter
            let current =
                self.requests_this_minute.fetch_add(1, Ordering::Relaxed);
            current + 1
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RateLimitsController {
    map: DashMap<ApiKeyId, RateLimiter>,
}

impl RateLimitsController {
    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    fn get_or_create(
        &self,
        id: &ApiKeyId,
    ) -> dashmap::mapref::one::Ref<'_, ApiKeyId, RateLimiter> {
        if !self.map.contains_key(id) {
            let new_limiter = RateLimiter::new();
            self.map.insert(id.to_owned(), new_limiter);
        }
        self.map.get(id).unwrap() // Safe because we just ensured it exists
    }

    pub fn add_active_key_sub(&self, id: &ApiKeyId) {
        if let Some(user_rate_limiter) = self.map.get_mut(id) {
            user_rate_limiter
                .current_subscriptions
                .fetch_add(1, Ordering::Relaxed);
        } else {
            let rate_limiter = RateLimiter::new();
            rate_limiter
                .current_subscriptions
                .fetch_add(1, Ordering::Relaxed);
            self.map.insert(id.to_owned(), rate_limiter);
        }
    }

    pub fn remove_active_key_sub(&self, id: &ApiKeyId) {
        if let Some(user_rate_limiter) = self.map.get_mut(id) {
            let current = user_rate_limiter
                .current_subscriptions
                .load(Ordering::Relaxed);
            if current > 0 {
                user_rate_limiter
                    .current_subscriptions
                    .fetch_sub(1, Ordering::Relaxed);
            }
        }
    }

    pub fn check_subscriptions(
        &self,
        api_key_id: &ApiKeyId,
        role: &ApiKeyRole,
    ) -> Result<(bool, u64), ApiKeyError> {
        let rate_limiter = self.get_or_create(api_key_id);
        let current_subscriptions = rate_limiter.current_subscriptions();
        let current_rate_limit =
            role.validate_subscription_limit(current_subscriptions.into())?;
        Ok((true, current_rate_limit.into()))
    }

    pub fn check_rate_limit(
        &self,
        api_key_id: &ApiKeyId,
        role: &ApiKeyRole,
    ) -> Result<(bool, u64), ApiKeyError> {
        let rate_limiter = self.get_or_create(api_key_id);
        let request_count = rate_limiter.record_request();
        let current_rate_limit =
            role.validate_rate_limit(request_count.into())?;
        Ok((true, current_rate_limit.into()))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::api_key::{ApiKeyId, MockApiKeyRole};

    // Setup function to create a rate limiter and API key with a specific role
    fn setup_rate_limiter(
        id: u64,
        role_type: &str,
    ) -> (RateLimitsController, ApiKeyId, ApiKeyRole) {
        let api_key_id = ApiKeyId::from(id);
        let role = match role_type {
            "admin" => MockApiKeyRole::admin().into_inner(),
            "builder" => MockApiKeyRole::builder().into_inner(),
            "web_client" => MockApiKeyRole::web_client().into_inner(),
            _ => MockApiKeyRole::web_client().into_inner(), /* Default to web_client */
        };
        let rate_limiter = RateLimitsController::default();
        (rate_limiter, api_key_id, role)
    }

    #[test]
    fn test_rate_limiter_add_remove_subscriptions() {
        let (rate_limiter, api_key_id, role) = setup_rate_limiter(1, "admin");

        // Add subscriptions
        rate_limiter.add_active_key_sub(&api_key_id);
        rate_limiter.add_active_key_sub(&api_key_id);

        // Check if subscriptions were added
        let (_, count) = rate_limiter
            .check_subscriptions(&api_key_id, &role)
            .expect("Failed to check rate limit");
        assert_eq!(count, 2, "Should have 2 active subscriptions");

        // Remove a subscription
        rate_limiter.remove_active_key_sub(&api_key_id);

        // Check if subscription was removed
        let (_, count) = rate_limiter
            .check_subscriptions(&api_key_id, &role)
            .expect("Failed to check rate limit");
        assert_eq!(count, 1, "Should have 1 active subscription after removal");
    }

    #[test]
    fn test_builder_subscription_limit() {
        let (rate_limiter, api_key_id, role) = setup_rate_limiter(3, "builder");

        // Add 50 subscriptions (at the limit)
        for _ in 0..50 {
            rate_limiter.add_active_key_sub(&api_key_id);
        }

        // At the limit - should still succeed
        let result = rate_limiter.check_subscriptions(&api_key_id, &role);
        assert!(
            result.is_ok(),
            "Subscription check should succeed at the limit"
        );

        // Add one more subscription (exceeding the limit)
        rate_limiter.add_active_key_sub(&api_key_id);

        // Now we're over the limit - should fail with SubscriptionLimitExceeded
        let result = rate_limiter.check_subscriptions(&api_key_id, &role);
        assert!(
            matches!(result, Err(ApiKeyError::SubscriptionLimitExceeded(_))),
            "Expected SubscriptionLimitExceeded error when exceeding subscription limit"
        );
    }

    #[test]
    fn test_new_key_initialization() {
        let (rate_limiter, api_key_id, role) =
            setup_rate_limiter(8, "web_client");

        // Key doesn't exist yet, should be initialized with 0 subscriptions
        let (success, count) = rate_limiter
            .check_subscriptions(&api_key_id, &role)
            .expect("Failed to check rate limit for new key");

        assert!(success, "New key check should succeed");
        assert_eq!(count, 0, "New key should start with 0 subscriptions");

        // Now the key should exist in the map
        rate_limiter.add_active_key_sub(&api_key_id);
        let (_, updated_count) = rate_limiter
            .check_subscriptions(&api_key_id, &role)
            .expect("Failed to check updated rate limit");
        assert_eq!(updated_count, 1, "Key should now have 1 subscription");
    }

    #[test]
    fn test_admin_role_unlimited() {
        // Test that admin role has no limits for both subscriptions and rate limits
        let (rate_limiter, api_key_id, role) = setup_rate_limiter(4, "admin");

        // Test unlimited subscriptions
        for _ in 0..2000 {
            rate_limiter.add_active_key_sub(&api_key_id);
        }
        let sub_result = rate_limiter.check_subscriptions(&api_key_id, &role);
        assert!(
            sub_result.is_ok(),
            "Admin role should have no subscription limit"
        );

        // Test unlimited rate limit
        for i in 0..2000 {
            let rate_result = rate_limiter.check_rate_limit(&api_key_id, &role);
            assert!(
                rate_result.is_ok(),
                "Request {} should be allowed for admin",
                i
            );
        }
    }

    #[test]
    fn test_web_client_rate_limit_and_reset() {
        let (rate_limiter, api_key_id, role) =
            setup_rate_limiter(10, "web_client");

        // Make 1000 requests (at the limit)
        for i in 0..1000 {
            let result = rate_limiter.check_rate_limit(&api_key_id, &role);
            assert!(result.is_ok(), "Request {} should be allowed", i);
            let (_, count) = result.unwrap();
            assert_eq!(count, i + 1, "Request count should match");
        }

        // Make one more request (exceeding the limit)
        let result = rate_limiter.check_rate_limit(&api_key_id, &role);
        assert!(
            matches!(result, Err(ApiKeyError::RateLimitExceeded(_))),
            "Request 1001 should be denied"
        );

        // Test rate limit reset
        // Manually set the minute start time to 61 seconds ago to simulate time passing
        if let Some(limiter) = rate_limiter.map.get_mut(&api_key_id) {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();

            limiter
                .minute_start_time
                .store(current_time - 61, Ordering::Relaxed);

            // Set requests to a high number
            limiter.requests_this_minute.store(999, Ordering::Relaxed);
        }

        // Next request should reset the counter since a minute has passed
        let result = rate_limiter.check_rate_limit(&api_key_id, &role);
        assert!(result.is_ok(), "Request after reset should be allowed");
        let (_, count) = result.unwrap();
        assert_eq!(count, 1, "Counter should be reset to 1");
    }

    #[test]
    fn test_multiple_keys_management() {
        // Test comprehensive management of multiple keys with different roles
        // Create multiple API key IDs with different roles
        let (rate_limiter, admin_key_id, admin_role) =
            setup_rate_limiter(5, "admin");
        let builder_key_id = ApiKeyId::from(6);
        let web_client_key_id = ApiKeyId::from(7);
        let builder_role = MockApiKeyRole::builder().into_inner();
        let web_client_role = MockApiKeyRole::web_client().into_inner();

        // Add different numbers of subscriptions to each key
        for _ in 0..100 {
            rate_limiter.add_active_key_sub(&admin_key_id);
        }

        for _ in 0..30 {
            rate_limiter.add_active_key_sub(&builder_key_id);
        }

        for _ in 0..500 {
            rate_limiter.add_active_key_sub(&web_client_key_id);
        }

        // Check subscription counts for each key
        let (_, admin_count) = rate_limiter
            .check_subscriptions(&admin_key_id, &admin_role)
            .expect("Failed to check rate limit for admin key");
        assert_eq!(
            admin_count, 100,
            "Admin key should have 100 active subscriptions"
        );

        let (_, builder_count) = rate_limiter
            .check_subscriptions(&builder_key_id, &builder_role)
            .expect("Failed to check rate limit for builder key");
        assert_eq!(
            builder_count, 30,
            "Builder key should have 30 active subscriptions"
        );

        let (_, web_client_count) = rate_limiter
            .check_subscriptions(&web_client_key_id, &web_client_role)
            .expect("Failed to check rate limit for web client key");
        assert_eq!(
            web_client_count, 500,
            "Web client key should have 500 active subscriptions"
        );

        // Removing subscriptions from one key shouldn't affect the others
        rate_limiter.remove_active_key_sub(&admin_key_id);

        let (_, updated_admin_count) = rate_limiter
            .check_subscriptions(&admin_key_id, &admin_role)
            .expect("Failed to check updated rate limit for admin key");
        assert_eq!(
            updated_admin_count, 99,
            "Admin key should now have 99 active subscriptions"
        );

        let (_, unchanged_builder_count) = rate_limiter
            .check_subscriptions(&builder_key_id, &builder_role)
            .expect("Failed to check unchanged rate limit for builder key");
        assert_eq!(
            unchanged_builder_count, 30,
            "Builder key should still have 30 active subscriptions"
        );

        // Test rate limits for multiple keys simultaneously
        for i in 0..1200 {
            // Admin key (unlimited)
            let admin_result =
                rate_limiter.check_rate_limit(&admin_key_id, &admin_role);
            assert!(
                admin_result.is_ok(),
                "Admin request {} should be allowed",
                i
            );

            // Web client key (limit 1000)
            let web_client_result = rate_limiter
                .check_rate_limit(&web_client_key_id, &web_client_role);

            if i < 1000 {
                assert!(
                    web_client_result.is_ok(),
                    "Web client request {} should be allowed",
                    i
                );
            } else {
                assert!(
                    matches!(
                        web_client_result,
                        Err(ApiKeyError::RateLimitExceeded(_))
                    ),
                    "Web client request {} should be denied",
                    i
                );
            }
        }
    }
}
