use crate::rules::core::{AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity};

pub struct BitcodeRule;

impl AppStoreRule for BitcodeRule {
    fn id(&self) -> &'static str {
        "RULE_BITCODE_ENABLED"
    }

    fn name(&self) -> &'static str {
        "Bitcode Disabled Requirement"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Signing
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn recommendation(&self) -> &'static str {
        "Apple no longer accepts apps with Bitcode. Disable Bitcode in Build Settings."
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

        // We search for the __LLVM segment name in the Mach-O binary.
        // Apple's bitcode is embedded in a segment exactly named "__LLVM" with null padding.
        let target = b"__LLVM\0\0\0\0\0\0\0\0\0\0";
        let found = bytes.windows(target.len()).any(|window| window == target);

        if found {
            return Ok(RuleReport {
                status: RuleStatus::Fail,
                message: Some("Bitcode segment (__LLVM) found in Mach-O binary.".to_string()),
                evidence: Some("Mach-O contains __LLVM segment".to_string()),
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Pass,
            message: None,
            evidence: None,
        })
    }
}
