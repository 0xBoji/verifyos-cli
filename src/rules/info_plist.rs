use crate::parsers::macho_scanner::{
    scan_capabilities_from_app_bundle, scan_usage_from_app_bundle,
};
use crate::parsers::plist_reader::InfoPlist;
use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

const LOCATION_KEYS: &[&str] = &[
    "NSLocationWhenInUseUsageDescription",
    "NSLocationAlwaysAndWhenInUseUsageDescription",
    "NSLocationAlwaysUsageDescription",
];
const LSQUERY_SCHEME_LIMIT: usize = 50;
const SUSPICIOUS_SCHEMES: &[&str] = &[
    "app-prefs",
    "prefs",
    "settings",
    "sb",
    "sbsettings",
    "sbprefs",
];

pub struct UsageDescriptionsRule;

impl AppStoreRule for UsageDescriptionsRule {
    fn id(&self) -> &'static str {
        "RULE_USAGE_DESCRIPTIONS"
    }

    fn name(&self) -> &'static str {
        "Missing Usage Description Keys"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Privacy
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Add NS*UsageDescription keys required by your app's feature usage."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let scan = match scan_usage_from_app_bundle(artifact.app_bundle_path) {
            Ok(scan) => scan,
            Err(err) => {
                return Ok(RuleReport {
                    status: RuleStatus::Skip,
                    message: Some(format!("Usage scan skipped: {err}")),
                    evidence: None,
                });
            }
        };

        if scan.required_keys.is_empty() && !scan.requires_location_key {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("No usage APIs detected".to_string()),
                evidence: None,
            });
        }

        let mut missing: Vec<&str> = scan
            .required_keys
            .iter()
            .copied()
            .filter(|key| !plist.has_key(key))
            .collect();

        if scan.requires_location_key && !has_any_location_key(plist) {
            missing.push("NSLocationWhenInUseUsageDescription | NSLocationAlwaysAndWhenInUseUsageDescription | NSLocationAlwaysUsageDescription");
        }

        if missing.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Missing required usage description keys".to_string()),
            evidence: Some(format!(
                "Missing keys: {}. Evidence: {}",
                missing.join(", "),
                format_evidence(&scan)
            )),
        })
    }
}

pub struct UsageDescriptionsValueRule;

impl AppStoreRule for UsageDescriptionsValueRule {
    fn id(&self) -> &'static str {
        "RULE_USAGE_DESCRIPTIONS_EMPTY"
    }

    fn name(&self) -> &'static str {
        "Empty Usage Description Values"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Privacy
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Ensure NS*UsageDescription values are non-empty and user-facing."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let scan = match scan_usage_from_app_bundle(artifact.app_bundle_path) {
            Ok(scan) => scan,
            Err(err) => {
                return Ok(RuleReport {
                    status: RuleStatus::Skip,
                    message: Some(format!("Usage scan skipped: {err}")),
                    evidence: None,
                });
            }
        };

        if scan.required_keys.is_empty() && !scan.requires_location_key {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("No usage APIs detected".to_string()),
                evidence: None,
            });
        }

        let mut empty: Vec<&str> = scan
            .required_keys
            .iter()
            .copied()
            .filter(|key| is_empty_string(plist, key))
            .collect();

        if scan.requires_location_key {
            if let Some(key) = find_empty_location_key(plist) {
                empty.push(key);
            }
        }

        if empty.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Usage description values are empty".to_string()),
            evidence: Some(format!(
                "Empty keys: {}. Evidence: {}",
                empty.join(", "),
                format_evidence(&scan)
            )),
        })
    }
}

pub struct InfoPlistRequiredKeysRule;

