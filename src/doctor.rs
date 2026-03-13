use crate::agent_assets::RepairPlanItem;
use crate::config::load_file_config;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DoctorStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorCheck {
    pub name: String,
    pub status: DoctorStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
    #[serde(default)]
    pub repair_plan: Vec<RepairPlanItem>,
    #[serde(default)]
    pub plan_context: Option<DoctorPlanContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorPlanContext {
    pub source: String,
    pub scan_artifact: Option<String>,
    pub baseline_path: Option<String>,
    pub freshness_source: Option<String>,
    pub repair_targets: Vec<String>,
}

impl DoctorReport {
    pub fn has_failures(&self) -> bool {
        self.checks
            .iter()
            .any(|item| item.status == DoctorStatus::Fail)
    }
}

pub fn run_doctor(
    config_path: Option<&Path>,
    agents_path: &Path,
    freshness_against: Option<&Path>,
) -> DoctorReport {
    let mut checks = Vec::new();

    checks.push(check_config(config_path));
    checks.push(check_agents_presence(agents_path));

    if agents_path.exists() {
        let contents = std::fs::read_to_string(agents_path).unwrap_or_default();
        checks.push(check_referenced_assets(&contents, agents_path));
        checks.push(check_asset_freshness(
            &contents,
            agents_path,
            freshness_against,
        ));
        checks.push(check_next_commands(&contents));
        checks.push(check_next_steps_script(&contents, agents_path));
    }

    DoctorReport {
        checks,
        repair_plan: Vec::new(),
        plan_context: None,
    }
}

fn check_config(config_path: Option<&Path>) -> DoctorCheck {
    match load_file_config(config_path) {
        Ok(_) => DoctorCheck {
            name: "Config".to_string(),
            status: DoctorStatus::Pass,
            detail: config_path
                .map(|path| format!("Config is valid: {}", path.display()))
                .unwrap_or_else(|| "Config is valid or not present".to_string()),
        },
        Err(err) => DoctorCheck {
            name: "Config".to_string(),
            status: DoctorStatus::Fail,
            detail: err.to_string(),
        },
    }
}

fn check_agents_presence(agents_path: &Path) -> DoctorCheck {
    if agents_path.exists() {
        DoctorCheck {
            name: "AGENTS.md".to_string(),
            status: DoctorStatus::Pass,
            detail: format!("Found {}", agents_path.display()),
        }
    } else {
        DoctorCheck {
            name: "AGENTS.md".to_string(),
            status: DoctorStatus::Warn,
            detail: format!("Missing {}", agents_path.display()),
        }
    }
}

fn check_referenced_assets(contents: &str, agents_path: &Path) -> DoctorCheck {
    let referenced = extract_backticked_paths(contents);
    if referenced.is_empty() {
        return DoctorCheck {
            name: "Referenced assets".to_string(),
            status: DoctorStatus::Warn,
            detail: "No referenced agent assets found in AGENTS.md".to_string(),
        };
    }

    let mut missing = Vec::new();
    for item in referenced {
        let path = resolve_reference(agents_path, &item);
        if !path.exists() {
            missing.push(path.display().to_string());
        }
    }

    if missing.is_empty() {
        DoctorCheck {
            name: "Referenced assets".to_string(),
            status: DoctorStatus::Pass,
            detail: "All referenced agent assets exist".to_string(),
        }
    } else {
        DoctorCheck {
            name: "Referenced assets".to_string(),
            status: DoctorStatus::Fail,
            detail: format!("Missing assets: {}", missing.join(", ")),
        }
    }
}

fn check_asset_freshness(
    contents: &str,
    agents_path: &Path,
    freshness_against: Option<&Path>,
) -> DoctorCheck {
    let output_dir = agents_path.parent().unwrap_or_else(|| Path::new("."));
    let Some((report_path, report_modified)) =
        resolve_freshness_source(output_dir, freshness_against)
    else {
        return DoctorCheck {
            name: "Asset freshness".to_string(),
            status: DoctorStatus::Pass,
            detail: "No report.json or report.sarif found; freshness check skipped".to_string(),
        };
    };

    let mut stale = Vec::new();
    for path in freshness_targets(contents, agents_path) {
        let Ok(metadata) = std::fs::metadata(&path) else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if modified < report_modified {
            stale.push(path.display().to_string());
        }
    }

    if stale.is_empty() {
        DoctorCheck {
            name: "Asset freshness".to_string(),
            status: DoctorStatus::Pass,
            detail: format!(
                "Generated assets are at least as new as {}",
                report_path.display()
            ),
        }
    } else {
        DoctorCheck {
            name: "Asset freshness".to_string(),
            status: DoctorStatus::Warn,
            detail: format!(
                "These assets look older than {}: {}",
                report_path.display(),
                stale.join(", ")
            ),
        }
    }
}

fn check_next_commands(contents: &str) -> DoctorCheck {
    let commands = extract_voc_commands(contents);
    if commands.is_empty() {
        return DoctorCheck {
            name: "Next commands".to_string(),
            status: DoctorStatus::Warn,
            detail: "No sample voc commands found in AGENTS.md".to_string(),
        };
    }

    let malformed: Vec<String> = commands
        .into_iter()
        .filter(|line| !line.starts_with("voc "))
        .collect();

    if malformed.is_empty() {
        DoctorCheck {
            name: "Next commands".to_string(),
            status: DoctorStatus::Pass,
            detail: "Sample voc commands look valid".to_string(),
        }
    } else {
        DoctorCheck {
            name: "Next commands".to_string(),
            status: DoctorStatus::Fail,
            detail: format!("Malformed commands: {}", malformed.join(" | ")),
        }
    }
}

fn check_next_steps_script(contents: &str, agents_path: &Path) -> DoctorCheck {
    let script_refs: Vec<String> = extract_backticked_paths(contents)
        .into_iter()
        .filter(|item| item.ends_with(".sh"))
        .collect();

    if script_refs.is_empty() {
        return DoctorCheck {
            name: "next-steps.sh".to_string(),
            status: DoctorStatus::Warn,
            detail: "No referenced next-steps.sh script found in AGENTS.md".to_string(),
        };
    }

    let script_path = resolve_reference(agents_path, &script_refs[0]);
    if !script_path.exists() {
        return DoctorCheck {
            name: "next-steps.sh".to_string(),
            status: DoctorStatus::Fail,
            detail: format!("Missing script: {}", script_path.display()),
        };
    }

    let script = match std::fs::read_to_string(&script_path) {
        Ok(script) => script,
        Err(err) => {
            return DoctorCheck {
                name: "next-steps.sh".to_string(),
                status: DoctorStatus::Fail,
                detail: format!("Failed to read {}: {}", script_path.display(), err),
            };
        }
    };

    let commands = extract_script_voc_commands(&script);
    if commands.is_empty() {
        return DoctorCheck {
            name: "next-steps.sh".to_string(),
            status: DoctorStatus::Fail,
            detail: format!("No voc commands found in {}", script_path.display()),
        };
    }

    let malformed: Vec<String> = commands
        .iter()
        .filter(|line| !line.starts_with("voc "))
        .cloned()
        .collect();
    if !malformed.is_empty() {
        return DoctorCheck {
            name: "next-steps.sh".to_string(),
            status: DoctorStatus::Fail,
            detail: format!("Malformed script commands: {}", malformed.join(" | ")),
        };
    }

    let expects_pr_brief = contents.contains("pr-brief.md");
    let expects_pr_comment = contents.contains("pr-comment.md");
    let combined = commands.join("\n");
    let mut missing_flags = Vec::new();
    if expects_pr_brief && !combined.contains("--open-pr-brief") {
        missing_flags.push("--open-pr-brief");
    }
    if expects_pr_comment && !combined.contains("--open-pr-comment") {
        missing_flags.push("--open-pr-comment");
    }

    if !missing_flags.is_empty() {
        return DoctorCheck {
            name: "next-steps.sh".to_string(),
            status: DoctorStatus::Fail,
            detail: format!(
                "Script is missing follow-up flags referenced by AGENTS.md: {}",
                missing_flags.join(", ")
            ),
        };
    }

    DoctorCheck {
        name: "next-steps.sh".to_string(),
        status: DoctorStatus::Pass,
        detail: format!("Shortcut script looks valid: {}", script_path.display()),
    }
}

fn extract_backticked_paths(contents: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut in_tick = false;
    let mut current = String::new();

    for ch in contents.chars() {
        if ch == '`' {
            if in_tick {
                if current.ends_with(".json")
                    || current.ends_with(".md")
                    || current.ends_with(".sh")
                {
                    items.push(current.clone());
                }
                current.clear();
            }
            in_tick = !in_tick;
            continue;
        }
        if in_tick {
            current.push(ch);
        }
    }

    items
}

fn freshness_targets(contents: &str, agents_path: &Path) -> Vec<PathBuf> {
    let mut targets = vec![agents_path.to_path_buf()];
    for item in extract_backticked_paths(contents) {
        let path = resolve_reference(agents_path, &item);
        if !targets.contains(&path) {
            targets.push(path);
        }
    }
    targets
}

fn latest_report_artifact(output_dir: &Path) -> Option<(PathBuf, SystemTime)> {
    ["report.json", "report.sarif"]
        .into_iter()
        .filter_map(|name| {
            let path = output_dir.join(name);
            let modified = std::fs::metadata(&path).ok()?.modified().ok()?;
            Some((path, modified))
        })
        .max_by_key(|(_, modified)| *modified)
}

fn resolve_freshness_source(
    output_dir: &Path,
    freshness_against: Option<&Path>,
) -> Option<(PathBuf, SystemTime)> {
    if let Some(path) = freshness_against {
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            output_dir.join(path)
        };
        let modified = std::fs::metadata(&resolved).ok()?.modified().ok()?;
        return Some((resolved, modified));
    }

    latest_report_artifact(output_dir)
}

