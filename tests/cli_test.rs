use std::path::PathBuf;
use verifyos_cli::core::engine::Engine;
use verifyos_cli::rules::ats::AtsExceptionsGranularityRule;
use verifyos_cli::rules::bundle_leakage::BundleResourceLeakageRule;
use verifyos_cli::rules::core::{RuleStatus, Severity};
use verifyos_cli::rules::entitlements::EntitlementsMismatchRule;
use verifyos_cli::rules::extensions::ExtensionEntitlementsCompatibilityRule;
use verifyos_cli::rules::info_plist::InfoPlistVersionConsistencyRule;
use verifyos_cli::rules::info_plist::LSApplicationQueriesSchemesAuditRule;
use verifyos_cli::rules::info_plist::UIRequiredDeviceCapabilitiesAuditRule;
use verifyos_cli::rules::permissions::CameraUsageDescriptionRule;
use verifyos_cli::rules::privacy::MissingPrivacyManifestRule;
use verifyos_cli::rules::privacy_sdk::PrivacyManifestSdkCrossCheckRule;
use verifyos_cli::rules::signing::EmbeddedCodeSignatureTeamRule;

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
    engine.register_rule(Box::new(EmbeddedCodeSignatureTeamRule));
    engine.register_rule(Box::new(LSApplicationQueriesSchemesAuditRule));
    engine.register_rule(Box::new(UIRequiredDeviceCapabilitiesAuditRule));
    engine.register_rule(Box::new(AtsExceptionsGranularityRule));
    engine.register_rule(Box::new(BundleResourceLeakageRule));
    engine.register_rule(Box::new(InfoPlistVersionConsistencyRule));
    engine.register_rule(Box::new(ExtensionEntitlementsCompatibilityRule));
    engine.register_rule(Box::new(PrivacyManifestSdkCrossCheckRule));
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

    let results = engine.run(&good_app).expect("Engine orchestrator failed");

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
