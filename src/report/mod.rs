use crate::core::engine::EngineResult;
use crate::rules::core::{
    ArtifactCacheStats, CacheCounter, RuleCategory, RuleStatus, Severity, RULESET_VERSION,
};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use textwrap::wrap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportData {
    pub ruleset_version: String,
    pub generated_at_unix: u64,
    pub total_duration_ms: u128,
    pub cache_stats: ArtifactCacheStats,
    pub results: Vec<ReportItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportItem {
    pub rule_id: String,
    pub rule_name: String,
    pub category: RuleCategory,
    pub severity: Severity,
    pub status: RuleStatus,
    pub message: Option<String>,
    pub evidence: Option<String>,
    pub recommendation: String,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlowRule {
    pub rule_id: String,
    pub rule_name: String,
    pub duration_ms: u128,
}

#[derive(Debug, Clone)]
pub struct BaselineSummary {
    pub suppressed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailOn {
    Off,
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingMode {
    Off,
    Summary,
    Full,
}

pub fn build_report(
    results: Vec<EngineResult>,
    total_duration_ms: u128,
    cache_stats: ArtifactCacheStats,
) -> ReportData {
    let generated_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut items = Vec::new();

    for res in results {
        let (status, message, evidence) = match res.report {
            Ok(report) => (report.status, report.message, report.evidence),
            Err(err) => (
                RuleStatus::Error,
                Some(err.to_string()),
                Some("Rule evaluation error".to_string()),
            ),
        };

        items.push(ReportItem {
            rule_id: res.rule_id.to_string(),
            rule_name: res.rule_name.to_string(),
            category: res.category,
            severity: res.severity,
            status,
            message,
            evidence,
            recommendation: res.recommendation.to_string(),
            duration_ms: res.duration_ms,
        });
    }

    ReportData {
        ruleset_version: RULESET_VERSION.to_string(),
        generated_at_unix,
        total_duration_ms,
        cache_stats,
        results: items,
    }
}

pub fn apply_baseline(report: &mut ReportData, baseline: &ReportData) -> BaselineSummary {
    let mut suppressed = 0;
    let baseline_keys: HashSet<String> = baseline
        .results
        .iter()
        .filter(|r| r.status == RuleStatus::Fail)
        .map(finding_key)
        .collect();

    report.results.retain(|r| {
        if r.status != RuleStatus::Fail {
            return true;
        }
        let key = finding_key(r);
        let keep = !baseline_keys.contains(&key);
        if !keep {
            suppressed += 1;
        }
        keep
    });

    BaselineSummary { suppressed }
}

fn finding_key(item: &ReportItem) -> String {
    format!(
        "{}|{}",
        item.rule_id,
        item.evidence.clone().unwrap_or_default()
    )
}

pub fn should_exit_with_failure(report: &ReportData, fail_on: FailOn) -> bool {
    match fail_on {
        FailOn::Off => false,
        FailOn::Error => report.results.iter().any(|item| {
            matches!(item.status, RuleStatus::Fail | RuleStatus::Error)
                && matches!(item.severity, Severity::Error)
        }),
        FailOn::Warning => report.results.iter().any(|item| {
            matches!(item.status, RuleStatus::Fail | RuleStatus::Error)
                && matches!(item.severity, Severity::Error | Severity::Warning)
        }),
    }
}

pub fn top_slow_rules(report: &ReportData, limit: usize) -> Vec<SlowRule> {
    let mut items: Vec<SlowRule> = report
        .results
        .iter()
        .map(|item| SlowRule {
            rule_id: item.rule_id.clone(),
            rule_name: item.rule_name.clone(),
            duration_ms: item.duration_ms,
        })
        .collect();
    items.sort_by(|a, b| {
        b.duration_ms
            .cmp(&a.duration_ms)
            .then_with(|| a.rule_id.cmp(&b.rule_id))
    });
    items.truncate(limit);
    items
}

pub fn render_table(report: &ReportData, timing_mode: TimingMode) -> String {
    let mut table = Table::new();
    let mut header = vec!["Rule", "Category", "Severity", "Status", "Message"];
    if timing_mode == TimingMode::Full {
        header.push("Time");
    }
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(header);

    for res in &report.results {
        let severity_cell = match res.severity {
            Severity::Error => Cell::new("ERROR").fg(Color::Red),
            Severity::Warning => Cell::new("WARNING").fg(Color::Yellow),
            Severity::Info => Cell::new("INFO").fg(Color::Blue),
        };

        let status_cell = match res.status {
            RuleStatus::Pass => Cell::new("PASS").fg(Color::Green),
            RuleStatus::Fail => Cell::new("FAIL").fg(Color::Red),
            RuleStatus::Error => Cell::new("ERROR").fg(Color::Red),
            RuleStatus::Skip => Cell::new("SKIP").fg(Color::Yellow),
        };

        let message = res.message.clone().unwrap_or_else(|| "PASS".to_string());
        let wrapped = wrap(&message, 50).join("\n");

        let mut row = vec![
            Cell::new(res.rule_name.clone()),
            Cell::new(format!("{:?}", res.category)),
            severity_cell,
            status_cell,
            Cell::new(wrapped),
        ];
        if timing_mode == TimingMode::Full {
            row.push(Cell::new(format!("{} ms", res.duration_ms)));
        }
        table.add_row(row);
    }

    if timing_mode != TimingMode::Off {
        let slow_rules = format_slow_rules(top_slow_rules(report, 3));
        let cache_summary = format_cache_stats(&report.cache_stats);
        format!(
            "{}\nTotal scan time: {} ms{}{}\n",
            table, report.total_duration_ms, slow_rules, cache_summary
        )
    } else {
        format!("{}", table)
    }
}

pub fn render_json(report: &ReportData) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

pub fn render_sarif(report: &ReportData) -> Result<String, serde_json::Error> {
    let mut rules = Vec::new();
    let mut results = Vec::new();

    for item in &report.results {
        rules.push(serde_json::json!({
        "id": item.rule_id,
        "name": item.rule_name,
        "shortDescription": { "text": item.rule_name },
        "fullDescription": { "text": item.message.clone().unwrap_or_default() },
        "help": { "text": item.recommendation },
            "properties": {
                "category": format!("{:?}", item.category),
                "severity": format!("{:?}", item.severity),
                "durationMs": item.duration_ms,
            }
        }));

        if item.status == RuleStatus::Fail || item.status == RuleStatus::Error {
            results.push(serde_json::json!({
                "ruleId": item.rule_id,
                "level": sarif_level(item.severity),
                "message": {
                    "text": item.message.clone().unwrap_or_else(|| item.rule_name.clone())
                },
                "properties": {
                    "category": format!("{:?}", item.category),
                    "evidence": item.evidence.clone().unwrap_or_default(),
                    "durationMs": item.duration_ms,
                }
            }));
        }
    }

    let sarif = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "runs": [
            {
                "tool": {
                    "driver": {
                        "name": "verifyos-cli",
                        "semanticVersion": report.ruleset_version,
                        "rules": rules
                    }
                },
                "results": results
            }
        ]
    });

    serde_json::to_string_pretty(&sarif)
}

fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "note",
    }
}