pub fn detect_freshness_source_path(
    output_dir: &Path,
    freshness_against: Option<&Path>,
) -> Option<PathBuf> {
    resolve_freshness_source(output_dir, freshness_against).map(|(path, _)| path)
}

fn extract_voc_commands(contents: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut in_block = false;
    for line in contents.lines() {
        if line.trim_start().starts_with("```") {
            in_block = !in_block;
            continue;
        }
        if in_block && line.trim_start().starts_with("voc ") {
            commands.push(line.trim().to_string());
        }
    }
    commands
}

fn extract_script_voc_commands(contents: &str) -> Vec<String> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("voc "))
        .map(ToString::to_string)
        .collect()
}

fn resolve_reference(agents_path: &Path, reference: &str) -> PathBuf {
    let ref_path = PathBuf::from(reference);
    if ref_path.is_absolute() {
        ref_path
    } else {
        agents_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(ref_path)
    }
}

#[cfg(test)]
mod tests {
    use super::{run_doctor, DoctorStatus};
    use crate::agent_assets::AgentAssetLayout;
    use std::fs;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn doctor_warns_when_agents_is_missing() {
        let dir = tempdir().expect("temp dir");
        let layout = AgentAssetLayout::from_output_dir(dir.path());
        let report = run_doctor(None, &layout.agents_path, None);

        assert_eq!(report.checks[1].status, DoctorStatus::Warn);
    }

