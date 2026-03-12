use std::fs;

use tempfile::tempdir;
use verifyos_cli::config::{load_file_config, resolve_runtime_config, CliOverrides, FileConfig};

#[test]
fn runtime_config_uses_file_defaults() {
    let file = FileConfig {
        format: Some("json".to_string()),
        profile: Some("basic".to_string()),
        fail_on: Some("warning".to_string()),
        agent_pack: Some("fixes.json".into()),
        timings: Some("summary".to_string()),
        include: Some(vec!["RULE_ATS_AUDIT".to_string()]),
        ..FileConfig::default()
    };

    let runtime = resolve_runtime_config(file, CliOverrides::default());

    assert_eq!(runtime.format, "json");
    assert_eq!(runtime.profile, "basic");
    assert_eq!(runtime.fail_on, "warning");
    assert_eq!(
        runtime.agent_pack.as_deref(),
        Some(std::path::Path::new("fixes.json"))
    );
    assert_eq!(runtime.timings, "summary");
    assert_eq!(runtime.include, vec!["RULE_ATS_AUDIT"]);
}

#[test]
fn runtime_config_prefers_cli_over_file() {
    let file = FileConfig {
        format: Some("json".to_string()),
        profile: Some("basic".to_string()),
        fail_on: Some("warning".to_string()),
        agent_pack: Some("file-fixes.json".into()),
        timings: Some("off".to_string()),
        include: Some(vec!["RULE_ATS_AUDIT".to_string()]),
        ..FileConfig::default()
    };

    let runtime = resolve_runtime_config(
        file,
        CliOverrides {
            format: Some("sarif".to_string()),
            profile: Some("full".to_string()),
            fail_on: Some("off".to_string()),
            agent_pack: Some("cli-fixes.json".into()),
            timings: Some("full".to_string()),
            include: vec!["RULE_PRIVATE_API".to_string()],
            ..CliOverrides::default()
        },
    );

    assert_eq!(runtime.format, "sarif");
    assert_eq!(runtime.profile, "full");
    assert_eq!(runtime.fail_on, "off");
    assert_eq!(
        runtime.agent_pack.as_deref(),
        Some(std::path::Path::new("cli-fixes.json"))
    );
    assert_eq!(runtime.timings, "full");
    assert_eq!(runtime.include, vec!["RULE_PRIVATE_API"]);
}

#[test]
fn load_file_config_reads_verifyos_toml() {
    let dir = tempdir().expect("temp dir");
    let config_path = dir.path().join("verifyos.toml");
    fs::write(
        &config_path,
        r#"
format = "json"
profile = "basic"
fail_on = "warning"
agent_pack = "fixes.json"
timings = "summary"
include = ["RULE_ATS_AUDIT"]
"#,
    )
    .expect("write config");

    let config = load_file_config(Some(&config_path)).expect("load config");

    assert_eq!(config.format.as_deref(), Some("json"));
    assert_eq!(config.profile.as_deref(), Some("basic"));
    assert_eq!(config.fail_on.as_deref(), Some("warning"));
    assert_eq!(
        config.agent_pack.as_deref(),
        Some(std::path::Path::new("fixes.json"))
    );
    assert_eq!(config.timings.as_deref(), Some("summary"));
    assert_eq!(
        config.include.as_deref(),
        Some(&["RULE_ATS_AUDIT".to_string()][..])
    );
}
