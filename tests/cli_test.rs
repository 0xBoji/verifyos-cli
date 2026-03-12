use std::path::PathBuf;
use verifyos_cli::core::engine::Engine;
use verifyos_cli::profiles::{register_rules, RuleSelection, ScanProfile};
use verifyos_cli::rules::core::{RuleStatus, Severity};

fn get_example_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(filename)
}

fn create_engine() -> Engine {
    let mut engine = Engine::new();
    register_rules(&mut engine, ScanProfile::Full, &RuleSelection::default());
    engine
}

#[test]
fn test_bad_app_fails_rules() {
    let bad_app = get_example_path("bad_app.ipa");
    let engine = create_engine();

    let run = engine.run(&bad_app).expect("Engine orchestrator failed");
    let results = run.results;

    let mut has_errors = false;
    for res in results {
        if let Severity::Error = res.severity {
            if matches!(res.report, Ok(ref report) if report.status == RuleStatus::Fail)
                || res.report.is_err()
            {
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

    let run = engine.run(&good_app).expect("Engine orchestrator failed");
    let results = run.results;

    let mut has_errors = false;
    for res in results {
        if let Severity::Error = res.severity {
            if matches!(res.report, Ok(ref report) if report.status == RuleStatus::Fail)
                || res.report.is_err()
            {
                has_errors = true;
                if let Err(err) = res.report {
                    println!("Unexpected error in good_app: {:?}", err);
                }
            }
        }
    }

    assert!(
        !has_errors,
        "Expected good_app.ipa to pass all rules, but it triggered an error."
    );
}