    #[test]
    fn doctor_fails_when_referenced_assets_are_missing() {
        let dir = tempdir().expect("temp dir");
        let layout = AgentAssetLayout::from_output_dir(dir.path());
        let agents = layout.agents_path.clone();
        fs::write(
            &agents,
            "### Current Project Risks\n\n- Agent bundle: `.verifyos-agent/agent-pack.json` and `.verifyos-agent/agent-pack.md`\n",
        )
        .expect("write agents");

        let report = run_doctor(None, &agents, None);

        assert!(report.has_failures());
        assert_eq!(report.checks[2].status, DoctorStatus::Fail);
    }

    #[test]
    fn doctor_warns_when_assets_are_older_than_report() {
        let dir = tempdir().expect("temp dir");
        let layout = AgentAssetLayout::from_output_dir(dir.path());
        let agents = layout.agents_path.clone();
        let script_dir = layout.agent_bundle_dir.clone();
        fs::create_dir_all(&script_dir).expect("create script dir");
        fs::write(
            script_dir.join("next-steps.sh"),
            "voc --app app.ipa --profile basic\n",
        )
        .expect("write script");
        fs::write(
            &agents,
            "## verifyOS-cli\n\n- Shortcut script: `.verifyos-agent/next-steps.sh`\n",
        )
        .expect("write agents");
        std::thread::sleep(Duration::from_secs(1));
        fs::write(dir.path().join("report.json"), "{}").expect("write report");

        let report = run_doctor(None, &agents, None);

        assert_eq!(report.checks[3].name, "Asset freshness");
        assert_eq!(report.checks[3].status, DoctorStatus::Warn);
        assert!(report.checks[3].detail.contains("report.json"));
    }

