use crate::rules::core::{AppStoreRule, ArtifactContext, RuleError, RuleResult, Severity};
use crate::parsers::macho_parser::MachOExecutable;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum EntitlementsError {
    #[error("Failed to parse Mach-O executable for entitlements")]
    #[diagnostic(
        code(verifyos::entitlements::parse_failure),
        help("The executable could not be parsed as a valid Mach-O binary.")
    )]
    ParseFailure,

    #[error("App contains `get-task-allow` entitlement")]
    #[diagnostic(
        code(verifyos::entitlements::debug_build),
        help("The `get-task-allow` entitlement is present and set to true. This indicates a debug build which will be rejected by the App Store.")
    )]
    DebugEntitlement,

    #[error("Mach-O Parsing Error: {0}")]
    #[diagnostic(code(verifyos::entitlements::macho_error))]
    MachO(#[from] crate::parsers::macho_parser::MachOError),
}

pub struct EntitlementsMismatchRule;

impl AppStoreRule for EntitlementsMismatchRule {
    fn id(&self) -> &'static str {
        "RULE_ENTITLEMENTS_MISMATCH"
    }

    fn name(&self) -> &'static str {
        "Entitlements Mismatch"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleResult, RuleError> {
        let app_name = artifact
            .app_bundle_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .trim_end_matches(".app");

        let executable_path = artifact.app_bundle_path.join(app_name);

        if executable_path.exists() {
            let macho =
                MachOExecutable::from_file(&executable_path).map_err(EntitlementsError::MachO)?;

            if let Some(entitlements_xml) = macho.entitlements {
                // Parse the XML using the plist reader since entitlements are a plist
                let plist = crate::parsers::plist_reader::InfoPlist::from_bytes(
                    entitlements_xml.as_bytes(),
                )
                .map_err(|_| EntitlementsError::ParseFailure)?;

                // For App Store submission, get-task-allow must NOT be true.
                if let Some(true) = plist.get_bool("get-task-allow") {
                    return Err(RuleError::Entitlements(EntitlementsError::DebugEntitlement));
                }
            }
        }

        Ok(RuleResult { success: true })
    }
}
