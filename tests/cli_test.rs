use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;
use verifyos_cli::core::engine::Engine;
use verifyos_cli::profiles::{register_rules, RuleSelection, ScanProfile};
use verifyos_cli::report::build_report;
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
    assert!(value["findings"][0].get("target_files").is_some());
    assert!(value["findings"][0].get("patch_hint").is_some());
    assert!(value["findings"][0].get("why_it_fails_review").is_some());
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

#[test]
fn test_init_creates_agents_file() {
    let dir = tempdir().expect("temp dir");
    let agents_path = dir.path().join("AGENTS.md");

    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .args([
            "init",
            "--path",
            agents_path.to_str().expect("utf8 agents path"),
        ])
        .output()
        .expect("init should run");

    assert!(output.status.success());

    let contents = std::fs::read_to_string(&agents_path).expect("agents file should exist");
    assert!(contents.contains("## verifyOS-cli"));
    assert!(contents.contains("RULE_PRIVACY_MANIFEST"));
    assert!(contents.contains("<!-- verifyos-cli:agents:start -->"));
}

#[test]
fn test_init_updates_existing_agents_file_without_removing_custom_content() {
    let dir = tempdir().expect("temp dir");
    let agents_path = dir.path().join("AGENTS.md");
    std::fs::write(
        &agents_path,
        "# AGENTS.md\n\nMy custom note\n\n<!-- verifyos-cli:agents:start -->\nold\n<!-- verifyos-cli:agents:end -->\n\nKeep me\n",
    )
    .expect("write existing AGENTS.md");

    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .args([
            "init",
            "--path",
            agents_path.to_str().expect("utf8 agents path"),
        ])
        .output()
        .expect("init update should run");

    assert!(output.status.success());

    let contents = std::fs::read_to_string(&agents_path).expect("agents file should exist");
    assert!(contents.contains("My custom note"));
    assert!(contents.contains("Keep me"));
    assert!(!contents.contains("\nold\n"));
    assert_eq!(
        contents
            .matches("<!-- verifyos-cli:agents:start -->")
            .count(),
        1
    );
}

#[test]
fn test_init_from_scan_injects_current_project_risks() {
    let dir = tempdir().expect("temp dir");
    let agents_path = dir.path().join("AGENTS.md");

    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .args([
            "init",
            "--path",
            agents_path.to_str().expect("utf8 agents path"),
            "--from-scan",
            get_example_path("bad_app.ipa")
                .to_str()
                .expect("utf8 app path"),
        ])
        .output()
        .expect("init from scan should run");

    assert!(output.status.success());

    let contents = std::fs::read_to_string(&agents_path).expect("agents file should exist");
    assert!(contents.contains("### Current Project Risks"));
    assert!(contents.contains("#### Suggested Patch Order"));
    assert!(contents.contains("Missing Privacy Manifest"));
}

#[test]
fn test_init_from_scan_with_baseline_keeps_only_new_risks() {
    let dir = tempdir().expect("temp dir");
    let agents_path = dir.path().join("AGENTS.md");
    let baseline_path = dir.path().join("baseline.json");

    let engine = create_engine();
    let run = engine
        .run(get_example_path("bad_app.ipa"))
        .expect("engine should scan bad app");
    let report = build_report(run.results, run.total_duration_ms, run.cache_stats);
    std::fs::write(
        &baseline_path,
        serde_json::to_string_pretty(&report).expect("baseline json"),
    )
    .expect("write baseline");

    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .args([
            "init",
            "--path",
            agents_path.to_str().expect("utf8 agents path"),
            "--from-scan",
            get_example_path("bad_app.ipa")
                .to_str()
                .expect("utf8 app path"),
            "--baseline",
            baseline_path.to_str().expect("utf8 baseline path"),
        ])
        .output()
        .expect("init from scan with baseline should run");

    assert!(output.status.success());

    let contents = std::fs::read_to_string(&agents_path).expect("agents file should exist");
    assert!(contents.contains("### Current Project Risks"));
    assert!(contents.contains("No new or regressed risks"));
    assert!(!contents.contains("| `high` | `RULE_PRIVACY_MANIFEST` |"));
    assert!(!contents.contains("- **Missing Privacy Manifest** (`RULE_PRIVACY_MANIFEST`)"));
}

