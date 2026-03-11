use std::path::PathBuf;
use verifyos_cli::parsers::plist_reader::InfoPlist;
use verifyos_cli::rules::core::{AppStoreRule, ArtifactContext, RuleStatus};
use verifyos_cli::rules::permissions::CameraUsageDescriptionRule;
use verifyos_cli::rules::privacy::MissingPrivacyManifestRule;
use verifyos_cli::rules::signing::EmbeddedCodeSignatureTeamRule;

fn get_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("TestApp.app")
}

#[test]
fn test_privacy_manifest_rule_passes() {
    let app_path = get_fixture_path();
    let context = ArtifactContext {
        app_bundle_path: &app_path,
        info_plist: None,
    };

    let rule = MissingPrivacyManifestRule;
    let result = rule.evaluate(&context).expect("Rule should evaluate");
    assert_eq!(result.status, RuleStatus::Pass);
}

#[test]
fn test_privacy_manifest_rule_fails() {
    let app_path = PathBuf::from("does_not_exist.app");
    let context = ArtifactContext {
        app_bundle_path: &app_path,
        info_plist: None,
    };

    let rule = MissingPrivacyManifestRule;
    let result = rule.evaluate(&context).expect("Rule should evaluate");
    assert_eq!(result.status, RuleStatus::Fail);
}

#[test]
fn test_camera_usage_rule_passes() {
    let app_path = get_fixture_path();
    let plist_path = app_path.join("Info.plist");
    let plist = InfoPlist::from_file(&plist_path).unwrap();

    let context = ArtifactContext {
        app_bundle_path: &app_path,
        info_plist: Some(&plist),
    };

    let rule = CameraUsageDescriptionRule;
    let result = rule.evaluate(&context).expect("Rule should evaluate");
    assert_eq!(result.status, RuleStatus::Pass);
}

#[test]
fn test_embedded_team_rule_skips_without_executable() {
    let app_path = get_fixture_path();
    let plist_path = app_path.join("Info.plist");
    let plist = InfoPlist::from_file(&plist_path).unwrap();

    let context = ArtifactContext {
        app_bundle_path: &app_path,
        info_plist: Some(&plist),
    };

    let rule = EmbeddedCodeSignatureTeamRule;
    let result = rule.evaluate(&context).expect("Rule should evaluate");
    assert_eq!(result.status, RuleStatus::Skip);
}
