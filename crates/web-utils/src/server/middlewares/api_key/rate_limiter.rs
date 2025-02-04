use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use dashmap::DashMap;

#[derive(Clone, Debug)]
struct RateLimiter {
    requests_per_key: Arc<AtomicU64>,
    max_requests_per_key: u64,
}

impl RateLimiter {
    fn new(max_requests_per_key: u64) -> Self {
        RateLimiter {
            requests_per_key: Arc::new(AtomicU64::new(0)),
            max_requests_per_key,
        }
    }
}

#[derive(Debug, Default)]
pub struct RateLimitsController {
    map: DashMap<u64, RateLimiter>,
    max_requests_per_key: u64,
}

impl RateLimitsController {
    pub fn new(max_requests_per_key: u64) -> Self {
        Self {
            map: DashMap::new(),
            max_requests_per_key,
        }
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn add_active_key_sub(&self, user_id: u64) {
        if let Some(user_rate_limiter) = self.map.get_mut(&user_id) {
            user_rate_limiter
                .requests_per_key
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn remove_active_key_sub(&self, user_id: u64) {
        if let Some(user_rate_limiter) = self.map.get_mut(&user_id) {
            user_rate_limiter
                .requests_per_key
                .fetch_sub(1, Ordering::Relaxed);
        }
    }

    pub fn check_rate_limit(&self, user_id: u64) -> (bool, u64) {
        let user_rate_limiter = self.map.get(&user_id);
        match user_rate_limiter {
            Some(rate_limiter) => {
                let requests_per_key =
                    rate_limiter.requests_per_key.load(Ordering::Relaxed);
                if requests_per_key >= rate_limiter.max_requests_per_key {
                    return (false, rate_limiter.max_requests_per_key);
                }
                (true, self.max_requests_per_key)
            }
            None => {
                let rate_limiter = RateLimiter::new(self.max_requests_per_key);
                self.map.insert(user_id, rate_limiter);
                (true, self.max_requests_per_key)
            }
        }
    }
}

impl Clone for RateLimitsController {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
            max_requests_per_key: self.max_requests_per_key,
        }
    }
}