impl AppStoreRule for InfoPlistRequiredKeysRule {
    fn id(&self) -> &'static str {
        "RULE_INFO_PLIST_REQUIRED_KEYS"
    }

    fn name(&self) -> &'static str {
        "Missing Required Info.plist Keys"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Metadata
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Add required Info.plist keys for your app's functionality."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let mut missing = Vec::new();
        if !plist.has_key("LSApplicationQueriesSchemes") {
            missing.push("LSApplicationQueriesSchemes");
        }
        if !plist.has_key("UIRequiredDeviceCapabilities") {
            missing.push("UIRequiredDeviceCapabilities");
        }

        if missing.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Missing required Info.plist keys".to_string()),
            evidence: Some(format!("Missing keys: {}", missing.join(", "))),
        })
    }
}

pub struct InfoPlistCapabilitiesRule;

impl AppStoreRule for InfoPlistCapabilitiesRule {
    fn id(&self) -> &'static str {
        "RULE_INFO_PLIST_CAPABILITIES_EMPTY"
    }

    fn name(&self) -> &'static str {
        "Empty Info.plist Capability Lists"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Metadata
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Remove empty arrays or populate capability keys with valid values."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let mut empty = Vec::new();

        if is_empty_array(plist, "LSApplicationQueriesSchemes") {
            empty.push("LSApplicationQueriesSchemes");
        }

        if is_empty_array(plist, "UIRequiredDeviceCapabilities") {
            empty.push("UIRequiredDeviceCapabilities");
        }

        if empty.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Capability keys are present but empty".to_string()),
            evidence: Some(format!("Empty keys: {}", empty.join(", "))),
        })
    }
}

pub struct UIRequiredDeviceCapabilitiesAuditRule;

impl AppStoreRule for UIRequiredDeviceCapabilitiesAuditRule {
    fn id(&self) -> &'static str {
        "RULE_DEVICE_CAPABILITIES_AUDIT"
    }

    fn name(&self) -> &'static str {
        "UIRequiredDeviceCapabilities Audit"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Metadata
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Only declare capabilities that match actual hardware usage in the binary."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let Some(declared) = parse_required_capabilities(plist) else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("UIRequiredDeviceCapabilities not declared".to_string()),
                evidence: None,
            });
        };

        if declared.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("UIRequiredDeviceCapabilities is empty".to_string()),
                evidence: None,
            });
        }

        let scan = match scan_capabilities_from_app_bundle(artifact.app_bundle_path) {
            Ok(scan) => scan,
            Err(err) => {
                return Ok(RuleReport {
                    status: RuleStatus::Skip,
                    message: Some(format!("Capability scan skipped: {err}")),
                    evidence: None,
                });
            }
        };

        let mut mismatches = Vec::new();
        for cap in declared {
            let Some(group) = capability_group(&cap) else {
                continue;
            };
            if !scan.detected.contains(group) {
                mismatches.push(format!(
                    "Declared capability '{}' without matching binary usage",
                    cap
                ));
            }
        }

        if mismatches.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("UIRequiredDeviceCapabilities matches binary usage".to_string()),
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Capability list may be overly restrictive".to_string()),
            evidence: Some(mismatches.join(" | ")),
        })
    }
}

pub struct LSApplicationQueriesSchemesAuditRule;

