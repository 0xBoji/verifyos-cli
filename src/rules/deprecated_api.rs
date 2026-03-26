use crate::rules::core::{AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity};

pub struct DeprecatedApiRule;

impl AppStoreRule for DeprecatedApiRule {
    fn id(&self) -> &'static str {
        "RULE_DEPRECATED_API"
    }

    fn name(&self) -> &'static str {
        "Deprecated API Usage"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Other
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn recommendation(&self) -> &'static str {
        "Remove usages of deprecated APIs (e.g., UIWebView). Use WKWebView instead."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let executable_path = match artifact.executable_path_for_bundle(artifact.app_bundle_path) {
            Some(path) => path,
            None => return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("No executable found in bundle".to_string()),
                evidence: None,
            }),
        };

        if !executable_path.exists() {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Executable file not found".to_string()),
                evidence: None,
            });
        }

        let bytes = std::fs::read(&executable_path).map_err(|e| {
            RuleError::MachO(crate::parsers::macho_parser::MachOError::ParseError(Box::new(
                apple_codesign::AppleCodesignError::Io(e),
            )))
        })?;

        let target = b"UIWebView";
        let found = bytes.windows(target.len()).any(|window| window == target);

        if found {
            return Ok(RuleReport {
                status: RuleStatus::Fail,
                message: Some("Deprecated API UIWebView detected in binary.".to_string()),
                evidence: Some("Executable contains 'UIWebView'".to_string()),
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Pass,
            message: None,
            evidence: None,
        })
    }
}
