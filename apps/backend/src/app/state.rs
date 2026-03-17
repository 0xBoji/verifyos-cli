use crate::app::{RateLimiter, ScanService};

#[derive(Clone)]
pub struct AppState {
    pub scan: ScanService,
    pub rate_limit: RateLimiter,
}

impl AppState {
    pub fn new(scan: ScanService, rate_limit: RateLimiter) -> Self {
        Self { scan, rate_limit }
    }
}
