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
        agent_pack_format: Some("markdown".to_string()),
        timings: Some("summary".to_string()),
        include: Some(vec!["RULE_ATS_AUDIT".to_string()]),
        init: Some(verifyos_cli::config::InitDefaults {
            output_dir: Some(".verifyos".into()),
            shell_script: Some(true),
            profile: Some("basic".to_string()),
            ..Default::default()
        }),
        doctor: Some(verifyos_cli::config::DoctorDefaults {
            output_dir: Some(".verifyos".into()),
            open_pr_comment: Some(true),
            repair: Some(vec!["pr-comment".to_string()]),
            freshness_against: Some("report.json".into()),
            ..Default::default()
        }),
        ci: Some(verifyos_cli::config::CiDefaults {
            doctor_repair: Some(vec!["agent-bundle".to_string()]),
            comment_mode: Some("plain".to_string()),
        }),
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
    assert_eq!(runtime.agent_pack_format, "markdown");
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
        agent_pack_format: Some("json".to_string()),
        timings: Some("off".to_string()),
        include: Some(vec!["RULE_ATS_AUDIT".to_string()]),
        init: Some(verifyos_cli::config::InitDefaults {
            output_dir: Some(".verifyos".into()),
            ..Default::default()
        }),
        ..FileConfig::default()
    };

    let runtime = resolve_runtime_config(
        file,
        CliOverrides {
            format: Some("sarif".to_string()),
            profile: Some("full".to_string()),
            fail_on: Some("off".to_string()),
            agent_pack: Some("cli-fixes.json".into()),
            agent_pack_format: Some("bundle".to_string()),
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
    assert_eq!(runtime.agent_pack_format, "bundle");
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
agent_pack_format = "markdown"
timings = "summary"
include = ["RULE_ATS_AUDIT"]

[init]
output_dir = ".verifyos"
shell_script = true
profile = "basic"

[doctor]
output_dir = ".verifyos"
open_pr_brief = true
open_pr_comment = true
repair = ["pr-comment"]
freshness_against = "report.json"

[ci]
doctor_repair = ["agent-bundle"]
comment_mode = "plain"
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
    assert_eq!(config.agent_pack_format.as_deref(), Some("markdown"));
    assert_eq!(config.timings.as_deref(), Some("summary"));
    assert_eq!(
        config.include.as_deref(),
        Some(&["RULE_ATS_AUDIT".to_string()][..])
    );
    let init = config.init.expect("init defaults");
    assert_eq!(
        init.output_dir.as_deref(),
        Some(std::path::Path::new(".verifyos"))
    );
    assert_eq!(init.shell_script, Some(true));
    assert_eq!(init.profile.as_deref(), Some("basic"));
    let doctor = config.doctor.expect("doctor defaults");
    assert_eq!(
        doctor.output_dir.as_deref(),
        Some(std::path::Path::new(".verifyos"))
    );
    assert_eq!(doctor.open_pr_brief, Some(true));
    assert_eq!(doctor.open_pr_comment, Some(true));
    assert_eq!(doctor.repair, Some(vec!["pr-comment".to_string()]));
    assert_eq!(
        doctor.freshness_against.as_deref(),
        Some(std::path::Path::new("report.json"))
    );
    let ci = config.ci.expect("ci defaults");
    assert_eq!(ci.doctor_repair, Some(vec!["agent-bundle".to_string()]));
    assert_eq!(ci.comment_mode.as_deref(), Some("plain"));
}
