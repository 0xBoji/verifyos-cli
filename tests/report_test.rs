use verifyos_cli::report::{
    render_markdown, render_table, should_exit_with_failure, FailOn, ReportData, ReportItem,
};
use verifyos_cli::rules::core::{RuleCategory, RuleStatus, Severity};

fn sample_report(items: Vec<ReportItem>) -> ReportData {
    ReportData {
        ruleset_version: "0.1.0".to_string(),
        generated_at_unix: 0,
        total_duration_ms: 42,
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
    let output = render_table(&report, false);

    assert!(!output.contains("Time"));
    assert!(!output.contains("Total scan time"));
}

#[test]
fn render_table_shows_timings_when_enabled() {
    let report = sample_report(vec![sample_item(Severity::Error, RuleStatus::Fail)]);
    let output = render_table(&report, true);

    assert!(output.contains("Time"));
    assert!(output.contains("Total scan time: 42 ms"));
}

#[test]
fn render_markdown_shows_timings_when_enabled() {
    let report = sample_report(vec![sample_item(Severity::Warning, RuleStatus::Fail)]);
    let output = render_markdown(&report, Some(1), true);

    assert!(output.contains("- Total scan time: 42 ms"));
    assert!(output.contains("  - Time: 7 ms"));
}
