use crate::app::AppState;
use crate::infra::http::handlers::{handoff_bundle, health, scan_bundle};
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::permissive();

    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/scan", post(scan_bundle))
        .route("/api/v1/handoff", post(handoff_bundle))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