impl AppStoreRule for LSApplicationQueriesSchemesAuditRule {
    fn id(&self) -> &'static str {
        "RULE_LSAPPLICATIONQUERIES_SCHEMES_AUDIT"
    }

    fn name(&self) -> &'static str {
        "LSApplicationQueriesSchemes Audit"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Metadata
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Keep LSApplicationQueriesSchemes minimal, valid, and aligned with actual app usage."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let Some(value) = plist.get_value("LSApplicationQueriesSchemes") else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("LSApplicationQueriesSchemes not declared".to_string()),
                evidence: None,
            });
        };

        let Some(entries) = value.as_array() else {
            return Ok(RuleReport {
                status: RuleStatus::Fail,
                message: Some("LSApplicationQueriesSchemes is not an array".to_string()),
                evidence: None,
            });
        };

        if entries.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("LSApplicationQueriesSchemes is empty".to_string()),
                evidence: None,
            });
        }

        let mut invalid = Vec::new();
        let mut suspicious = Vec::new();
        let mut normalized = std::collections::HashMap::new();

        for entry in entries {
            let Some(raw) = entry.as_string() else {
                invalid.push("<non-string>".to_string());
                continue;
            };
            let trimmed = raw.trim();
            if trimmed.is_empty() || !is_valid_scheme(trimmed) {
                invalid.push(raw.to_string());
                continue;
            }

            let normalized_key = trimmed.to_ascii_lowercase();
            *normalized.entry(normalized_key.clone()).or_insert(0usize) += 1;

            if SUSPICIOUS_SCHEMES
                .iter()
                .any(|scheme| scheme.eq_ignore_ascii_case(&normalized_key))
            {
                suspicious.push(trimmed.to_string());
            }
        }

        let mut issues = Vec::new();
        if entries.len() > LSQUERY_SCHEME_LIMIT {
            issues.push(format!(
                "Contains {} schemes (limit {})",
                entries.len(),
                LSQUERY_SCHEME_LIMIT
            ));
        }

        let mut duplicates: Vec<String> = normalized
            .iter()
            .filter_map(|(scheme, count)| {
                if *count > 1 {
                    Some(scheme.clone())
                } else {
                    None
                }
            })
            .collect();
        duplicates.sort();
        if !duplicates.is_empty() {
            issues.push(format!("Duplicate schemes: {}", duplicates.join(", ")));
        }

        if !invalid.is_empty() {
            issues.push(format!("Invalid scheme entries: {}", invalid.join(", ")));
        }

        if !suspicious.is_empty() {
            issues.push(format!(
                "Potentially private schemes: {}",
                unique_sorted(suspicious).join(", ")
            ));
        }

        if issues.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("LSApplicationQueriesSchemes looks sane".to_string()),
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("LSApplicationQueriesSchemes audit failed".to_string()),
            evidence: Some(issues.join(" | ")),
        })
    }
}

fn is_empty_string(plist: &InfoPlist, key: &str) -> bool {
    match plist.get_string(key) {
        Some(value) => value.trim().is_empty(),
        None => false,
    }
}

fn is_empty_array(plist: &InfoPlist, key: &str) -> bool {
    match plist.get_value(key) {
        Some(value) => value.as_array().map(|arr| arr.is_empty()).unwrap_or(false),
        None => false,
    }
}

fn parse_required_capabilities(plist: &InfoPlist) -> Option<Vec<String>> {
    let value = plist.get_value("UIRequiredDeviceCapabilities")?;

    if let Some(array) = value.as_array() {
        let mut out = Vec::new();
        for item in array {
            if let Some(value) = item.as_string() {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    out.push(trimmed.to_string());
                }
            }
        }
        return Some(out);
    }

    if let Some(dict) = value.as_dictionary() {
        let mut out = Vec::new();
        for (key, value) in dict {
            if let Some(true) = value.as_boolean() {
                out.push(key.to_string());
            }
        }
        return Some(out);
    }

    None
}

fn capability_group(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "camera" | "front-facing-camera" | "rear-facing-camera" => Some("camera"),
        "gps" | "location-services" => Some("location"),
        _ => None,
    }
}

fn has_any_location_key(plist: &InfoPlist) -> bool {
    LOCATION_KEYS.iter().any(|key| plist.has_key(key))
}

fn find_empty_location_key(plist: &InfoPlist) -> Option<&'static str> {
    for key in LOCATION_KEYS {
        if plist.has_key(key) && is_empty_string(plist, key) {
            return Some(*key);
        }
    }
    None
}

fn format_evidence(scan: &crate::parsers::macho_scanner::UsageScan) -> String {
    let mut list: Vec<&str> = scan.evidence.iter().copied().collect();
    list.sort_unstable();
    list.join(", ")
}

fn is_valid_scheme(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_alphabetic() {
        return false;
    }

    for ch in chars {
        if !(ch.is_ascii_alphanumeric() || ch == '+' || ch == '-' || ch == '.') {
            return false;
        }
    }

    true
}

fn unique_sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}
