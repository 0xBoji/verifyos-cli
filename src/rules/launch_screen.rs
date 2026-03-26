use crate::rules::core::{AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity};

pub struct LaunchScreenStoryboardRule;

impl AppStoreRule for LaunchScreenStoryboardRule {
    fn id(&self) -> &'static str {
        "RULE_LAUNCH_SCREEN_STORYBOARD"
    }

    fn name(&self) -> &'static str {
        "Launch Screen Storyboard Requirement"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Metadata
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn recommendation(&self) -> &'static str {
        "Remove UILaunchImages from Info.plist and use UILaunchStoryboardName instead."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("No Info.plist found".to_string()),
                evidence: None,
            });
        };

        // Skip for App Extensions, typical extensions have NSExtension in Info.plist
        if plist.get_value("NSExtension").is_some() {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Rule does not apply to app extensions".to_string()),
                evidence: None,
            });
        }

        let has_launch_images = plist.get_value("UILaunchImages").is_some();
        let has_storyboard = plist.get_value("UILaunchStoryboardName").is_some();

        if has_launch_images {
            return Ok(RuleReport {
                status: RuleStatus::Fail,
                message: Some("App uses deprecated UILaunchImages.".to_string()),
                evidence: Some("UILaunchImages key found in Info.plist".to_string()),
            });
        }

        if !has_storyboard {
            return Ok(RuleReport {
                status: RuleStatus::Fail,
                message: Some("App is missing UILaunchStoryboardName.".to_string()),
                evidence: Some("UILaunchStoryboardName key missing from Info.plist".to_string()),
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Pass,
            message: None,
            evidence: None,
        })
    }
}
