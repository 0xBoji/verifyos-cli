use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use verifyos_cli::parsers::plist_reader::InfoPlist;
use verifyos_cli::rules::core::{AppStoreRule, ArtifactContext, RuleStatus};
use verifyos_cli::rules::info_plist::LSApplicationQueriesSchemesAuditRule;
use verifyos_cli::rules::info_plist::UIRequiredDeviceCapabilitiesAuditRule;
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

#[test]
fn test_lsapplicationqueries_schemes_passes() {
    let mut dict = plist::Dictionary::new();
    dict.insert(
        "LSApplicationQueriesSchemes".to_string(),
        plist::Value::Array(vec![
            plist::Value::String("fb".to_string()),
            plist::Value::String("twitter".to_string()),
        ]),
    );

    let plist = InfoPlist::from_dictionary(dict);
    let app_path = PathBuf::from("does_not_exist.app");
    let context = ArtifactContext {
        app_bundle_path: &app_path,
        info_plist: Some(&plist),
    };

    let rule = LSApplicationQueriesSchemesAuditRule;
    let result = rule.evaluate(&context).expect("Rule should evaluate");
    assert_eq!(result.status, RuleStatus::Pass);
}

#[test]
fn test_lsapplicationqueries_schemes_fails_on_duplicates() {
    let mut dict = plist::Dictionary::new();
    dict.insert(
        "LSApplicationQueriesSchemes".to_string(),
        plist::Value::Array(vec![
            plist::Value::String("fb".to_string()),
            plist::Value::String("fb".to_string()),
            plist::Value::String("prefs".to_string()),
        ]),
    );

    let plist = InfoPlist::from_dictionary(dict);
    let app_path = PathBuf::from("does_not_exist.app");
    let context = ArtifactContext {
        app_bundle_path: &app_path,
        info_plist: Some(&plist),
    };

    let rule = LSApplicationQueriesSchemesAuditRule;
    let result = rule.evaluate(&context).expect("Rule should evaluate");
    assert_eq!(result.status, RuleStatus::Fail);
}

#[test]
fn test_device_capabilities_audit_fails_on_missing_usage() {
    let dir = tempdir().expect("temp dir");
    let app_dir = dir.path().join("TestApp.app");
    fs::create_dir_all(&app_dir).expect("create app dir");

    let executable_path = app_dir.join("TestApp");
    fs::write(&executable_path, b"no usage signatures").expect("write executable");

    let mut dict = plist::Dictionary::new();
    dict.insert(
        "UIRequiredDeviceCapabilities".to_string(),
        plist::Value::Array(vec![plist::Value::String("camera".to_string())]),
    );
    let plist = InfoPlist::from_dictionary(dict);

    let context = ArtifactContext {
        app_bundle_path: &app_dir,
        info_plist: Some(&plist),
    };

    let rule = UIRequiredDeviceCapabilitiesAuditRule;
    let result = rule.evaluate(&context).expect("Rule should evaluate");
    assert_eq!(result.status, RuleStatus::Fail);
}

#[test]
fn test_device_capabilities_audit_passes_on_usage() {
    let dir = tempdir().expect("temp dir");
    let app_dir = dir.path().join("TestApp.app");
    fs::create_dir_all(&app_dir).expect("create app dir");

    let executable_path = app_dir.join("TestApp");
    fs::write(&executable_path, b"AVCaptureDevice").expect("write executable");

    let mut dict = plist::Dictionary::new();
    dict.insert(
        "UIRequiredDeviceCapabilities".to_string(),
        plist::Value::Array(vec![plist::Value::String("camera".to_string())]),
    );
    let plist = InfoPlist::from_dictionary(dict);

    let context = ArtifactContext {
        app_bundle_path: &app_dir,
        info_plist: Some(&plist),
    };

    let rule = UIRequiredDeviceCapabilitiesAuditRule;
    let result = rule.evaluate(&context).expect("Rule should evaluate");
    assert_eq!(result.status, RuleStatus::Pass);
}
