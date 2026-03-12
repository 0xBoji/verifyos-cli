use crate::parsers::macho_parser::MachOExecutable;
use crate::parsers::plist_reader::InfoPlist;
use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};
use std::path::Path;

pub struct ExtensionEntitlementsCompatibilityRule;

impl AppStoreRule for ExtensionEntitlementsCompatibilityRule {
    fn id(&self) -> &'static str {
        "RULE_EXTENSION_ENTITLEMENTS_COMPAT"
    }

    fn name(&self) -> &'static str {
        "Extension Entitlements Compatibility"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Entitlements
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Ensure extension entitlements are a subset of the host app and required keys exist for the extension type."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let bundles = artifact
            .nested_bundles()
            .map_err(|_| crate::rules::entitlements::EntitlementsError::ParseFailure)?;

        let extensions: Vec<_> = bundles
            .into_iter()
            .filter(|bundle| {
                bundle
                    .bundle_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|ext| ext == "appex")
                    .unwrap_or(false)
            })
            .collect();

        if extensions.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("No extensions found".to_string()),
                evidence: None,
            });
        }

        let Some(app_entitlements) = load_entitlements_plist(artifact.app_bundle_path)? else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Host app entitlements not found".to_string()),
                evidence: None,
            });
        };

        let mut issues = Vec::new();

        for extension in extensions {
            let plist_path = extension.bundle_path.join("Info.plist");
            let plist = match InfoPlist::from_file(&plist_path) {
                Ok(plist) => plist,
                Err(_) => continue,
            };

            let extension_point = plist
                .get_dictionary("NSExtension")
                .and_then(|dict| dict.get("NSExtensionPointIdentifier"))
                .and_then(|value| value.as_string())
                .unwrap_or("unknown")
                .to_string();

            let Some(ext_entitlements) = load_entitlements_for_bundle(&extension.bundle_path)?
            else {
                continue;
            };

            let subset_issues = compare_entitlements(&app_entitlements, &ext_entitlements);
            for issue in subset_issues {
                issues.push(format!("{}: {}", extension.display_name, issue));
            }

            let required = required_entitlements_for_extension(&extension_point);
            for requirement in required {
                if !ext_entitlements.has_key(requirement) {
                    issues.push(format!(
                        "{}: missing entitlement {} for {}",
                        extension.display_name, requirement, extension_point
                    ));
                    continue;
                }
                if *requirement == "com.apple.security.application-groups" {
                    let groups = ext_entitlements
                        .get_array_strings(requirement)
                        .unwrap_or_default();
                    if groups.is_empty() {
                        issues.push(format!(
                            "{}: empty entitlement {} for {}",
                            extension.display_name, requirement, extension_point
                        ));
                    }
                }
            }
        }

        if issues.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("Extension entitlements are compatible".to_string()),
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Extension entitlements mismatch".to_string()),
            evidence: Some(issues.join(" | ")),
        })
    }
}

fn load_entitlements_plist(app_bundle_path: &Path) -> Result<Option<InfoPlist>, RuleError> {
    let app_name = app_bundle_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .trim_end_matches(".app");

    if app_name.is_empty() {
        return Ok(None);
    }

    let executable_path = app_bundle_path.join(app_name);
    if !executable_path.exists() {
        return Ok(None);
    }

    let macho = MachOExecutable::from_file(&executable_path)
        .map_err(crate::rules::entitlements::EntitlementsError::MachO)
        .map_err(RuleError::Entitlements)?;
    let Some(entitlements_xml) = macho.entitlements else {
        return Ok(None);
    };

    let plist = InfoPlist::from_bytes(entitlements_xml.as_bytes())
        .map_err(|_| crate::rules::entitlements::EntitlementsError::ParseFailure)?;

    Ok(Some(plist))
}

fn load_entitlements_for_bundle(bundle_path: &Path) -> Result<Option<InfoPlist>, RuleError> {
    let bundle_name = bundle_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .trim_end_matches(".appex")
        .trim_end_matches(".app");

    if bundle_name.is_empty() {
        return Ok(None);
    }

    let executable_path = bundle_path.join(bundle_name);
    if !executable_path.exists() {
        return Ok(None);
    }

    let macho = MachOExecutable::from_file(&executable_path)
        .map_err(crate::rules::entitlements::EntitlementsError::MachO)
        .map_err(RuleError::Entitlements)?;
    let Some(entitlements_xml) = macho.entitlements else {
        return Ok(None);
    };

    let plist = InfoPlist::from_bytes(entitlements_xml.as_bytes())
        .map_err(|_| crate::rules::entitlements::EntitlementsError::ParseFailure)?;

    Ok(Some(plist))
}

fn compare_entitlements(app: &InfoPlist, ext: &InfoPlist) -> Vec<String> {
    let mut issues = Vec::new();

    for key in [
        "aps-environment",
        "keychain-access-groups",
        "com.apple.security.application-groups",
        "com.apple.developer.icloud-container-identifiers",
        "com.apple.developer.icloud-services",
        "com.apple.developer.associated-domains",
        "com.apple.developer.in-app-payments",
        "com.apple.developer.ubiquity-kvstore-identifier",
        "com.apple.developer.ubiquity-container-identifiers",
        "com.apple.developer.networking.wifi-info",
    ] {
        if !ext.has_key(key) {
            continue;
        }

        if !app.has_key(key) {
            issues.push(format!("entitlement {key} not present in host app"));
            continue;
        }

        if let Some(values) = ext.get_array_strings(key) {
            let app_values = app.get_array_strings(key).unwrap_or_default();
            let missing: Vec<String> = values
                .into_iter()
                .filter(|value| !app_values.contains(value))
                .collect();
            if !missing.is_empty() {
                issues.push(format!(
                    "entitlement {key} values not in host app: {}",
                    missing.join(", ")
                ));
            }
            continue;
        }

        if let Some(value) = ext.get_string(key) {
            if app.get_string(key) != Some(value) {
                issues.push(format!("entitlement {key} mismatch"));
            }
            continue;
        }

        if let Some(value) = ext.get_bool(key) {
            if app.get_bool(key) != Some(value) {
                issues.push(format!("entitlement {key} mismatch"));
            }
        }
    }

    issues
}

fn required_entitlements_for_extension(extension_point: &str) -> &'static [&'static str] {
    match extension_point {
        "com.apple.widgetkit-extension" => &["com.apple.security.application-groups"],
        "com.apple.usernotifications.service" => &["aps-environment"],
        _ => &[],
    }
}
