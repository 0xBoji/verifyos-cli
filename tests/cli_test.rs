use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;
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

#[test]
fn test_help_shows_verify_os_banner() {
    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .arg("--help")
        .output()
        .expect("help should run");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("verify-OS"));
    assert!(stdout.contains("████"));
}

#[test]
fn test_list_rules_table_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .arg("--list-rules")
        .output()
        .expect("list-rules should run");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("Rule ID"));
    assert!(stdout.contains("RULE_PRIVACY_MANIFEST"));
    assert!(stdout.contains("basic, full"));
}

#[test]
fn test_list_rules_json_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .args(["--list-rules", "--format", "json"])
        .output()
        .expect("list-rules json should run");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    let first = value
        .as_array()
        .and_then(|items| items.first())
        .expect("items");
    assert!(first.get("rule_id").is_some());
    assert!(first.get("name").is_some());
    assert!(first.get("severity").is_some());
    assert!(first.get("category").is_some());
    assert!(first.get("default_profiles").is_some());
}

#[test]
fn test_show_rule_table_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .args(["--show-rule", "RULE_PRIVATE_API"])
        .output()
        .expect("show-rule should run");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("Rule ID"));
    assert!(stdout.contains("RULE_PRIVATE_API"));
    assert!(stdout.contains("Recommendation"));
}

#[test]
fn test_show_rule_json_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .args(["--show-rule", "RULE_PRIVATE_API", "--format", "json"])
        .output()
        .expect("show-rule json should run");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert_eq!(value["rule_id"], "RULE_PRIVATE_API");
    assert!(value.get("recommendation").is_some());
    assert!(value.get("default_profiles").is_some());
}

#[test]
fn test_agent_pack_writes_fix_json() {
    let dir = tempdir().expect("temp dir");
    let output_path = dir.path().join("fixes.json");

    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .args([
            "--app",
            get_example_path("bad_app.ipa").to_str().expect("utf8 path"),
            "--agent-pack",
            output_path.to_str().expect("utf8 output path"),
        ])
        .output()
        .expect("agent-pack run should succeed");

    assert!(
        !output.status.success(),
        "bad_app should still fail exit threshold"
    );

    let contents = std::fs::read_to_string(&output_path).expect("agent pack should be written");
    let value: serde_json::Value = serde_json::from_str(&contents).expect("valid agent pack");
    assert!(value["total_findings"].as_u64().unwrap_or_default() >= 1);
    assert!(value["findings"].as_array().is_some());
    assert!(value["findings"][0].get("rule_id").is_some());
    assert!(value["findings"][0].get("suggested_fix_scope").is_some());
}

#[test]
fn test_agent_pack_writes_markdown() {
    let dir = tempdir().expect("temp dir");
    let output_path = dir.path().join("fixes.md");

    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .args([
            "--app",
            get_example_path("bad_app.ipa").to_str().expect("utf8 path"),
            "--agent-pack",
            output_path.to_str().expect("utf8 output path"),
            "--agent-pack-format",
            "markdown",
        ])
        .output()
        .expect("agent-pack markdown run should succeed");

    assert!(
        !output.status.success(),
        "bad_app should still fail exit threshold"
    );

    let contents = std::fs::read_to_string(&output_path).expect("agent markdown pack exists");
    assert!(contents.contains("# verifyOS Agent Fix Pack"));
    assert!(contents.contains("## Findings by Fix Scope"));
}

#[test]
fn test_agent_pack_bundle_writes_json_and_markdown() {
    let dir = tempdir().expect("temp dir");
    let output_dir = dir.path().join("agent-pack");

    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .args([
            "--app",
            get_example_path("bad_app.ipa").to_str().expect("utf8 path"),
            "--agent-pack",
            output_dir.to_str().expect("utf8 output dir"),
            "--agent-pack-format",
            "bundle",
        ])
        .output()
        .expect("agent-pack bundle run should succeed");

    assert!(
        !output.status.success(),
        "bad_app should still fail exit threshold"
    );

    assert!(output_dir.join("agent-pack.json").exists());
    assert!(output_dir.join("agent-pack.md").exists());
}
