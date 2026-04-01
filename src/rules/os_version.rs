use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

pub struct OSVersionConsistencyRule;

impl AppStoreRule for OSVersionConsistencyRule {
    fn id(&self) -> &'static str {
        "RULE_OS_VERSION_CONSISTENCY"
    }

    fn name(&self) -> &'static str {
        "Minimum OS Version Consistency Check"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Bundling
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Ensure MinimumOSVersion is consistent across the main app and all extensions. Extensions should generally not have a higher MinimumOSVersion than the main app."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(main_plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Main Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let main_version = main_plist.get_string("MinimumOSVersion").unwrap_or("0.0");
        let mut issues = Vec::new();

        if let Ok(nested) = artifact.nested_bundles() {
            for bundle in nested {
                // We only care about app extensions for this specific check,
                // but frameworks also have MinimumOSVersion.
                let bundle_name = &bundle.display_name;
                if let Ok(Some(sub_plist)) = artifact.bundle_info_plist(&bundle.bundle_path) {
                    if let Some(sub_version) = sub_plist.get_string("MinimumOSVersion") {
                        if is_version_higher(sub_version, main_version) {
                            issues.push(format!(
                                "{} has higher MinimumOSVersion ({}) than main app ({})",
                                bundle_name, sub_version, main_version
                            ));
                        }
                    }
                }
            }
        }

        if issues.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some(format!(
                    "MinimumOSVersion ({}) is consistent across all bundles",
                    main_version
                )),
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Inconsistent MinimumOSVersion detected".to_string()),
            evidence: Some(issues.join(" | ")),
        })
    }
}

fn is_version_higher(v1: &str, v2: &str) -> bool {
    let p1: Vec<u32> = v1
        .split('.')
        .filter_map(|s| s.parse::<u32>().ok())
        .collect();
    let p2: Vec<u32> = v2
        .split('.')
        .filter_map(|s| s.parse::<u32>().ok())
        .collect();

    for i in 0..std::cmp::max(p1.len(), p2.len()) {
        let n1 = *p1.get(i).unwrap_or(&0);
        let n2 = *p2.get(i).unwrap_or(&0);
        if n1 > n2 {
            return true;
        }
        if n1 < n2 {
            return false;
        }
    }
    false
}
