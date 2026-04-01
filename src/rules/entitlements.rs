use crate::parsers::plist_reader::InfoPlist;
use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

pub(crate) const APS_ENVIRONMENT_KEY: &str = "aps-environment";
pub(crate) const KEYCHAIN_ACCESS_GROUPS_KEY: &str = "keychain-access-groups";
pub(crate) const ICLOUD_CONTAINER_IDENTIFIERS_KEY: &str =
    "com.apple.developer.icloud-container-identifiers";
pub(crate) const EXTENSION_SUBSET_ENTITLEMENT_KEYS: &[&str] = &[
    APS_ENVIRONMENT_KEY,
    KEYCHAIN_ACCESS_GROUPS_KEY,
    "com.apple.security.application-groups",
    ICLOUD_CONTAINER_IDENTIFIERS_KEY,
    "com.apple.developer.icloud-services",
    "com.apple.developer.associated-domains",
    "com.apple.developer.in-app-payments",
    "com.apple.developer.ubiquity-kvstore-identifier",
    "com.apple.developer.ubiquity-container-identifiers",
    "com.apple.developer.networking.wifi-info",
];

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
        "Debug Entitlements Present"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Entitlements
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn recommendation(&self) -> &'static str {
        "Remove the get-task-allow entitlement for App Store builds."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        if let Some(plist) = artifact.entitlements_for_bundle(artifact.app_bundle_path)? {
            if let Some(true) = plist.get_bool("get-task-allow") {
                return Ok(RuleReport {
                    status: RuleStatus::Fail,
                    message: Some("get-task-allow entitlement is true".to_string()),
                    evidence: Some("Entitlements plist has get-task-allow=true".to_string()),
                });
            }
        }

        Ok(RuleReport {
            status: RuleStatus::Pass,
            message: None,
            evidence: None,
        })
    }
}

pub struct EntitlementsProvisioningMismatchRule;

impl AppStoreRule for EntitlementsProvisioningMismatchRule {
    fn id(&self) -> &'static str {
        "RULE_ENTITLEMENTS_PROVISIONING_MISMATCH"
    }

    fn name(&self) -> &'static str {
        "Entitlements vs Provisioning Mismatch"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Entitlements
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn recommendation(&self) -> &'static str {
        "Ensure entitlements in the app match the embedded provisioning profile."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(entitlements) = load_entitlements_plist(artifact)? else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("No entitlements found".to_string()),
                evidence: None,
            });
        };

        let Some(profile) = artifact
            .provisioning_profile_for_bundle(artifact.app_bundle_path)
            .map_err(RuleError::Provisioning)?
        else {
            let provisioning_path = artifact.app_bundle_path.join("embedded.mobileprovision");
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("embedded.mobileprovision not found".to_string()),
                evidence: Some(provisioning_path.display().to_string()),
            });
        };
        let provisioning_entitlements = profile.entitlements;

        let mut mismatches = Vec::new();

        if let Some(app_aps) = entitlements.get_string(APS_ENVIRONMENT_KEY) {
            match provisioning_entitlements.get_string(APS_ENVIRONMENT_KEY) {
                Some(prov_aps) if prov_aps != app_aps => mismatches.push(format!(
                    "aps-environment: app={} profile={}",
                    app_aps, prov_aps
                )),
                None => mismatches.push("aps-environment missing in profile".to_string()),
                _ => {}
            }
        }

        let keychain_diff = diff_string_array(
            &entitlements,
            &provisioning_entitlements,
            KEYCHAIN_ACCESS_GROUPS_KEY,
        );
        if !keychain_diff.is_empty() {
            mismatches.push(format!(
                "keychain-access-groups missing in profile: {}",
                keychain_diff.join(", ")
            ));
        }

        let icloud_diff = diff_string_array(
            &entitlements,
            &provisioning_entitlements,
            ICLOUD_CONTAINER_IDENTIFIERS_KEY,
        );
        if !icloud_diff.is_empty() {
            mismatches.push(format!(
                "iCloud containers missing in profile: {}",
                icloud_diff.join(", ")
            ));
        }

        if mismatches.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Provisioning profile mismatch".to_string()),
            evidence: Some(mismatches.join("; ")),
        })
    }
}

fn load_entitlements_plist(artifact: &ArtifactContext) -> Result<Option<InfoPlist>, RuleError> {
    artifact.entitlements_for_bundle(artifact.app_bundle_path)
}

pub(crate) fn diff_string_array(
    entitlements: &InfoPlist,
    profile: &InfoPlist,
    key: &str,
) -> Vec<String> {
    let app_values = entitlements.get_array_strings(key).unwrap_or_default();
    let profile_values = profile.get_array_strings(key).unwrap_or_default();

    app_values
        .into_iter()
        .filter(|value| !profile_values.contains(value))
        .collect()
}
