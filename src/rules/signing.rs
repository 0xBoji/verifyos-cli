use crate::parsers::bundle_scanner::find_nested_bundles;
use crate::parsers::macho_parser::read_macho_signature_summary;
use crate::parsers::plist_reader::InfoPlist;
use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};
use std::path::{Path, PathBuf};

pub struct EmbeddedCodeSignatureTeamRule;

impl AppStoreRule for EmbeddedCodeSignatureTeamRule {
    fn id(&self) -> &'static str {
        "RULE_EMBEDDED_TEAM_ID_MISMATCH"
    }

    fn name(&self) -> &'static str {
        "Embedded Team ID Mismatch"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Signing
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn recommendation(&self) -> &'static str {
        "Ensure all embedded frameworks/extensions are signed with the same Team ID as the app binary."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let info_plist = match artifact.info_plist {
            Some(plist) => plist,
            None => {
                let plist_path = artifact.app_bundle_path.join("Info.plist");
                if !plist_path.exists() {
                    return Ok(RuleReport {
                        status: RuleStatus::Skip,
                        message: Some("Info.plist not found".to_string()),
                        evidence: None,
                    });
                }
                match InfoPlist::from_file(&plist_path) {
                    Ok(plist) => return evaluate_with_plist(artifact, &plist),
                    Err(err) => {
                        return Ok(RuleReport {
                            status: RuleStatus::Skip,
                            message: Some(format!("Failed to parse Info.plist: {err}")),
                            evidence: Some(plist_path.display().to_string()),
                        })
                    }
                }
            }
        };

        evaluate_with_plist(artifact, info_plist)
    }
}

fn evaluate_with_plist(
    artifact: &ArtifactContext,
    info_plist: &InfoPlist,
) -> Result<RuleReport, RuleError> {
    let Some(app_executable) = info_plist.get_string("CFBundleExecutable") else {
        return Ok(RuleReport {
            status: RuleStatus::Skip,
            message: Some("CFBundleExecutable not found".to_string()),
            evidence: None,
        });
    };

    let app_executable_path = artifact.app_bundle_path.join(app_executable);
    if !app_executable_path.exists() {
        return Ok(RuleReport {
            status: RuleStatus::Skip,
            message: Some("App executable not found".to_string()),
            evidence: Some(app_executable_path.display().to_string()),
        });
    }

    let app_summary =
        read_macho_signature_summary(&app_executable_path).map_err(RuleError::MachO)?;

    if app_summary.total_slices == 0 {
        return Ok(RuleReport {
            status: RuleStatus::Skip,
            message: Some("No Mach-O slices found".to_string()),
            evidence: Some(app_executable_path.display().to_string()),
        });
    }

    if app_summary.signed_slices == 0 {
        return Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("App executable missing code signature".to_string()),
            evidence: Some(app_executable_path.display().to_string()),
        });
    }

    if app_summary.signed_slices < app_summary.total_slices {
        return Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("App executable has unsigned slices".to_string()),
            evidence: Some(app_executable_path.display().to_string()),
        });
    }

    let Some(app_team_id) = app_summary.team_id else {
        return Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("App executable missing Team ID".to_string()),
            evidence: Some(app_executable_path.display().to_string()),
        });
    };

    let bundles = find_nested_bundles(artifact.app_bundle_path)
        .map_err(|_| crate::rules::entitlements::EntitlementsError::ParseFailure)?;

    if bundles.is_empty() {
        return Ok(RuleReport {
            status: RuleStatus::Pass,
            message: Some("No embedded bundles found".to_string()),
            evidence: None,
        });
    }

    let mut mismatches = Vec::new();

    for bundle in bundles {
        let Some(executable_path) = resolve_bundle_executable(&bundle.bundle_path) else {
            mismatches.push(format!(
                "{}: Missing CFBundleExecutable",
                bundle.display_name
            ));
            continue;
        };

        if !executable_path.exists() {
            mismatches.push(format!(
                "{}: Executable not found at {}",
                bundle.display_name,
                executable_path.display()
            ));
            continue;
        }

        let summary = read_macho_signature_summary(&executable_path).map_err(RuleError::MachO)?;

        if summary.total_slices == 0 {
            mismatches.push(format!("{}: No Mach-O slices found", bundle.display_name));
            continue;
        }

        if summary.signed_slices == 0 {
            mismatches.push(format!("{}: Missing code signature", bundle.display_name));
            continue;
        }

        if summary.signed_slices < summary.total_slices {
            mismatches.push(format!("{}: Unsigned Mach-O slices", bundle.display_name));
            continue;
        }

        let Some(team_id) = summary.team_id else {
            mismatches.push(format!("{}: Missing Team ID", bundle.display_name));
            continue;
        };

        if team_id != app_team_id {
            mismatches.push(format!(
                "{}: Team ID mismatch ({} != {})",
                bundle.display_name, team_id, app_team_id
            ));
        }
    }

    if mismatches.is_empty() {
        return Ok(RuleReport {
            status: RuleStatus::Pass,
            message: Some("Embedded bundles share the same Team ID".to_string()),
            evidence: None,
        });
    }

    Ok(RuleReport {
        status: RuleStatus::Fail,
        message: Some("Embedded bundle signing mismatch".to_string()),
        evidence: Some(mismatches.join(" | ")),
    })
}

fn resolve_bundle_executable(bundle_path: &Path) -> Option<PathBuf> {
    let plist_path = bundle_path.join("Info.plist");
    if plist_path.exists() {
        if let Ok(plist) = InfoPlist::from_file(&plist_path) {
            if let Some(executable) = plist.get_string("CFBundleExecutable") {
                let candidate = bundle_path.join(executable);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

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

    let fallback = bundle_path.join(bundle_name);
    if fallback.exists() {
        Some(fallback)
    } else {
        None
    }
}
