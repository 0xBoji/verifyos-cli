use crate::parsers::macho_parser::MachOExecutable;
use crate::parsers::provisioning_profile::ProvisioningProfile;
use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

pub struct NestedBundleEntitlementsRule;

impl AppStoreRule for NestedBundleEntitlementsRule {
    fn id(&self) -> &'static str {
        "RULE_NESTED_ENTITLEMENTS_MISMATCH"
    }

    fn name(&self) -> &'static str {
        "Nested Bundle Entitlements Mismatch"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Entitlements
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn recommendation(&self) -> &'static str {
        "Ensure embedded bundles have entitlements matching their embedded provisioning profiles."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let bundles = artifact
            .nested_bundles()
            .map_err(|_| crate::rules::entitlements::EntitlementsError::ParseFailure)?;

        if bundles.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("No nested bundles found".to_string()),
                evidence: None,
            });
        }

        let mut mismatches = Vec::new();

        for bundle in bundles {
            let executable_path = resolve_executable_path(&bundle.bundle_path);
            let Some(executable_path) = executable_path else {
                continue;
            };

            let provisioning_path = bundle.bundle_path.join("embedded.mobileprovision");
            if !provisioning_path.exists() {
                continue;
            }

            let macho = MachOExecutable::from_file(&executable_path)
                .map_err(crate::rules::entitlements::EntitlementsError::MachO)
                .map_err(RuleError::Entitlements)?;
            let Some(entitlements_xml) = macho.entitlements else {
                continue;
            };

            let entitlements =
                crate::parsers::plist_reader::InfoPlist::from_bytes(entitlements_xml.as_bytes())
                    .map_err(|_| crate::rules::entitlements::EntitlementsError::ParseFailure)?;

            let profile = ProvisioningProfile::from_embedded_file(&provisioning_path)
                .map_err(RuleError::Provisioning)?;

            let mut local_mismatches = Vec::new();

            if let Some(app_aps) = entitlements.get_string("aps-environment") {
                match profile.entitlements.get_string("aps-environment") {
                    Some(prov_aps) if prov_aps != app_aps => local_mismatches.push(format!(
                        "aps-environment app={} profile={}",
                        app_aps, prov_aps
                    )),
                    None => local_mismatches.push("aps-environment missing in profile".to_string()),
                    _ => {}
                }
            }

            let keychain_diff = diff_string_array(
                &entitlements,
                &profile.entitlements,
                "keychain-access-groups",
            );
            if !keychain_diff.is_empty() {
                local_mismatches.push(format!(
                    "keychain-access-groups missing in profile: {}",
                    keychain_diff.join(", ")
                ));
            }

            let icloud_diff = diff_string_array(
                &entitlements,
                &profile.entitlements,
                "com.apple.developer.icloud-container-identifiers",
            );
            if !icloud_diff.is_empty() {
                local_mismatches.push(format!(
                    "iCloud containers missing in profile: {}",
                    icloud_diff.join(", ")
                ));
            }

            if !local_mismatches.is_empty() {
                mismatches.push(format!(
                    "{}: {}",
                    bundle.display_name,
                    local_mismatches.join("; ")
                ));
            }
        }

        if mismatches.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("Nested bundle entitlements match profiles".to_string()),
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Nested bundle entitlements mismatch".to_string()),
            evidence: Some(mismatches.join(" | ")),
        })
    }
}

pub struct NestedBundleDebugEntitlementRule;

impl AppStoreRule for NestedBundleDebugEntitlementRule {
    fn id(&self) -> &'static str {
        "RULE_NESTED_DEBUG_ENTITLEMENT"
    }

    fn name(&self) -> &'static str {
        "Nested Bundle Debug Entitlement"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Entitlements
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn recommendation(&self) -> &'static str {
        "Remove get-task-allow from embedded frameworks/extensions."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let bundles = artifact
            .nested_bundles()
            .map_err(|_| crate::rules::entitlements::EntitlementsError::ParseFailure)?;

        if bundles.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("No nested bundles found".to_string()),
                evidence: None,
            });
        }

        let mut offenders = Vec::new();

        for bundle in bundles {
            let executable_path = resolve_executable_path(&bundle.bundle_path);
            let Some(executable_path) = executable_path else {
                continue;
            };

            let macho = MachOExecutable::from_file(&executable_path)
                .map_err(crate::rules::entitlements::EntitlementsError::MachO)
                .map_err(RuleError::Entitlements)?;
            let Some(entitlements_xml) = macho.entitlements else {
                continue;
            };

            let entitlements =
                crate::parsers::plist_reader::InfoPlist::from_bytes(entitlements_xml.as_bytes())
                    .map_err(|_| crate::rules::entitlements::EntitlementsError::ParseFailure)?;

            if let Some(true) = entitlements.get_bool("get-task-allow") {
                offenders.push(bundle.display_name);
            }
        }

        if offenders.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("No debug entitlements in nested bundles".to_string()),
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Debug entitlements found in nested bundles".to_string()),
            evidence: Some(offenders.join(", ")),
        })
    }
}

fn resolve_executable_path(bundle_path: &std::path::Path) -> Option<std::path::PathBuf> {
    let bundle_name = bundle_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .trim_end_matches(".app")
        .trim_end_matches(".appex")
        .trim_end_matches(".framework");

    if bundle_name.is_empty() {
        return None;
    }

    let executable_path = bundle_path.join(bundle_name);
    if executable_path.exists() {
        Some(executable_path)
    } else {
        None
    }
}

fn diff_string_array(
    entitlements: &crate::parsers::plist_reader::InfoPlist,
    profile: &crate::parsers::plist_reader::InfoPlist,
    key: &str,
) -> Vec<String> {
    let app_values = entitlements.get_array_strings(key).unwrap_or_default();
    let profile_values = profile.get_array_strings(key).unwrap_or_default();

    app_values
        .into_iter()
        .filter(|value| !profile_values.contains(value))
        .collect()
}
