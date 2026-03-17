use crate::parsers::plist_reader::InfoPlist;
use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

pub struct PrivacyManifestCompletenessRule;

impl AppStoreRule for PrivacyManifestCompletenessRule {
    fn id(&self) -> &'static str {
        "RULE_PRIVACY_MANIFEST_COMPLETENESS"
    }

    fn name(&self) -> &'static str {
        "Privacy Manifest Completeness"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Privacy
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Declare accessed API categories in PrivacyInfo.xcprivacy."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(manifest_path) = artifact.bundle_relative_file("PrivacyInfo.xcprivacy") else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("PrivacyInfo.xcprivacy not found".to_string()),
                evidence: None,
            });
        };

        let manifest = match InfoPlist::from_file(&manifest_path) {
            Ok(m) => m,
            Err(_) => {
                return Ok(RuleReport {
                    status: RuleStatus::Skip,
                    message: Some(
                        "PrivacyInfo.xcprivacy is empty or invalid; skipping".to_string(),
                    ),
                    evidence: Some(manifest_path.display().to_string()),
                });
            }
        };

        let scan = match artifact.usage_scan() {
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

        let has_accessed_api_types = manifest
            .get_value("NSPrivacyAccessedAPITypes")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);

        if has_accessed_api_types {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Privacy manifest missing accessed API types".to_string()),
            evidence: Some("NSPrivacyAccessedAPITypes is missing or empty".to_string()),
        })
    }
}
