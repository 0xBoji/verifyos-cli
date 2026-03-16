use crate::app::{AuthStore, ScanService};

#[derive(Clone)]
pub struct AppState {
    pub scan: ScanService,
    pub auth: AuthStore,
    pub require_auth: bool,
}

impl AppState {
    pub fn new(scan: ScanService, auth: AuthStore, require_auth: bool) -> Self {
        Self {
            scan,
            auth,
            require_auth,
        }
    }
}