pub fn render_markdown(
    report: &ReportData,
    suppressed: Option<usize>,
    timing_mode: TimingMode,
) -> String {
    let total = report.results.len();
    let fail_count = report
        .results
        .iter()
        .filter(|r| matches!(r.status, RuleStatus::Fail | RuleStatus::Error))
        .count();
    let warn_count = report
        .results
        .iter()
        .filter(|r| r.severity == Severity::Warning)
        .count();
    let error_count = report
        .results
        .iter()
        .filter(|r| r.severity == Severity::Error)
        .count();

    let mut out = String::new();
    out.push_str("# verifyOS-cli Report\n\n");
    out.push_str(&format!("- Total rules: {total}\n"));
    out.push_str(&format!("- Failures: {fail_count}\n"));
    out.push_str(&format!(
        "- Severity: error={error_count}, warning={warn_count}\n"
    ));
    if timing_mode != TimingMode::Off {
        out.push_str(&format!(
            "- Total scan time: {} ms\n",
            report.total_duration_ms
        ));
        let slow_rules = top_slow_rules(report, 3);
        if !slow_rules.is_empty() {
            out.push_str("- Slowest rules:\n");
            for item in &slow_rules {
                out.push_str(&format!(
                    "  - {} (`{}`): {} ms\n",
                    item.rule_name, item.rule_id, item.duration_ms
                ));
            }
        }
        let cache_lines = markdown_cache_stats(&report.cache_stats);
        if !cache_lines.is_empty() {
            out.push_str("- Cache activity:\n");
            for line in cache_lines {
                out.push_str(&format!("  - {}\n", line));
            }
        }
    }
    if let Some(suppressed) = suppressed {
        out.push_str(&format!("- Baseline suppressed: {suppressed}\n"));
    }
    out.push('\n');

    let mut failures = report
        .results
        .iter()
        .filter(|r| matches!(r.status, RuleStatus::Fail | RuleStatus::Error));

    if failures.next().is_none() {
        out.push_str("## Findings\n\n- No failing findings.\n");
        return out;
    }

    out.push_str("## Findings\n\n");
    for item in report
        .results
        .iter()
        .filter(|r| matches!(r.status, RuleStatus::Fail | RuleStatus::Error))
    {
        out.push_str(&format!("- **{}** (`{}`)\n", item.rule_name, item.rule_id));
        out.push_str(&format!("  - Category: `{:?}`\n", item.category));
        out.push_str(&format!("  - Severity: `{:?}`\n", item.severity));
        out.push_str(&format!("  - Status: `{:?}`\n", item.status));
        if let Some(message) = &item.message {
            out.push_str(&format!("  - Message: {}\n", message));
        }
        if let Some(evidence) = &item.evidence {
            out.push_str(&format!("  - Evidence: {}\n", evidence));
        }
        if !item.recommendation.is_empty() {
            out.push_str(&format!("  - Recommendation: {}\n", item.recommendation));
        }
        if timing_mode == TimingMode::Full {
            out.push_str(&format!("  - Time: {} ms\n", item.duration_ms));
        }
    }

    out
}

