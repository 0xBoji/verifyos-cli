use crate::rules::core::{AppStoreRule, ArtifactContext, RuleError, RuleResult, Severity};

pub struct CameraUsageDescriptionRule;

impl AppStoreRule for CameraUsageDescriptionRule {
    fn id(&self) -> &'static str {
        "RULE_CAMERA_USAGE"
    }

    fn name(&self) -> &'static str {
        "Missing Camera Usage Description"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleResult, RuleError> {
        if let Some(plist) = artifact.info_plist {
            if !plist.has_key("NSCameraUsageDescription") {
                return Err(RuleError::MissingCameraUsageDescription);
            }
        }

        Ok(RuleResult { success: true })
    }
}
