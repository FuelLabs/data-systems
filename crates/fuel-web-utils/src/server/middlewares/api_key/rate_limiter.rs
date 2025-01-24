use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
struct RateLimiter {
    last_request_time: Arc<RwLock<Instant>>,
    rate_limit: Duration,
}

impl RateLimiter {
    fn new(rate_limit: Duration) -> Self {
        RateLimiter {
            last_request_time: Arc::new(RwLock::new(Instant::now())),
            rate_limit,
        }
    }
}

#[derive(Debug, Default)]
pub struct RateLimitsController {
    map: DashMap<u64, RateLimiter>,
    rate_limit: Duration,
}

impl RateLimitsController {
    pub fn new(rate_limit: Duration) -> Self {
        Self {
            map: DashMap::new(),
            rate_limit,
        }
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub async fn check_rate_limit(&self, user_id: u64) -> bool {
        let user_rate_limiter = self.map.get(&user_id);
        match user_rate_limiter {
            Some(rate_limiter) => {
                let last_request_time =
                    rate_limiter.last_request_time.read().await;
                let elapsed = last_request_time.elapsed();
                if elapsed < rate_limiter.rate_limit {
                    return false;
                }
                true
            }
            None => {
                let rate_limiter = RateLimiter::new(self.rate_limit);
                self.map.insert(user_id, rate_limiter);
                true
            }
        }
    }
}

impl Clone for RateLimitsController {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
            rate_limit: self.rate_limit,
        }
    }
}
