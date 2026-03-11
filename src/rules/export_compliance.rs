use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

pub struct ExportComplianceRule;

impl AppStoreRule for ExportComplianceRule {
    fn id(&self) -> &'static str {
        "RULE_EXPORT_COMPLIANCE"
    }

    fn name(&self) -> &'static str {
        "Export Compliance Declaration"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Metadata
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Set ITSAppUsesNonExemptEncryption to avoid App Store Connect prompts."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let uses_encryption = plist.get_bool("ITSAppUsesNonExemptEncryption");

        match uses_encryption {
            Some(false) => Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            }),
            Some(true) => Ok(RuleReport {
                status: RuleStatus::Fail,
                message: Some("App uses non-exempt encryption".to_string()),
                evidence: Some("ITSAppUsesNonExemptEncryption=true".to_string()),
            }),
            None => Ok(RuleReport {
                status: RuleStatus::Fail,
                message: Some("Missing export compliance declaration".to_string()),
                evidence: Some("ITSAppUsesNonExemptEncryption not set".to_string()),
            }),
        }
    }
}
