use crate::parsers::plist_reader::{InfoPlist, PlistError};
use crate::parsers::zip_extractor::{extract_ipa, ExtractionError};
use crate::rules::core::{
    AppStoreRule, ArtifactCacheStats, ArtifactContext, RuleCategory, RuleError, RuleReport,
    Severity,
};
use std::path::Path;
use std::time::Instant;

#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    #[error("Extraction failed: {0}")]
    Extraction(#[from] ExtractionError),
    #[error("Failed to parse Info.plist: {0}")]
    PlistParse(#[from] PlistError),
    #[error("Could not locate App Bundle (.app) inside {0}. Found entries: {1}")]
    AppBundleNotFoundWithContext(String, String),
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
    pub duration_ms: u128,
}

pub struct EngineRun {
    pub results: Vec<EngineResult>,
    pub total_duration_ms: u128,
    pub cache_stats: ArtifactCacheStats,
}

pub struct Engine {
    rules: Vec<Box<dyn AppStoreRule>>,
    pub xcode_project: Option<crate::parsers::xcode_parser::XcodeProject>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            xcode_project: None,
        }
    }

    pub fn register_rule(&mut self, rule: Box<dyn AppStoreRule>) {
        self.rules.push(rule);
    }

    pub fn run<P: AsRef<Path>>(&self, ipa_path: P) -> Result<EngineRun, OrchestratorError> {
        let run_started = Instant::now();
        let path = ipa_path.as_ref();

        if path.is_dir() {
            return self.run_on_bundle(path, run_started);
        }

        let extracted_ipa = extract_ipa(path)?;

        let app_bundle_path = extracted_ipa
            .get_app_bundle_path()
            .map_err(|e| OrchestratorError::Extraction(ExtractionError::Io(e)))?;

        let app_bundle_path = match app_bundle_path {
            Some(p) => p,
            None => {
                let mut entries = Vec::new();
                if let Ok(rd) = std::fs::read_dir(&extracted_ipa.payload_dir) {
                    for entry in rd.flatten().take(10) {
                        entries.push(entry.file_name().to_string_lossy().into_owned());
                    }
                }
                return Err(OrchestratorError::AppBundleNotFoundWithContext(
                    extracted_ipa.payload_dir.display().to_string(),
                    entries.join(", "),
                ));
            }
        };

        self.run_on_bundle(&app_bundle_path, run_started)
    }

    pub fn run_on_bundle<P: AsRef<Path>>(
        &self,
        app_bundle_path: P,
        run_started: Instant,
    ) -> Result<EngineRun, OrchestratorError> {
        let app_bundle_path = app_bundle_path.as_ref();
        let info_plist_path = app_bundle_path.join("Info.plist");
        let info_plist = if info_plist_path.exists() {
            Some(InfoPlist::from_file(&info_plist_path)?)
        } else {
            None
        };

        let context = ArtifactContext::new(
            app_bundle_path,
            info_plist.as_ref(),
            self.xcode_project.as_ref(),
        );

        let mut results = Vec::new();

        for rule in &self.rules {
            let rule_started = Instant::now();
            let res = rule.evaluate(&context);
            results.push(EngineResult {
                rule_id: rule.id(),
                rule_name: rule.name(),
                category: rule.category(),
                severity: rule.severity(),
                recommendation: rule.recommendation(),
                report: res,
                duration_ms: rule_started.elapsed().as_millis(),
            });
        }

        Ok(EngineRun {
            results,
            total_duration_ms: run_started.elapsed().as_millis(),
            cache_stats: context.cache_stats(),
        })
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
