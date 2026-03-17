use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("rate limit exceeded")]
    Exceeded,
}

#[derive(Debug)]
struct Window {
    start: Instant,
    count: u32,
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    inner: Arc<RateLimiterInner>,
}

#[derive(Debug)]
struct RateLimiterInner {
    windows: RwLock<HashMap<String, Window>>,
    limit_per_min: u32,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        let limit_per_min = env_u64("RATE_LIMIT_PER_MIN", 60) as u32;
        Self {
            inner: Arc::new(RateLimiterInner {
                windows: RwLock::new(HashMap::new()),
                limit_per_min: limit_per_min.max(1),
            }),
        }
    }

    pub async fn check(&self, key: &str) -> Result<(), RateLimitError> {
        let now = Instant::now();
        let mut windows = self.inner.windows.write().await;
        let entry = windows.entry(key.to_string()).or_insert(Window {
            start: now,
            count: 0,
        });

        if now.duration_since(entry.start) >= Duration::from_secs(60) {
            entry.start = now;
            entry.count = 0;
        }

        if entry.count >= self.inner.limit_per_min {
            return Err(RateLimitError::Exceeded);
        }

        entry.count += 1;
        Ok(())
    }
}

fn env_u64(key: &str, fallback: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(fallback)
}
