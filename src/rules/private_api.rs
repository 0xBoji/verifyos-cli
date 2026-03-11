use crate::parsers::macho_scanner::scan_private_api_from_app_bundle;
use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

pub struct PrivateApiRule;

impl AppStoreRule for PrivateApiRule {
    fn id(&self) -> &'static str {
        "RULE_PRIVATE_API"
    }

    fn name(&self) -> &'static str {
        "Private API Usage Detected"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::ThirdParty
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Remove private API usage or replace with public alternatives."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let scan = match scan_private_api_from_app_bundle(artifact.app_bundle_path) {
            Ok(scan) => scan,
            Err(err) => {
                return Ok(RuleReport {
                    status: RuleStatus::Skip,
                    message: Some(format!("Private API scan skipped: {err}")),
                    evidence: None,
                });
            }
        };

        if scan.hits.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Private API signatures found".to_string()),
            evidence: Some(scan.hits.join(", ")),
        })
    }
}
