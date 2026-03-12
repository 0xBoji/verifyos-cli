use crate::config::load_file_config;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum DoctorStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DoctorCheck {
    pub name: String,
    pub status: DoctorStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    pub fn has_failures(&self) -> bool {
        self.checks
            .iter()
            .any(|item| item.status == DoctorStatus::Fail)
    }
}

pub fn run_doctor(config_path: Option<&Path>, agents_path: &Path) -> DoctorReport {
    let mut checks = Vec::new();

    checks.push(check_config(config_path));
    checks.push(check_agents_presence(agents_path));

    if agents_path.exists() {
        let contents = std::fs::read_to_string(agents_path).unwrap_or_default();
        checks.push(check_referenced_assets(&contents, agents_path));
        checks.push(check_next_commands(&contents));
    }

    DoctorReport { checks }
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
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn doctor_warns_when_agents_is_missing() {
        let dir = tempdir().expect("temp dir");
        let report = run_doctor(None, &dir.path().join("AGENTS.md"));

        assert_eq!(report.checks[1].status, DoctorStatus::Warn);
    }

    #[test]
    fn doctor_fails_when_referenced_assets_are_missing() {
        let dir = tempdir().expect("temp dir");
        let agents = dir.path().join("AGENTS.md");
        fs::write(
            &agents,
            "### Current Project Risks\n\n- Agent bundle: `.verifyos-agent/agent-pack.json` and `.verifyos-agent/agent-pack.md`\n",
        )
        .expect("write agents");

        let report = run_doctor(None, &agents);

        assert!(report.has_failures());
        assert_eq!(report.checks[2].status, DoctorStatus::Fail);
    }
}
