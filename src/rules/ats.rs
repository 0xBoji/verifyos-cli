use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

pub struct AtsAuditRule;

impl AppStoreRule for AtsAuditRule {
    fn id(&self) -> &'static str {
        "RULE_ATS_AUDIT"
    }

    fn name(&self) -> &'static str {
        "ATS Exceptions Detected"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Ats
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Remove ATS exceptions or scope them to specific domains with justification."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let Some(ats_dict) = plist.get_dictionary("NSAppTransportSecurity") else {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        };

        let mut issues = Vec::new();

        if let Some(true) = ats_dict
            .get("NSAllowsArbitraryLoads")
            .and_then(|v| v.as_boolean())
        {
            issues.push("NSAllowsArbitraryLoads=true".to_string());
        }

        if let Some(true) = ats_dict
            .get("NSAllowsArbitraryLoadsInWebContent")
            .and_then(|v| v.as_boolean())
        {
            issues.push("NSAllowsArbitraryLoadsInWebContent=true".to_string());
        }

        if let Some(domains) = ats_dict
            .get("NSExceptionDomains")
            .and_then(|v| v.as_dictionary())
        {
            for (domain, config) in domains {
                if let Some(true) = config
                    .as_dictionary()
                    .and_then(|d| d.get("NSExceptionAllowsInsecureHTTPLoads"))
                    .and_then(|v| v.as_boolean())
                {
                    issues.push(format!("NSExceptionAllowsInsecureHTTPLoads for {domain}"));
                }
            }
        }

        if issues.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("ATS exceptions detected".to_string()),
            evidence: Some(issues.join("; ")),
        })
    }
}
