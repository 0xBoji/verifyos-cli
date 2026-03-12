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

fn finding_key(item: &ReportItem) -> String {
    format!(
        "{}|{}",
        item.rule_id,
        item.evidence.clone().unwrap_or_default()
    )
}

fn agent_pack_baseline_key_from_report(item: &ReportItem) -> String {
    format!(
        "{}|{}",
        item.rule_id,
        item.message.clone().unwrap_or_default().trim()
    )
}

fn agent_pack_baseline_key_from_finding(item: &AgentFinding) -> String {
    format!("{}|{}", item.rule_id, item.message.trim())
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

pub fn build_agent_pack(report: &ReportData) -> AgentPack {
    let findings: Vec<AgentFinding> = report
        .results
        .iter()
        .filter(|item| matches!(item.status, RuleStatus::Fail | RuleStatus::Error))
        .map(|item| AgentFinding {
            rule_id: item.rule_id.clone(),
            rule_name: item.rule_name.clone(),
            severity: item.severity,
            category: item.category,
            priority: agent_priority(item.severity).to_string(),
            message: item
                .message
                .clone()
                .unwrap_or_else(|| item.rule_name.clone()),
            evidence: item.evidence.clone(),
            recommendation: item.recommendation.clone(),
            suggested_fix_scope: suggested_fix_scope(item),
            target_files: target_files(item),
            patch_hint: patch_hint(item),
            why_it_fails_review: why_it_fails_review(item),
        })
        .collect();

    AgentPack {
        generated_at_unix: report.generated_at_unix,
        total_findings: findings.len(),
        findings,
    }
}

pub fn apply_agent_pack_baseline(pack: &mut AgentPack, baseline: &ReportData) {
    let baseline_keys: HashSet<String> = baseline
        .results
        .iter()
        .filter(|item| matches!(item.status, RuleStatus::Fail | RuleStatus::Error))
        .map(agent_pack_baseline_key_from_report)
        .collect();

    pack.findings.retain(|finding| {
        let key = agent_pack_baseline_key_from_finding(finding);
        !baseline_keys.contains(&key)
    });
    pack.total_findings = pack.findings.len();
}

pub fn render_agent_pack_markdown(pack: &AgentPack) -> String {
    let mut out = String::new();
    out.push_str("# verifyOS Agent Fix Pack\n\n");
    out.push_str(&format!("- Generated at: `{}`\n", pack.generated_at_unix));
    out.push_str(&format!("- Total findings: `{}`\n\n", pack.total_findings));

    if pack.findings.is_empty() {
        out.push_str("## Findings\n\n- No failing findings.\n");
        return out;
    }

    let mut findings = pack.findings.clone();
    findings.sort_by(|a, b| {
        a.suggested_fix_scope
            .cmp(&b.suggested_fix_scope)
            .then_with(|| a.rule_id.cmp(&b.rule_id))
    });

    out.push_str("## Findings by Fix Scope\n\n");

    let mut current_scope: Option<&str> = None;
    for finding in &findings {
        let scope = finding.suggested_fix_scope.as_str();
        if current_scope != Some(scope) {
            if current_scope.is_some() {
                out.push('\n');
            }
            out.push_str(&format!("### {}\n\n", scope));
            current_scope = Some(scope);
        }

        out.push_str(&format!(
            "- **{}** (`{}`)\n",
            finding.rule_name, finding.rule_id
        ));
        out.push_str(&format!("  - Priority: `{}`\n", finding.priority));
        out.push_str(&format!("  - Severity: `{:?}`\n", finding.severity));
        out.push_str(&format!("  - Category: `{:?}`\n", finding.category));
        out.push_str(&format!("  - Message: {}\n", finding.message));
        if let Some(evidence) = &finding.evidence {
            out.push_str(&format!("  - Evidence: {}\n", evidence));
        }
        if !finding.target_files.is_empty() {
            out.push_str(&format!(
                "  - Target files: {}\n",
                finding.target_files.join(", ")
            ));
        }
        out.push_str(&format!(
            "  - Why it fails review: {}\n",
            finding.why_it_fails_review
        ));
        out.push_str(&format!("  - Patch hint: {}\n", finding.patch_hint));
        out.push_str(&format!("  - Recommendation: {}\n", finding.recommendation));
    }

    out
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
        let slow_rules = format_slow_rules(report.slow_rules.clone());
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
                "invocations": [
                    {
                        "executionSuccessful": true,
                        "properties": {
                            "totalDurationMs": report.total_duration_ms,
                            "slowRules": sarif_slow_rules(&report.slow_rules),
                            "cacheStats": report.cache_stats,
                        }
                    }
                ],
                "tool": {
                    "driver": {
                        "name": "verifyos-cli",
                        "semanticVersion": report.ruleset_version,
                        "rules": rules
                    }
                },
                "properties": {
                    "totalDurationMs": report.total_duration_ms,
                    "slowRules": sarif_slow_rules(&report.slow_rules),
                    "cacheStats": report.cache_stats,
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
        if !report.slow_rules.is_empty() {
            out.push_str("- Slowest rules:\n");
            for item in &report.slow_rules {
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

fn sarif_slow_rules(items: &[SlowRule]) -> Vec<serde_json::Value> {
    items
        .iter()
        .map(|item| {
            serde_json::json!({
                "ruleId": item.rule_id,
                "ruleName": item.rule_name,
                "durationMs": item.duration_ms,
            })
        })
        .collect()
}

fn agent_priority(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "high",
        Severity::Warning => "medium",
        Severity::Info => "low",
    }
}

fn suggested_fix_scope(item: &ReportItem) -> String {
    match item.category {
        RuleCategory::Privacy | RuleCategory::Permissions | RuleCategory::Metadata => {
            "Info.plist".to_string()
        }
        RuleCategory::Entitlements | RuleCategory::Signing => "entitlements".to_string(),
        RuleCategory::Bundling => "bundle-resources".to_string(),
        RuleCategory::Ats => "ats-config".to_string(),
        RuleCategory::ThirdParty => "dependencies".to_string(),
        RuleCategory::Other => "app-bundle".to_string(),
    }
}

fn target_files(item: &ReportItem) -> Vec<String> {
    match item.rule_id.as_str() {
        "RULE_USAGE_DESCRIPTIONS"
        | "RULE_USAGE_DESCRIPTIONS_VALUE"
        | "RULE_CAMERA_USAGE_DESCRIPTION"
        | "RULE_LSAPPLICATIONQUERIESSCHEMES"
        | "RULE_UIREQUIREDDEVICECAPABILITIES"
        | "RULE_INFO_PLIST_VERSIONING" => vec!["Info.plist".to_string()],
        "RULE_PRIVACY_MANIFEST" | "RULE_PRIVACY_SDK_CROSSCHECK" => {
            vec!["PrivacyInfo.xcprivacy".to_string()]
        }
        "RULE_ATS_AUDIT" => vec!["Info.plist (NSAppTransportSecurity)".to_string()],
        "RULE_BUNDLE_RESOURCE_LEAKAGE" => vec!["App bundle resources".to_string()],
        "RULE_ENTITLEMENTS_MISMATCH"
        | "RULE_ENTITLEMENTS_PROVISIONING_MISMATCH"
        | "RULE_EXTENSION_ENTITLEMENTS"
        | "RULE_DEBUG_ENTITLEMENTS" => vec![
            "App entitlements plist".to_string(),
            "embedded.mobileprovision".to_string(),
        ],
        "RULE_EMBEDDED_SIGNING_CONSISTENCY" => vec![
            "Main app executable signature".to_string(),
            "Embedded frameworks/extensions".to_string(),
        ],
        "RULE_PRIVATE_API" => vec!["Linked SDKs or app binary".to_string()],
        _ => match item.category {
            RuleCategory::Privacy | RuleCategory::Permissions | RuleCategory::Metadata => {
                vec!["Info.plist".to_string()]
            }
            RuleCategory::Entitlements | RuleCategory::Signing => {
                vec!["App signing and entitlements".to_string()]
            }
            RuleCategory::Bundling => vec!["App bundle resources".to_string()],
            RuleCategory::Ats => vec!["Info.plist (NSAppTransportSecurity)".to_string()],
            RuleCategory::ThirdParty => vec!["Embedded SDKs or dependencies".to_string()],
            RuleCategory::Other => vec!["App bundle".to_string()],
        },
    }
}

fn patch_hint(item: &ReportItem) -> String {
    match item.rule_id.as_str() {
        "RULE_USAGE_DESCRIPTIONS"
        | "RULE_USAGE_DESCRIPTIONS_VALUE"
        | "RULE_CAMERA_USAGE_DESCRIPTION" => {
            "Update Info.plist with the required NS*UsageDescription keys and give each key a user-facing reason that matches the in-app behavior.".to_string()
        }
        "RULE_LSAPPLICATIONQUERIESSCHEMES" => {
            "Trim LSApplicationQueriesSchemes to only the schemes the app really probes, remove duplicates, and avoid private or overly broad schemes.".to_string()
        }
        "RULE_UIREQUIREDDEVICECAPABILITIES" => {
            "Align UIRequiredDeviceCapabilities with real binary usage so review devices are not excluded by mistake and unsupported hardware is not declared.".to_string()
        }
        "RULE_INFO_PLIST_VERSIONING" => {
            "Set a valid CFBundleShortVersionString and increment CFBundleVersion before the next submission.".to_string()
        }
        "RULE_PRIVACY_MANIFEST" => {
            "Add PrivacyInfo.xcprivacy to the shipped bundle and declare the accessed APIs and collected data used by the app or bundled SDKs.".to_string()
        }
        "RULE_PRIVACY_SDK_CROSSCHECK" => {
            "Review bundled SDKs and extend PrivacyInfo.xcprivacy so their accessed APIs and collected data are explicitly declared.".to_string()
        }
        "RULE_ATS_AUDIT" => {
            "Narrow NSAppTransportSecurity exceptions, remove arbitrary loads when possible, and scope domain exceptions to the smallest set that works.".to_string()
        }
        "RULE_BUNDLE_RESOURCE_LEAKAGE" => {
            "Remove secrets, certificates, provisioning artifacts, debug leftovers, and environment files from the packaged app bundle before archiving.".to_string()
        }
        "RULE_ENTITLEMENTS_MISMATCH" | "RULE_ENTITLEMENTS_PROVISIONING_MISMATCH" => {
            "Make the exported entitlements match the provisioning profile and enabled capabilities for APNs, keychain groups, and iCloud.".to_string()
        }
        "RULE_EXTENSION_ENTITLEMENTS" => {
            "Make each extension entitlement set a valid subset of the host app and add the extension-specific capabilities it actually needs.".to_string()
        }
        "RULE_DEBUG_ENTITLEMENTS" => {
            "Strip debug-only entitlements like get-task-allow from release builds and regenerate the final signed archive.".to_string()
        }
        "RULE_EMBEDDED_SIGNING_CONSISTENCY" => {
            "Re-sign embedded frameworks, dylibs, and extensions with the same Team ID and release identity as the host app.".to_string()
        }
        "RULE_PRIVATE_API" => {
            "Remove or replace private API references in the app binary or third-party SDKs, then rebuild so the shipped binary no longer exposes them.".to_string()
        }
        _ => format!(
            "Patch the {} scope first, then re-run voc to confirm the finding disappears.",
            suggested_fix_scope(item)
        ),
    }
}

fn why_it_fails_review(item: &ReportItem) -> String {
    match item.rule_id.as_str() {
        "RULE_USAGE_DESCRIPTIONS"
        | "RULE_USAGE_DESCRIPTIONS_VALUE"
        | "RULE_CAMERA_USAGE_DESCRIPTION" => {
            "App Review rejects binaries that touch protected APIs without clear, user-facing usage descriptions in Info.plist.".to_string()
        }
        "RULE_LSAPPLICATIONQUERIESSCHEMES" => {
            "Overreaching canOpenURL allowlists look like app enumeration and often trigger manual review questions or rejection.".to_string()
        }
        "RULE_UIREQUIREDDEVICECAPABILITIES" => {
            "Incorrect device capability declarations can exclude valid review devices or misrepresent the hardware the app actually requires.".to_string()
        }
        "RULE_INFO_PLIST_VERSIONING" => {
            "Invalid or non-incrementing version metadata blocks submission and confuses App Store release processing.".to_string()
        }
        "RULE_PRIVACY_MANIFEST" | "RULE_PRIVACY_SDK_CROSSCHECK" => {
            "Apple now expects accurate privacy manifests for apps and bundled SDKs, and missing declarations can block review.".to_string()
        }
        "RULE_ATS_AUDIT" => {
            "Broad ATS exceptions weaken transport security and are a common reason App Review asks teams to justify or remove insecure settings.".to_string()
        }
        "RULE_BUNDLE_RESOURCE_LEAKAGE" => {
            "Shipping secrets, certificates, or provisioning artifacts in the final bundle is treated as a serious distribution and security issue.".to_string()
        }
        "RULE_ENTITLEMENTS_MISMATCH"
        | "RULE_ENTITLEMENTS_PROVISIONING_MISMATCH"
        | "RULE_EXTENSION_ENTITLEMENTS"
        | "RULE_DEBUG_ENTITLEMENTS" => {
            "Entitlements that do not match the signed capabilities or release profile frequently cause validation failures or manual rejection.".to_string()
        }
        "RULE_EMBEDDED_SIGNING_CONSISTENCY" => {
            "Embedded code signed with a different identity or Team ID can fail notarization-style checks during App Store validation.".to_string()
        }
        "RULE_PRIVATE_API" => {
            "Private API usage is one of the clearest App Store rejection reasons because it relies on unsupported system behavior.".to_string()
        }
        _ => format!(
            "This finding maps to the {} scope and signals metadata, signing, or bundle state that App Review may treat as invalid or risky.",
            suggested_fix_scope(item)
        ),
    }
}
