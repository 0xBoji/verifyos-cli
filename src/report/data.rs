use crate::core::engine::EngineResult;
use crate::rules::core::{ArtifactCacheStats, RuleCategory, RuleStatus, Severity, RULESET_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportData {
    pub ruleset_version: String,
    pub generated_at_unix: u64,
    pub total_duration_ms: u128,
    pub cache_stats: ArtifactCacheStats,
    pub slow_rules: Vec<SlowRule>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPack {
    pub generated_at_unix: u64,
    pub total_findings: usize,
    pub findings: Vec<AgentFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFinding {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: Severity,
    pub category: RuleCategory,
    pub priority: String,
    pub message: String,
    pub evidence: Option<String>,
    pub recommendation: String,
    pub suggested_fix_scope: String,
    pub target_files: Vec<String>,
    pub patch_hint: String,
    pub why_it_fails_review: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentPackFormat {
    Json,
    Markdown,
    Bundle,
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

    let report = ReportData {
        ruleset_version: RULESET_VERSION.to_string(),
        generated_at_unix,
        total_duration_ms,
        cache_stats,
        slow_rules: Vec::new(),
        results: items,
    };

    ReportData {
        slow_rules: top_slow_rules(&report, 3),
        ..report
    }
}

pub fn apply_baseline(report: &mut ReportData, baseline: &ReportData) -> BaselineSummary {
    let mut suppressed = 0;
    let baseline_keys: HashSet<String> = baseline
        .results
        .iter()
        .filter(|r| matches!(r.status, RuleStatus::Fail | RuleStatus::Error))
        .map(finding_key)
        .collect();

    report.results.retain(|r| {
        if !matches!(r.status, RuleStatus::Fail | RuleStatus::Error) {
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

fn finding_key(item: &ReportItem) -> String {
    format!(
        "{}|{}",
        item.rule_id,
        item.evidence.clone().unwrap_or_default()
    )
}
