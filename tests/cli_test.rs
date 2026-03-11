use std::path::PathBuf;
use verifyos_cli::core::engine::Engine;
use verifyos_cli::rules::core::Severity;
use verifyos_cli::rules::entitlements::EntitlementsMismatchRule;
use verifyos_cli::rules::permissions::CameraUsageDescriptionRule;
use verifyos_cli::rules::privacy::MissingPrivacyManifestRule;

fn get_example_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(filename)
}

fn create_engine() -> Engine {
    let mut engine = Engine::new();
    engine.register_rule(Box::new(MissingPrivacyManifestRule));
    engine.register_rule(Box::new(CameraUsageDescriptionRule));
    engine.register_rule(Box::new(EntitlementsMismatchRule));
    engine
}

#[test]
fn test_bad_app_fails_rules() {
    let bad_app = get_example_path("bad_app.ipa");
    let engine = create_engine();

    let results = engine.run(&bad_app).expect("Engine orchestrator failed");

    let mut has_errors = false;
    for res in results {
        if let Severity::Error = res.severity {
            if res.result.is_err() {
                has_errors = true;
            }
        }
    }

    assert!(
        has_errors,
        "Expected bad_app.ipa to trigger rule errors, but it passed cleanly."
    );
}

#[test]
fn test_good_app_passes_rules() {
    let good_app = get_example_path("good_app.ipa");
    let engine = create_engine();

    let results = engine.run(&good_app).expect("Engine orchestrator failed");

    let mut has_errors = false;
    for res in results {
        if let Severity::Error = res.severity {
            if res.result.is_err() {
                has_errors = true;
                println!("Unexpected error in good_app: {:?}", res.result.err());
            }
        }
    }

    assert!(
        !has_errors,
        "Expected good_app.ipa to pass all rules, but it triggered an error."
    );
}
