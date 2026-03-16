use verifyos_backend::app::{AppState, AuthStore, ScanService};
use verifyos_backend::infra::http::router::build_router;

#[tokio::main]
async fn main() {
    verifyos_backend::infra::telemetry::init_tracing();
    let service = ScanService::new();
    let auth = AuthStore::new();
    let require_auth = std::env::var("REQUIRE_AUTH")
        .ok()
        .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));
    let app = build_router(AppState::new(service, auth, require_auth));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:7070")
        .await
        .expect("bind backend listener");
    axum::serve(listener, app).await.expect("serve backend");
}
