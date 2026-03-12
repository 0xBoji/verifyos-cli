use crate::parsers::plist_reader::InfoPlist;
use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

pub struct BundleMetadataConsistencyRule;

impl AppStoreRule for BundleMetadataConsistencyRule {
    fn id(&self) -> &'static str {
        "RULE_BUNDLE_METADATA_CONSISTENCY"
    }

    fn name(&self) -> &'static str {
        "Bundle Metadata Consistency"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Metadata
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Align CFBundleIdentifier and versioning across app and extensions."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(app_plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let app_bundle_id = app_plist.get_string("CFBundleIdentifier");
        let app_short_version = app_plist.get_string("CFBundleShortVersionString");
        let app_build_version = app_plist.get_string("CFBundleVersion");

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
            if !bundle
                .bundle_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|ext| ext == "appex" || ext == "app")
                .unwrap_or(false)
            {
                continue;
            }

            let plist_path = bundle.bundle_path.join("Info.plist");
            if !plist_path.exists() {
                continue;
            }

            let plist = match InfoPlist::from_file(&plist_path) {
                Ok(plist) => plist,
                Err(_) => continue,
            };

            if let (Some(app_id), Some(child_id)) =
                (app_bundle_id, plist.get_string("CFBundleIdentifier"))
            {
                if !child_id.starts_with(app_id) {
                    mismatches.push(format!(
                        "{}: CFBundleIdentifier {} not under {}",
                        bundle.display_name, child_id, app_id
                    ));
                }
            }

            if let (Some(app_short), Some(child_short)) = (
                app_short_version,
                plist.get_string("CFBundleShortVersionString"),
            ) {
                if child_short != app_short {
                    mismatches.push(format!(
                        "{}: CFBundleShortVersionString {} != {}",
                        bundle.display_name, child_short, app_short
                    ));
                }
            }

            if let (Some(app_build), Some(child_build)) =
                (app_build_version, plist.get_string("CFBundleVersion"))
            {
                if child_build != app_build {
                    mismatches.push(format!(
                        "{}: CFBundleVersion {} != {}",
                        bundle.display_name, child_build, app_build
                    ));
                }
            }
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
            message: Some("Bundle metadata mismatches".to_string()),
            evidence: Some(mismatches.join("; ")),
        })
    }
}
