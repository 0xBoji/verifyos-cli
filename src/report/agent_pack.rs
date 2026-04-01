use crate::report::data::{AgentFinding, AgentPack, ReportData, ReportItem};
use crate::rules::core::{RuleCategory, RuleStatus, Severity};
use std::collections::HashSet;

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

fn canonical_rule_id(rule_id: &str) -> &str {
    match rule_id {
        "RULE_USAGE_DESCRIPTIONS_VALUE" => "RULE_USAGE_DESCRIPTIONS_EMPTY",
        "RULE_CAMERA_USAGE_DESCRIPTION" => "RULE_CAMERA_USAGE",
        "RULE_LSAPPLICATIONQUERIESSCHEMES" => "RULE_LSAPPLICATIONQUERIES_SCHEMES_AUDIT",
        "RULE_UIREQUIREDDEVICECAPABILITIES" => "RULE_DEVICE_CAPABILITIES_AUDIT",
        "RULE_EXTENSION_ENTITLEMENTS" => "RULE_EXTENSION_ENTITLEMENTS_COMPAT",
        "RULE_EMBEDDED_SIGNING_CONSISTENCY" => "RULE_EMBEDDED_TEAM_ID_MISMATCH",
        other => other,
    }
}

fn target_files(item: &ReportItem) -> Vec<String> {
    match canonical_rule_id(&item.rule_id) {
        "RULE_USAGE_DESCRIPTIONS"
        | "RULE_USAGE_DESCRIPTIONS_EMPTY"
        | "RULE_CAMERA_USAGE"
        | "RULE_INFO_PLIST_VERSIONING" => vec!["Info.plist".to_string()],
        "RULE_LSAPPLICATIONQUERIES_SCHEMES_AUDIT" | "RULE_DEVICE_CAPABILITIES_AUDIT" => {
            vec!["Info.plist".to_string()]
        }
        "RULE_PRIVACY_MANIFEST"
        | "RULE_PRIVACY_MANIFEST_COMPLETENESS"
        | "RULE_PRIVACY_SDK_CROSSCHECK" => {
            vec!["PrivacyInfo.xcprivacy".to_string()]
        }
        "RULE_ATS_AUDIT" => vec!["Info.plist (NSAppTransportSecurity)".to_string()],
        "RULE_BUNDLE_RESOURCE_LEAKAGE" => vec!["App bundle resources".to_string()],
        "RULE_ENTITLEMENTS_MISMATCH"
        | "RULE_ENTITLEMENTS_PROVISIONING_MISMATCH"
        | "RULE_EXTENSION_ENTITLEMENTS_COMPAT"
        | "RULE_NESTED_ENTITLEMENTS_MISMATCH"
        | "RULE_NESTED_DEBUG_ENTITLEMENT" => vec![
            "App entitlements plist".to_string(),
            "embedded.mobileprovision".to_string(),
        ],
        "RULE_EMBEDDED_TEAM_ID_MISMATCH" => vec![
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
    match canonical_rule_id(&item.rule_id) {
        "RULE_USAGE_DESCRIPTIONS"
        | "RULE_USAGE_DESCRIPTIONS_EMPTY"
        | "RULE_CAMERA_USAGE" => {
            "Update Info.plist with the required NS*UsageDescription keys and give each key a user-facing reason that matches the in-app behavior.".to_string()
        }
        "RULE_LSAPPLICATIONQUERIES_SCHEMES_AUDIT" => {
            "Trim LSApplicationQueriesSchemes to only the schemes the app really probes, remove duplicates, and avoid private or overly broad schemes.".to_string()
        }
        "RULE_DEVICE_CAPABILITIES_AUDIT" => {
            "Align UIRequiredDeviceCapabilities with real binary usage so review devices are not excluded by mistake and unsupported hardware is not declared.".to_string()
        }
        "RULE_INFO_PLIST_VERSIONING" => {
            "Set a valid CFBundleShortVersionString and increment CFBundleVersion before the next submission.".to_string()
        }
        "RULE_PRIVACY_MANIFEST" | "RULE_PRIVACY_MANIFEST_COMPLETENESS" => {
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
        "RULE_EXTENSION_ENTITLEMENTS_COMPAT" | "RULE_NESTED_ENTITLEMENTS_MISMATCH" => {
            "Make each extension entitlement set a valid subset of the host app and add the extension-specific capabilities it actually needs.".to_string()
        }
        "RULE_NESTED_DEBUG_ENTITLEMENT" => {
            "Strip debug-only entitlements like get-task-allow from release builds and regenerate the final signed archive.".to_string()
        }
        "RULE_EMBEDDED_TEAM_ID_MISMATCH" => {
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
    match canonical_rule_id(&item.rule_id) {
        "RULE_USAGE_DESCRIPTIONS"
        | "RULE_USAGE_DESCRIPTIONS_EMPTY"
        | "RULE_CAMERA_USAGE" => {
            "App Review rejects binaries that touch protected APIs without clear, user-facing usage descriptions in Info.plist.".to_string()
        }
        "RULE_LSAPPLICATIONQUERIES_SCHEMES_AUDIT" => {
            "Overreaching canOpenURL allowlists look like app enumeration and often trigger manual review questions or rejection.".to_string()
        }
        "RULE_DEVICE_CAPABILITIES_AUDIT" => {
            "Incorrect device capability declarations can exclude valid review devices or misrepresent the hardware the app actually requires.".to_string()
        }
        "RULE_INFO_PLIST_VERSIONING" => {
            "Invalid or non-incrementing version metadata blocks submission and confuses App Store release processing.".to_string()
        }
        "RULE_PRIVACY_MANIFEST"
        | "RULE_PRIVACY_MANIFEST_COMPLETENESS"
        | "RULE_PRIVACY_SDK_CROSSCHECK" => {
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
        | "RULE_EXTENSION_ENTITLEMENTS_COMPAT"
        | "RULE_NESTED_ENTITLEMENTS_MISMATCH"
        | "RULE_NESTED_DEBUG_ENTITLEMENT" => {
            "Entitlements that do not match the signed capabilities or release profile frequently cause validation failures or manual rejection.".to_string()
        }
        "RULE_EMBEDDED_TEAM_ID_MISMATCH" => {
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

#[cfg(test)]
mod tests {
    use super::build_agent_pack;
    use crate::report::data::{ReportData, ReportItem};
    use crate::rules::core::{ArtifactCacheStats, RuleCategory, RuleStatus, Severity};

    fn report_item(rule_id: &str) -> ReportItem {
        ReportItem {
            rule_id: rule_id.to_string(),
            rule_name: rule_id.to_string(),
            category: RuleCategory::Metadata,
            severity: Severity::Warning,
            target: "Demo.app".to_string(),
            status: RuleStatus::Fail,
            message: Some("example".to_string()),
            evidence: None,
            recommendation: "fix it".to_string(),
            duration_ms: 1,
        }
    }

    #[test]
    fn agent_pack_maps_current_rule_ids() {
        let report = ReportData {
            ruleset_version: "test".to_string(),
            generated_at_unix: 0,
            total_duration_ms: 0,
            cache_stats: ArtifactCacheStats::default(),
            slow_rules: Vec::new(),
            results: vec![
                report_item("RULE_USAGE_DESCRIPTIONS_EMPTY"),
                report_item("RULE_LSAPPLICATIONQUERIES_SCHEMES_AUDIT"),
                report_item("RULE_DEVICE_CAPABILITIES_AUDIT"),
            ],
            scanned_targets: vec!["Demo.app".to_string()],
        };

        let pack = build_agent_pack(&report);

        assert_eq!(pack.total_findings, 3);
        assert!(pack
            .findings
            .iter()
            .all(|finding| finding.target_files == vec!["Info.plist".to_string()]));
    }
}
