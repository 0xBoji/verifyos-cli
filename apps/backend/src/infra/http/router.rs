use crate::app::AppState;
use crate::infra::http::handlers::{
    handle_google_callback, health, handoff_bundle, scan_bundle, start_auth, start_google_auth,
    verify_auth,
};
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::permissive();

    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/auth/start", post(start_auth))
        .route("/api/v1/auth/verify", post(verify_auth))
        .route("/api/v1/auth/google", get(start_google_auth))
        .route("/api/v1/auth/google/callback", get(handle_google_callback))
        .route("/api/v1/scan", post(scan_bundle))
        .route("/api/v1/handoff", post(handoff_bundle))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