#[test]
fn test_init_from_scan_with_agent_pack_dir_writes_bundle_and_links_it() {
    let dir = tempdir().expect("temp dir");
    let agents_path = dir.path().join("AGENTS.md");
    let pack_dir = dir.path().join(".verifyos-agent");

    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .args([
            "init",
            "--path",
            agents_path.to_str().expect("utf8 agents path"),
            "--from-scan",
            get_example_path("bad_app.ipa")
                .to_str()
                .expect("utf8 app path"),
            "--agent-pack-dir",
            pack_dir.to_str().expect("utf8 agent pack dir"),
        ])
        .output()
        .expect("init from scan with pack dir should run");

    assert!(output.status.success());
    assert!(pack_dir.join("agent-pack.json").exists());
    assert!(pack_dir.join("agent-pack.md").exists());

    let contents = std::fs::read_to_string(&agents_path).expect("agents file should exist");
    assert!(contents.contains("Agent bundle:"));
    assert!(contents.contains(&format!("{}/agent-pack.md", pack_dir.display())));
}

#[test]
fn test_init_write_commands_injects_follow_up_commands() {
    let dir = tempdir().expect("temp dir");
    let agents_path = dir.path().join("AGENTS.md");
    let pack_dir = dir.path().join(".verifyos-agent");

    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .args([
            "init",
            "--path",
            agents_path.to_str().expect("utf8 agents path"),
            "--from-scan",
            get_example_path("bad_app.ipa")
                .to_str()
                .expect("utf8 app path"),
            "--agent-pack-dir",
            pack_dir.to_str().expect("utf8 agent pack dir"),
            "--profile",
            "basic",
            "--write-commands",
        ])
        .output()
        .expect("init write commands should run");

    assert!(output.status.success());

    let contents = std::fs::read_to_string(&agents_path).expect("agents file should exist");
    assert!(contents.contains("### Next Commands"));
    assert!(contents.contains("voc --app"));
    assert!(contents.contains("--profile basic"));
    assert!(contents.contains("--write-commands"));
    assert!(contents.contains(&pack_dir.display().to_string()));
    assert!(contents.contains("agent-pack-format bundle"));
}

#[test]
fn test_init_shell_script_writes_next_steps_and_mentions_it() {
    let dir = tempdir().expect("temp dir");
    let agents_path = dir.path().join("AGENTS.md");
    let pack_dir = dir.path().join(".verifyos-agent");

    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .args([
            "init",
            "--path",
            agents_path.to_str().expect("utf8 agents path"),
            "--from-scan",
            get_example_path("bad_app.ipa")
                .to_str()
                .expect("utf8 app path"),
            "--agent-pack-dir",
            pack_dir.to_str().expect("utf8 agent pack dir"),
            "--shell-script",
        ])
        .output()
        .expect("init shell script should run");

    assert!(output.status.success());

    let script_path = pack_dir.join("next-steps.sh");
    assert!(script_path.exists());
    let script = std::fs::read_to_string(&script_path).expect("script should exist");
    assert!(script.contains("#!/usr/bin/env bash"));
    assert!(script.contains("voc --app"));
    assert!(script.contains("--agent-pack-format bundle"));
    assert!(script.contains("--shell-script"));

    let contents = std::fs::read_to_string(&agents_path).expect("agents file should exist");
    assert!(contents.contains("### Next Commands"));
    assert!(contents.contains("next-steps.sh"));
}

#[test]
fn test_init_shell_script_without_agent_pack_dir_creates_default_bundle() {
    let dir = tempdir().expect("temp dir");
    let agents_path = dir.path().join("AGENTS.md");

    let output = Command::new(env!("CARGO_BIN_EXE_voc"))
        .current_dir(dir.path())
        .args([
            "init",
            "--path",
            agents_path.to_str().expect("utf8 agents path"),
            "--from-scan",
            get_example_path("bad_app.ipa")
                .to_str()
                .expect("utf8 app path"),
            "--shell-script",
        ])
        .output()
        .expect("init shell script without dir should run");

    assert!(output.status.success());

    let default_dir = dir.path().join(".verifyos-agent");
    assert!(default_dir.join("agent-pack.json").exists());
    assert!(default_dir.join("agent-pack.md").exists());
    assert!(default_dir.join("next-steps.sh").exists());

    let contents = std::fs::read_to_string(&agents_path).expect("agents file should exist");
    assert!(contents.contains(".verifyos-agent/agent-pack.md"));
    assert!(contents.contains(".verifyos-agent/next-steps.sh"));
}
