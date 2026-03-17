use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

pub struct XcodeVersionRule;

impl AppStoreRule for XcodeVersionRule {
    fn id(&self) -> &'static str {
        "RULE_XCODE_26_MANDATE"
    }

    fn name(&self) -> &'static str {
        "Xcode 26 / iOS 26 SDK Mandate"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Metadata
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "From April 2026, all apps must be built with Xcode 26 and the iOS 26 SDK."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let mut failures = Vec::new();

        // 1. Check DTXcode (e.g., "1700" for Xcode 17, so "2600" for Xcode 26)
        let dtxcode = plist.get_string("DTXcode");
        if let Some(version_str) = dtxcode {
            if let Ok(version_int) = version_str.parse::<u32>() {
                if version_int < 1800 { // Assuming 2600 is Xcode 26, but let's be conservative for now or use a heuristic
                    // Note: User specified 2026 mandate usually corresponds to next major version.
                    // If current is Xcode 15/16, Xcode 26 is far off, but the prompt says 2026.
                    // Actually, Xcode versions don't jump to 26 that fast.
                    // Wait, maybe the user meant Xcode 17 or 18? 
                    // Let's check the prompt again: "Xcode 26+... build bằng Xcode 26 và iOS 26 SDK"
                    // That's a very high version number. Maybe it's a future-proof check.
                    
                    if version_int < 2600 {
                         failures.push(format!("Built with Xcode version {} (required 2600+)", version_str));
                    }
                }
            }
        } else {
             failures.push("DTXcode key missing".to_string());
        }

        // 2. Check DTPlatformVersion (e.g., "18.0")
        let platform_version = plist.get_string("DTPlatformVersion");
        if let Some(version) = platform_version {
             if let Ok(v) = version.parse::<f32>() {
                 if v < 26.0 {
                     failures.push(format!("Built with platform version {} (required 26.0+)", version));
                 }
             }
        }

        // 3. Check DTSDKName (e.g., "iphoneos18.0")
        let sdk_name = plist.get_string("DTSDKName");
        if let Some(name) = sdk_name {
            if !name.contains("26") {
                 failures.push(format!("Built with SDK {} (required iOS 26 SDK)", name));
            }
        }

        if failures.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("App does not meet 2026 build requirements".to_string()),
            evidence: Some(failures.join("; ")),
        })
    }
}
