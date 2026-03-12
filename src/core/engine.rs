use crate::parsers::plist_reader::{InfoPlist, PlistError};
use crate::parsers::zip_extractor::{extract_ipa, ExtractionError};
use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, Severity,
};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    #[error("Extraction failed: {0}")]
    Extraction(#[from] ExtractionError),
    #[error("Failed to parse Info.plist: {0}")]
    PlistParse(#[from] PlistError),
    #[error("Could not locate App Bundle (.app) inside IPAPayload")]
    AppBundleNotFound,
}

pub struct EngineResult {
    pub rule_id: &'static str,
    pub rule_name: &'static str,
    pub category: RuleCategory,
    pub severity: Severity,
    pub recommendation: &'static str,
    pub report: Result<RuleReport, RuleError>,
}

pub struct Engine {
    rules: Vec<Box<dyn AppStoreRule>>,
}

impl Engine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn register_rule(&mut self, rule: Box<dyn AppStoreRule>) {
        self.rules.push(rule);
    }

    pub fn run<P: AsRef<Path>>(&self, ipa_path: P) -> Result<Vec<EngineResult>, OrchestratorError> {
        let extracted_ipa = extract_ipa(ipa_path)?;

        let app_bundle_path = extracted_ipa
            .get_app_bundle_path()
            .map_err(|e| OrchestratorError::Extraction(ExtractionError::Io(e)))?
            .ok_or(OrchestratorError::AppBundleNotFound)?;

        let info_plist_path = app_bundle_path.join("Info.plist");
        let info_plist = if info_plist_path.exists() {
            Some(InfoPlist::from_file(&info_plist_path)?)
        } else {
            None
        };

        let context = ArtifactContext::new(&app_bundle_path, info_plist.as_ref());

        let mut results = Vec::new();

        for rule in &self.rules {
            let res = rule.evaluate(&context);
            results.push(EngineResult {
                rule_id: rule.id(),
                rule_name: rule.name(),
                category: rule.category(),
                severity: rule.severity(),
                recommendation: rule.recommendation(),
                report: res,
            });
        }

        Ok(results)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
