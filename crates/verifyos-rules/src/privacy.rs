use crate::core::{AppStoreRule, ArtifactContext, RuleError, RuleResult, Severity};

pub struct MissingPrivacyManifestRule;

impl AppStoreRule for MissingPrivacyManifestRule {
    fn id(&self) -> &'static str {
        "RULE_PRIVACY_MANIFEST"
    }

    fn name(&self) -> &'static str {
        "Missing Privacy Manifest"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleResult, RuleError> {
        let manifest_path = artifact.app_bundle_path.join("PrivacyInfo.xcprivacy");
        if !manifest_path.exists() {
            return Err(RuleError::MissingPrivacyManifest);
        }

        Ok(RuleResult { success: true })
    }
}
