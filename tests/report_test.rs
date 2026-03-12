use verifyos_cli::report::{
    build_agent_pack, render_json, render_markdown, render_sarif, render_table,
    should_exit_with_failure, top_slow_rules, FailOn, ReportData, ReportItem, SlowRule, TimingMode,
};
use verifyos_cli::rules::core::{
    ArtifactCacheStats, CacheCounter, RuleCategory, RuleStatus, Severity,
};

fn sample_report(items: Vec<ReportItem>) -> ReportData {
    ReportData {
        ruleset_version: "0.1.0".to_string(),
        generated_at_unix: 0,
        total_duration_ms: 42,
        cache_stats: ArtifactCacheStats {
            usage_scan: CacheCounter { hits: 2, misses: 1 },
            bundle_plist: CacheCounter { hits: 1, misses: 1 },
            ..ArtifactCacheStats::default()
        },
        slow_rules: vec![SlowRule {
            rule_id: "RULE_SAMPLE".to_string(),
            rule_name: "Sample Rule".to_string(),
            duration_ms: 7,
        }],
        results: items,
    }
}

fn sample_item(severity: Severity, status: RuleStatus) -> ReportItem {
    ReportItem {
        rule_id: "RULE_SAMPLE".to_string(),
        rule_name: "Sample Rule".to_string(),
        category: RuleCategory::Other,
        severity,
        status,
        message: None,
        evidence: None,
        recommendation: String::new(),
        duration_ms: 7,
    }
}

#[test]
fn fail_on_off_never_fails() {
    let report = sample_report(vec![sample_item(Severity::Error, RuleStatus::Fail)]);
    assert!(!should_exit_with_failure(&report, FailOn::Off));
}

#[test]
fn fail_on_error_only_fails_for_error_findings() {
    let warning_report = sample_report(vec![sample_item(Severity::Warning, RuleStatus::Fail)]);
    let error_report = sample_report(vec![sample_item(Severity::Error, RuleStatus::Fail)]);

    assert!(!should_exit_with_failure(&warning_report, FailOn::Error));
    assert!(should_exit_with_failure(&error_report, FailOn::Error));
}

#[test]
fn fail_on_warning_fails_for_warning_and_error_findings() {
    let warning_report = sample_report(vec![sample_item(Severity::Warning, RuleStatus::Fail)]);
    let error_report = sample_report(vec![sample_item(Severity::Error, RuleStatus::Error)]);

    assert!(should_exit_with_failure(&warning_report, FailOn::Warning));
    assert!(should_exit_with_failure(&error_report, FailOn::Warning));
}

#[test]
fn render_table_omits_timings_by_default() {
    let report = sample_report(vec![sample_item(Severity::Error, RuleStatus::Fail)]);
    let output = render_table(&report, TimingMode::Off);

    assert!(!output.contains("Time"));
    assert!(!output.contains("Total scan time"));
}

#[test]
fn render_table_shows_timings_when_enabled() {
    let report = sample_report(vec![sample_item(Severity::Error, RuleStatus::Fail)]);
    let output = render_table(&report, TimingMode::Full);

    assert!(output.contains("Time"));
    assert!(output.contains("Total scan time: 42 ms"));
    assert!(output.contains("Slowest rules: RULE_SAMPLE (7 ms)"));
    assert!(output.contains("Cache activity: usage_scan h/m=2/1, bundle_plist h/m=1/1"));
}

#[test]
fn render_markdown_shows_timings_when_enabled() {
    let report = sample_report(vec![sample_item(Severity::Warning, RuleStatus::Fail)]);
    let output = render_markdown(&report, Some(1), TimingMode::Full);

    assert!(output.contains("- Total scan time: 42 ms"));
    assert!(output.contains("- Slowest rules:"));
    assert!(output.contains("- Cache activity:"));
    assert!(output.contains("  - usage_scan h/m=2/1"));
    assert!(output.contains("  - Time: 7 ms"));
}

