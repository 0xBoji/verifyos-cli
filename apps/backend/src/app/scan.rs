use crate::domain::{BaselineInfo, ScanProfileInput, ScanRequest, ScanResponse};
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;
use verifyos_cli::core::engine::{Engine, OrchestratorError};
use verifyos_cli::profiles::{available_rule_ids, normalize_rule_id, register_rules, RuleSelection, ScanProfile};
use verifyos_cli::report::{apply_baseline, build_report, BaselineSummary, ReportData};
use zip::ZipArchive;

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("scan failed: {0}")]
    ScanFailed(String),
}

#[derive(Clone, Copy)]
pub struct ScanService;

pub struct ScanOutcome {
    pub report: ReportData,
    pub baseline: Option<BaselineSummary>,
}

impl ScanService {
    pub fn new() -> Self {
        Self
    }

    pub fn run_scan<P: AsRef<Path>>(
        &self,
        request: ScanRequest,
        bundle_path: P,
        project_path: Option<&Path>,
    ) -> Result<ScanResponse, ScanError> {
        let started = Instant::now();
        let outcome = self.run_scan_report(request, bundle_path, project_path)?;
        Ok(ScanResponse {
            report: outcome.report,
            warnings: vec![format!(
                "scan completed in {duration}ms",
                duration = started.elapsed().as_millis()
            )],
            baseline: outcome
                .baseline
                .map(|summary| BaselineInfo { suppressed: summary.suppressed }),
        })
    }

    pub fn run_scan_report<P: AsRef<Path>>(
        &self,
        request: ScanRequest,
        bundle_path: P,
        project_path: Option<&Path>,
    ) -> Result<ScanOutcome, ScanError> {
        let profile = match request.profile {
            Some(ScanProfileInput::Basic) => ScanProfile::Basic,
            Some(ScanProfileInput::Full) | None => ScanProfile::Full,
        };

        let mut engine = Engine::new();
        let selection = build_rule_selection(profile, &request.include, &request.exclude)?;
        register_rules(&mut engine, profile, &selection);

        if let Some(project_path) = project_path {
            if let Some(project) = load_xcode_project(project_path) {
                engine.xcode_project = Some(project);
            }
        }

        let run_started = Instant::now();
        let bundle_path = bundle_path.as_ref();
        let run = engine.run(bundle_path)
            .map_err(|err| ScanError::ScanFailed(err.to_string()))?;

        let mut report = build_report(run.results, run.total_duration_ms, run.cache_stats);
        let baseline = request.baseline.as_ref().map(|baseline| apply_baseline(&mut report, baseline));
        Ok(ScanOutcome { report, baseline })
    }
}

fn build_rule_selection(
    profile: ScanProfile,
    include: &[String],
    exclude: &[String],
) -> Result<RuleSelection, ScanError> {
    let available: std::collections::HashSet<String> =
        available_rule_ids(profile).into_iter().collect();
    let mut include_set = std::collections::HashSet::new();
    let mut exclude_set = std::collections::HashSet::new();

    for value in include {
        let normalized = normalize_rule_id(value);
        if !available.contains(&normalized) {
            return Err(ScanError::ScanFailed(format!(
                "Rule `{}` is not available for this profile",
                normalized
            )));
        }
        include_set.insert(normalized);
    }

    for value in exclude {
        let normalized = normalize_rule_id(value);
        if !available.contains(&normalized) {
            return Err(ScanError::ScanFailed(format!(
                "Rule `{}` is not available for this profile",
                normalized
            )));
        }
        exclude_set.insert(normalized);
    }

    Ok(RuleSelection {
        include: include_set,
        exclude: exclude_set,
    })
}



fn load_xcode_project(
    path: &Path,
) -> Option<verifyos_cli::parsers::xcode_parser::XcodeProject> {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    if extension.eq_ignore_ascii_case("xcworkspace") {
        match verifyos_cli::parsers::xcworkspace_parser::Xcworkspace::from_path(path) {
            Ok(workspace) => {
                for project_path in workspace.project_paths {
                    match verifyos_cli::parsers::xcode_parser::XcodeProject::from_path(
                        &project_path,
                    ) {
                        Ok(project) => return Some(project),
                        Err(err) => {
                            eprintln!(
                                "Warning: Failed to load Xcode project at {}: {}",
                                project_path.display(),
                                err
                            );
                        }
                    }
                }
                eprintln!(
                    "Warning: No usable .xcodeproj found in workspace {}",
                    path.display()
                );
                None
            }
            Err(err) => {
                eprintln!(
                    "Warning: Failed to read Xcode workspace at {}: {}",
                    path.display(),
                    err
                );
                None
            }
        }
    } else if extension.eq_ignore_ascii_case("xcodeproj") {
        match verifyos_cli::parsers::xcode_parser::XcodeProject::from_path(path) {
            Ok(project) => Some(project),
            Err(err) => {
                eprintln!(
                    "Warning: Failed to load Xcode project at {}: {}",
                    path.display(),
                    err
                );
                None
            }
        }
    } else {
        eprintln!(
            "Warning: Unsupported project type at {} (expected .xcodeproj or .xcworkspace)",
            path.display()
        );
        None
    }
}
