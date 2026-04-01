use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};
use std::path::Path;

pub struct AppIconAlphaRule;

impl AppStoreRule for AppIconAlphaRule {
    fn id(&self) -> &'static str {
        "RULE_APP_ICON_ALPHA"
    }

    fn name(&self) -> &'static str {
        "App Icon Alpha Channel Check"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Metadata
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn recommendation(&self) -> &'static str {
        "Remove the alpha channel from your app icon. App Store icons must be opaque."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let icon_names = plist.get_app_icons();
        if icon_names.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("No app icons found in Info.plist".to_string()),
                evidence: None,
            });
        }

        let mut alpha_icons = Vec::new();
        let all_files = artifact.bundle_file_paths();

        for name in icon_names {
            // App Store icons are usually the largest PNGs.
            // We search for files matching the name patterns.
            for file_path in &all_files {
                let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name.starts_with(&name) && file_name.ends_with(".png") {
                    if let Ok(has_alpha) = check_png_alpha(file_path) {
                        if has_alpha {
                            alpha_icons.push(file_name.to_string());
                        }
                    }
                }
            }
        }

        if alpha_icons.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("No alpha channel detected in app icons".to_string()),
                evidence: None,
            });
        }

        alpha_icons.sort();
        alpha_icons.dedup();

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("App icons contain alpha channel (transparency)".to_string()),
            evidence: Some(format!("Icons with alpha: {}", alpha_icons.join(", "))),
        })
    }
}

fn check_png_alpha(path: &Path) -> std::io::Result<bool> {
    let bytes = std::fs::read(path)?;
    if bytes.len() < 26 {
        return Ok(false);
    }

    // Check PNG signature
    if &bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Ok(false);
    }

    // Check IHDR chunk
    if &bytes[12..16] != b"IHDR" {
        return Ok(false);
    }

    // Color type is at index 25
    let color_type = bytes[25];
    match color_type {
        4 | 6 => Ok(true), // Grayscale+Alpha or RGB+Alpha
        3 => {
            // Indexed color - check for tRNS chunk
            Ok(bytes.windows(4).any(|w| w == b"tRNS"))
        }
        _ => Ok(false),
    }
}