#[test]
fn render_table_summary_mode_hides_per_rule_column() {
    let report = sample_report(vec![sample_item(Severity::Error, RuleStatus::Fail)]);
    let output = render_table(&report, TimingMode::Summary);

    assert!(!output.contains("│ Time"));
    assert!(output.contains("Total scan time: 42 ms"));
    assert!(output.contains("Slowest rules: RULE_SAMPLE (7 ms)"));
}

#[test]
fn render_markdown_summary_mode_hides_per_rule_time_lines() {
    let report = sample_report(vec![sample_item(Severity::Warning, RuleStatus::Fail)]);
    let output = render_markdown(&report, Some(1), TimingMode::Summary);

    assert!(output.contains("- Total scan time: 42 ms"));
    assert!(output.contains("- Cache activity:"));
    assert!(!output.contains("  - Time: 7 ms"));
}

#[test]
fn top_slow_rules_returns_descending_breakdown() {
    let mut slow = sample_item(Severity::Warning, RuleStatus::Pass);
    slow.rule_id = "RULE_SLOW".to_string();
    slow.rule_name = "Slow Rule".to_string();
    slow.duration_ms = 25;

    let mut fast = sample_item(Severity::Info, RuleStatus::Pass);
    fast.rule_id = "RULE_FAST".to_string();
    fast.rule_name = "Fast Rule".to_string();
    fast.duration_ms = 3;

    let mut medium = sample_item(Severity::Error, RuleStatus::Fail);
    medium.rule_id = "RULE_MEDIUM".to_string();
    medium.rule_name = "Medium Rule".to_string();
    medium.duration_ms = 10;

    let report = sample_report(vec![medium, fast, slow]);
    let breakdown = top_slow_rules(&report, 2);

    assert_eq!(breakdown.len(), 2);
    assert_eq!(breakdown[0].rule_id, "RULE_SLOW");
    assert_eq!(breakdown[1].rule_id, "RULE_MEDIUM");
}

#[test]
fn render_json_includes_machine_readable_perf_sections() {
    let report = sample_report(vec![sample_item(Severity::Error, RuleStatus::Fail)]);
    let json = render_json(&report).expect("json render");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse json");

    assert_eq!(value["total_duration_ms"], 42);
    assert_eq!(value["slow_rules"][0]["rule_id"], "RULE_SAMPLE");
    assert_eq!(value["cache_stats"]["usage_scan"]["hits"], 2);
}

#[test]
fn render_sarif_includes_perf_metadata() {
    let report = sample_report(vec![sample_item(Severity::Error, RuleStatus::Fail)]);
    let sarif = render_sarif(&report).expect("sarif render");
    let value: serde_json::Value = serde_json::from_str(&sarif).expect("parse sarif");

    assert_eq!(
        value["runs"][0]["properties"]["totalDurationMs"],
        serde_json::Value::from(42)
    );
    assert_eq!(
        value["runs"][0]["properties"]["slowRules"][0]["ruleId"],
        "RULE_SAMPLE"
    );
    assert_eq!(
        value["runs"][0]["invocations"][0]["properties"]["cacheStats"]["usage_scan"]["hits"],
        serde_json::Value::from(2)
    );
}

#[test]
fn build_agent_pack_extracts_fix_focused_findings() {
    let mut pass_item = sample_item(Severity::Info, RuleStatus::Pass);
    pass_item.rule_id = "RULE_PASS".to_string();

    let mut fail_item = sample_item(Severity::Error, RuleStatus::Fail);
    fail_item.rule_id = "RULE_PRIVATE_API".to_string();
    fail_item.rule_name = "Private API Usage Detected".to_string();
    fail_item.category = RuleCategory::ThirdParty;
    fail_item.message = Some("Private API signatures found".to_string());
    fail_item.evidence = Some("LSApplicationWorkspace".to_string());
    fail_item.recommendation =
        "Remove private API usage or replace with public alternatives.".to_string();

    let report = sample_report(vec![pass_item, fail_item]);
    let pack = build_agent_pack(&report);

    assert_eq!(pack.total_findings, 1);
    assert_eq!(pack.findings[0].rule_id, "RULE_PRIVATE_API");
    assert_eq!(pack.findings[0].priority, "high");
    assert_eq!(pack.findings[0].suggested_fix_scope, "dependencies");
}
