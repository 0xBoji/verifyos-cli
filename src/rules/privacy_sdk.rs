use crate::parsers::macho_scanner::scan_sdks_from_app_bundle;
use crate::parsers::plist_reader::InfoPlist;
use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

pub struct PrivacyManifestSdkCrossCheckRule;

impl AppStoreRule for PrivacyManifestSdkCrossCheckRule {
    fn id(&self) -> &'static str {
        "RULE_PRIVACY_SDK_CROSSCHECK"
    }

    fn name(&self) -> &'static str {
        "Privacy Manifest vs SDK Usage"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Privacy
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Ensure PrivacyInfo.xcprivacy declares data collection and accessed APIs for included SDKs."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let manifest_path = artifact.app_bundle_path.join("PrivacyInfo.xcprivacy");
        if !manifest_path.exists() {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("PrivacyInfo.xcprivacy not found".to_string()),
                evidence: None,
            });
        }

        let manifest = InfoPlist::from_file(&manifest_path)
            .map_err(|_| crate::rules::entitlements::EntitlementsError::ParseFailure)?;

        let scan = match scan_sdks_from_app_bundle(artifact.app_bundle_path) {
            Ok(scan) => scan,
            Err(err) => {
                return Ok(RuleReport {
                    status: RuleStatus::Skip,
                    message: Some(format!("SDK scan skipped: {err}")),
                    evidence: None,
                });
            }
        };

        if scan.hits.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("No SDK signatures detected".to_string()),
                evidence: None,
            });
        }

        let has_data_types = manifest
            .get_value("NSPrivacyCollectedDataTypes")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);
        let has_accessed_api_types = manifest
            .get_value("NSPrivacyAccessedAPITypes")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);

        if has_data_types || has_accessed_api_types {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("Privacy manifest includes SDK data declarations".to_string()),
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("SDKs detected but privacy manifest lacks declarations".to_string()),
            evidence: Some(format!("SDK signatures: {}", scan.hits.join(", "))),
        })
    }
}
