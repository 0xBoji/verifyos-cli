use std::path::PathBuf;
use verifyos_parsers::plist_reader::InfoPlist;
use verifyos_rules::core::{AppStoreRule, ArtifactContext};
use verifyos_rules::permissions::CameraUsageDescriptionRule;
use verifyos_rules::privacy::MissingPrivacyManifestRule;

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
    let result = rule.evaluate(&context);
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

#[test]
fn test_privacy_manifest_rule_fails() {
    let app_path = PathBuf::from("does_not_exist.app");
    let context = ArtifactContext {
        app_bundle_path: &app_path,
        info_plist: None,
    };
    
    let rule = MissingPrivacyManifestRule;
    let result = rule.evaluate(&context);
    assert!(result.is_err());
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
    let result = rule.evaluate(&context);
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}