    #[test]
    fn doctor_passes_when_assets_are_fresh_against_report() {
        let dir = tempdir().expect("temp dir");
        let layout = AgentAssetLayout::from_output_dir(dir.path());
        let agents = layout.agents_path.clone();
        let script_dir = layout.agent_bundle_dir.clone();
        fs::create_dir_all(&script_dir).expect("create script dir");
        fs::write(dir.path().join("report.sarif"), "{}").expect("write report");
        std::thread::sleep(Duration::from_secs(1));
        fs::write(
            script_dir.join("next-steps.sh"),
            "voc --app app.ipa --profile basic\n",
        )
        .expect("write script");
        fs::write(
            &agents,
            "## verifyOS-cli\n\n- Shortcut script: `.verifyos-agent/next-steps.sh`\n",
        )
        .expect("write agents");

        let report = run_doctor(None, &agents, None);

        assert_eq!(report.checks[3].name, "Asset freshness");
        assert_eq!(report.checks[3].status, DoctorStatus::Pass);
    }

    #[test]
    fn doctor_fails_when_next_steps_script_drifts_from_agents_block() {
        let dir = tempdir().expect("temp dir");
        let layout = AgentAssetLayout::from_output_dir(dir.path());
        let agents = layout.agents_path.clone();
        let script_dir = layout.agent_bundle_dir.clone();
        fs::create_dir_all(&script_dir).expect("create script dir");
        fs::write(
            script_dir.join("next-steps.sh"),
            "#!/usr/bin/env bash\nset -euo pipefail\nvoc --app path/to/app.ipa --profile basic\nvoc doctor --output-dir .verifyos --fix --from-scan path/to/app.ipa --profile basic\n",
        )
        .expect("write script");
        fs::write(
            &agents,
            "## verifyOS-cli\n\n- Shortcut script: `.verifyos-agent/next-steps.sh`\n- PR comment draft: `pr-comment.md`\n",
        )
        .expect("write agents");

        let report = run_doctor(None, &agents, None);

        assert!(report.has_failures());
        assert_eq!(report.checks[5].status, DoctorStatus::Fail);
        assert!(report.checks[5].detail.contains("--open-pr-comment"));
    }

    #[test]
    fn doctor_passes_when_next_steps_script_matches_agents_block() {
        let dir = tempdir().expect("temp dir");
        let layout = AgentAssetLayout::from_output_dir(dir.path());
        let agents = layout.agents_path.clone();
        let script_dir = layout.agent_bundle_dir.clone();
        fs::create_dir_all(&script_dir).expect("create script dir");
        fs::write(&layout.pr_brief_path, "brief").expect("write brief");
        fs::write(&layout.pr_comment_path, "comment").expect("write comment");
        fs::write(
            script_dir.join("next-steps.sh"),
            "#!/usr/bin/env bash\nset -euo pipefail\nvoc --app path/to/app.ipa --profile basic\nvoc doctor --output-dir .verifyos --fix --from-scan path/to/app.ipa --profile basic --open-pr-brief --open-pr-comment\n",
        )
        .expect("write script");
        fs::write(
            &agents,
            "## verifyOS-cli\n\n- Shortcut script: `.verifyos-agent/next-steps.sh`\n- PR brief: `pr-brief.md`\n- PR comment draft: `pr-comment.md`\n",
        )
        .expect("write agents");

        let report = run_doctor(None, &agents, None);

        assert!(!report.has_failures());
        assert_eq!(report.checks[5].status, DoctorStatus::Pass);
    }
}
