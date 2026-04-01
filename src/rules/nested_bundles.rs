use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};
use crate::rules::entitlements::{
    diff_string_array, APS_ENVIRONMENT_KEY, ICLOUD_CONTAINER_IDENTIFIERS_KEY,
    KEYCHAIN_ACCESS_GROUPS_KEY,
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
            let Some(entitlements) = artifact.entitlements_for_bundle(&bundle.bundle_path)? else {
                continue;
            };
            let Some(profile) = artifact
                .provisioning_profile_for_bundle(&bundle.bundle_path)
                .map_err(RuleError::Provisioning)?
            else {
                continue;
            };

            let mut local_mismatches = Vec::new();

            if let Some(app_aps) = entitlements.get_string(APS_ENVIRONMENT_KEY) {
                match profile.entitlements.get_string(APS_ENVIRONMENT_KEY) {
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
                KEYCHAIN_ACCESS_GROUPS_KEY,
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
                ICLOUD_CONTAINER_IDENTIFIERS_KEY,
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
            let Some(entitlements) = artifact.entitlements_for_bundle(&bundle.bundle_path)? else {
                continue;
            };

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
