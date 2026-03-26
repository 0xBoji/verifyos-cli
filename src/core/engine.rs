use crate::parsers::plist_reader::{InfoPlist, PlistError};
use crate::parsers::zip_extractor::{extract_ipa, ExtractionError};
use crate::rules::core::{
    AppStoreRule, ArtifactCacheStats, ArtifactContext, RuleCategory, RuleError, RuleReport,
    Severity,
};
use std::path::Path;
use std::time::Instant;
use rayon::prelude::*;

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
    pub target: String,
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

    pub fn run<P: AsRef<Path>>(&self, path_or_ipa: P) -> Result<EngineRun, OrchestratorError> {
        let run_started = Instant::now();
        let path = path_or_ipa.as_ref();

        let mut extracted = None;
        let targets = if path.is_dir() {
            // Direct directory scan: mock ExtractedIpa discover_targets behavior
            let mut targets = Vec::new();
            let mut queue = vec![path.to_path_buf()];
            while let Some(dir) = queue.pop() {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            let extension = p.extension().and_then(|e| e.to_str());
                            match extension {
                                Some("app") => targets.push((p.clone(), "app".to_string())),
                                Some("xcodeproj") => {
                                    targets.push((p.clone(), "project".to_string()))
                                }
                                Some("xcworkspace") => {
                                    targets.push((p.clone(), "workspace".to_string()))
                                }
                                _ => queue.push(p),
                            }
                        } else if p.extension().and_then(|e| e.to_str()) == Some("ipa") {
                            targets.push((p.clone(), "ipa".to_string()));
                        }
                    }
                }
            }
            targets
        } else {
            let extracted_ipa = extract_ipa(path)?;
            let t = extracted_ipa
                .discover_targets()
                .map_err(|e| OrchestratorError::Extraction(ExtractionError::Io(e)))?;
            extracted = Some(extracted_ipa);
            t
        };

        let mut all_results = Vec::new();
        let mut total_cache_stats = ArtifactCacheStats::default();

        if targets.is_empty() {
            // Fallback to original logic if no specific targets found
            let res = if let Some(ref ext) = extracted {
                self.run_on_bundle_internal(&ext.payload_dir, run_started, None)?
            } else {
                self.run_on_bundle_internal(path, run_started, None)?
            };
            return Ok(res);
        }

        for (target_path, target_type) in targets {
            let project_context = if target_type == "app" {
                // Try to find a project context for this app if it's in a larger folder
                let mut p_context = None;
                if let Some(parent) = target_path.parent() {
                    if let Ok(entries) = std::fs::read_dir(parent) {
                        for entry in entries.flatten() {
                            let p = entry.path();
                            if p.extension().is_some_and(|e| e == "xcworkspace") {
                                p_context =
                                    crate::parsers::xcworkspace_parser::Xcworkspace::from_path(&p)
                                        .ok()
                                        .and_then(|ws| ws.project_paths.first().cloned())
                                        .and_then(|proj_path| {
                                            crate::parsers::xcode_parser::XcodeProject::from_path(
                                                proj_path,
                                            )
                                            .ok()
                                        });
                                break;
                            } else if p.extension().is_some_and(|e| e == "xcodeproj") {
                                p_context =
                                    crate::parsers::xcode_parser::XcodeProject::from_path(&p).ok();
                            }
                        }
                    }
                }
                p_context
            } else if target_type == "project" {
                crate::parsers::xcode_parser::XcodeProject::from_path(&target_path).ok()
            } else if target_type == "workspace" {
                crate::parsers::xcworkspace_parser::Xcworkspace::from_path(&target_path)
                    .ok()
                    .and_then(|ws| ws.project_paths.first().cloned())
                    .and_then(|proj_path| {
                        crate::parsers::xcode_parser::XcodeProject::from_path(proj_path).ok()
                    })
            } else {
                None
            };

            let app_results =
                self.run_on_bundle_internal(&target_path, run_started, project_context)?;

            // Tag results with target name
            let target_name = target_path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Unknown".to_string());

            for mut res in app_results.results {
                res.target = target_name.clone();
                all_results.push(res);
            }

            // Merge stats
            total_cache_stats.nested_bundles.hits += app_results.cache_stats.nested_bundles.hits;
            total_cache_stats.nested_bundles.misses +=
                app_results.cache_stats.nested_bundles.misses;
        }

        Ok(EngineRun {
            results: all_results,
            total_duration_ms: run_started.elapsed().as_millis(),
            cache_stats: total_cache_stats,
        })
    }

    pub fn run_on_bundle<P: AsRef<Path>>(
        &self,
        app_bundle_path: P,
        run_started: Instant,
    ) -> Result<EngineRun, OrchestratorError> {
        self.run_on_bundle_internal(app_bundle_path, run_started, None)
    }

    fn run_on_bundle_internal<P: AsRef<Path>>(
        &self,
        app_bundle_path: P,
        run_started: Instant,
        project_override: Option<crate::parsers::xcode_parser::XcodeProject>,
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
            project_override.as_ref().or(self.xcode_project.as_ref()),
        );

        let results: Vec<EngineResult> = self
            .rules
            .par_iter()
            .map(|rule| {
                let rule_started = Instant::now();
                let res = rule.evaluate(&context);
                EngineResult {
                    rule_id: rule.id(),
                    rule_name: rule.name(),
                    category: rule.category(),
                    severity: rule.severity(),
                    target: app_bundle_path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Bundle".to_string()),
                    recommendation: rule.recommendation(),
                    report: res,
                    duration_ms: rule_started.elapsed().as_millis(),
                }
            })
            .collect();

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
