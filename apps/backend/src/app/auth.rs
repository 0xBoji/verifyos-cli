use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthToken {
    pub token: String,
    pub email: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("auth token missing")]
    MissingToken,
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("too many requests")]
    RateLimited,
    #[error("invalid or expired code")]
    InvalidCode,
    #[error("invalid oauth state")]
    InvalidState,
}

#[derive(Debug)]
struct PendingCode {
    code: String,
    expires_at: Instant,
}

#[derive(Debug)]
struct Session {
    email: String,
    expires_at: Instant,
    window_start: Instant,
    window_count: u32,
}

#[derive(Debug, Clone)]
pub struct AuthStore {
    inner: Arc<AuthStoreInner>,
}

#[derive(Debug)]
struct AuthStoreInner {
    pending: RwLock<HashMap<String, PendingCode>>,
    pending_oauth: RwLock<HashMap<String, Instant>>,
    sessions: RwLock<HashMap<String, Session>>,
    code_ttl: Duration,
    session_ttl: Duration,
    rate_limit_per_min: u32,
    dev_mode: bool,
    oauth_state_ttl: Duration,
}

impl AuthStore {
    pub fn new() -> Self {
        let code_ttl = env_u64("AUTH_CODE_TTL_MIN", 10);
        let session_ttl = env_u64("AUTH_SESSION_TTL_HOURS", 24);
        let rate_limit_per_min = env_u64("RATE_LIMIT_PER_MIN", 20) as u32;
        let dev_mode = std::env::var("AUTH_DEV_MODE")
            .ok()
            .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));

        Self {
            inner: Arc::new(AuthStoreInner {
                pending: RwLock::new(HashMap::new()),
                pending_oauth: RwLock::new(HashMap::new()),
                sessions: RwLock::new(HashMap::new()),
                code_ttl: Duration::from_secs(code_ttl * 60),
                session_ttl: Duration::from_secs(session_ttl * 60 * 60),
                rate_limit_per_min: rate_limit_per_min.max(1),
                dev_mode,
                oauth_state_ttl: Duration::from_secs(10 * 60),
            }),
        }
    }

    pub fn dev_mode(&self) -> bool {
        self.inner.dev_mode
    }

    pub async fn start_login(&self, email: &str) -> String {
        let code = Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .take(6)
            .collect::<String>()
            .to_uppercase();
        let expires_at = Instant::now() + self.inner.code_ttl;
        let mut pending = self.inner.pending.write().await;
        pending.insert(
            email.to_string(),
            PendingCode {
                code: code.clone(),
                expires_at,
            },
        );
        code
    }

    pub async fn verify_code(&self, email: &str, code: &str) -> Result<AuthToken, AuthError> {
        let now = Instant::now();
        let mut pending = self.inner.pending.write().await;
        let Some(entry) = pending.get(email) else {
            return Err(AuthError::InvalidCode);
        };
        if entry.expires_at < now || entry.code != code {
            return Err(AuthError::InvalidCode);
        }
        pending.remove(email);
        self.issue_session(email).await
    }

    pub async fn issue_session(&self, email: &str) -> Result<AuthToken, AuthError> {
        let now = Instant::now();
        let token = Uuid::new_v4().to_string();
        let expires_at = now + self.inner.session_ttl;
        let mut sessions = self.inner.sessions.write().await;
        sessions.insert(
            token.clone(),
            Session {
                email: email.to_string(),
                expires_at,
                window_start: now,
                window_count: 0,
            },
        );

        Ok(AuthToken {
            token,
            email: email.to_string(),
            expires_in_seconds: self.inner.session_ttl.as_secs(),
        })
    }

    pub async fn start_oauth_state(&self) -> String {
        let state = Uuid::new_v4().to_string();
        let expires_at = Instant::now() + self.inner.oauth_state_ttl;
        let mut pending = self.inner.pending_oauth.write().await;
        pending.insert(state.clone(), expires_at);
        state
    }

    pub async fn consume_oauth_state(&self, state: &str) -> Result<(), AuthError> {
        let now = Instant::now();
        let mut pending = self.inner.pending_oauth.write().await;
        let Some(expires_at) = pending.get(state) else {
            return Err(AuthError::InvalidState);
        };
        if *expires_at < now {
            pending.remove(state);
            return Err(AuthError::InvalidState);
        }
        pending.remove(state);
        Ok(())
    }

    pub async fn authorize(&self, token: &str) -> Result<String, AuthError> {
        let now = Instant::now();
        let mut sessions = self.inner.sessions.write().await;
        let Some(session) = sessions.get_mut(token) else {
            return Err(AuthError::InvalidToken);
        };
        if session.expires_at < now {
            sessions.remove(token);
            return Err(AuthError::InvalidToken);
        }
        if now.duration_since(session.window_start) >= Duration::from_secs(60) {
            session.window_start = now;
            session.window_count = 0;
        }
        if session.window_count >= self.inner.rate_limit_per_min {
            return Err(AuthError::RateLimited);
        }
        session.window_count += 1;
        Ok(session.email.clone())
    }
}

fn env_u64(key: &str, fallback: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(fallback)
}