fn format_slow_rules(items: Vec<SlowRule>) -> String {
    if items.is_empty() {
        return String::new();
    }

    let parts: Vec<String> = items
        .into_iter()
        .map(|item| format!("{} ({} ms)", item.rule_id, item.duration_ms))
        .collect();
    format!("\nSlowest rules: {}", parts.join(", "))
}

fn format_cache_stats(stats: &ArtifactCacheStats) -> String {
    let lines = markdown_cache_stats(stats);
    if lines.is_empty() {
        return String::new();
    }

    format!("\nCache activity: {}", lines.join(", "))
}

fn markdown_cache_stats(stats: &ArtifactCacheStats) -> Vec<String> {
    let counters = [
        ("nested_bundles", stats.nested_bundles),
        ("usage_scan", stats.usage_scan),
        ("private_api_scan", stats.private_api_scan),
        ("sdk_scan", stats.sdk_scan),
        ("capability_scan", stats.capability_scan),
        ("signature_summary", stats.signature_summary),
        ("bundle_plist", stats.bundle_plist),
        ("entitlements", stats.entitlements),
        ("provisioning_profile", stats.provisioning_profile),
        ("bundle_files", stats.bundle_files),
    ];

    counters
        .into_iter()
        .filter(|(_, counter)| counter.hits > 0 || counter.misses > 0)
        .map(|(name, counter)| format_cache_counter(name, counter))
        .collect()
}

fn format_cache_counter(name: &str, counter: CacheCounter) -> String {
    format!("{name} h/m={}/{}", counter.hits, counter.misses)
}
