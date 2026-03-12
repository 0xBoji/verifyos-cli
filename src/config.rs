use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct FileConfig {
    pub format: Option<String>,
    pub baseline: Option<PathBuf>,
    pub md_out: Option<PathBuf>,
    pub agent_pack: Option<PathBuf>,
    pub agent_pack_format: Option<String>,
    pub profile: Option<String>,
    pub fail_on: Option<String>,
    pub timings: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CliOverrides {
    pub format: Option<String>,
    pub baseline: Option<PathBuf>,
    pub md_out: Option<PathBuf>,
    pub agent_pack: Option<PathBuf>,
    pub agent_pack_format: Option<String>,
    pub profile: Option<String>,
    pub fail_on: Option<String>,
    pub timings: Option<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub format: String,
    pub baseline: Option<PathBuf>,
    pub md_out: Option<PathBuf>,
    pub agent_pack: Option<PathBuf>,
    pub agent_pack_format: String,
    pub profile: String,
    pub fail_on: String,
    pub timings: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

pub fn load_file_config(config_path: Option<&Path>) -> Result<FileConfig, miette::Report> {
    let Some(path) = resolve_config_path(config_path) else {
        return Ok(FileConfig::default());
    };

    let raw = std::fs::read_to_string(&path)
        .map_err(|err| miette::miette!("Failed to read config file {}: {}", path.display(), err))?;
    toml::from_str(&raw)
        .map_err(|err| miette::miette!("Failed to parse config file {}: {}", path.display(), err))
}

pub fn resolve_runtime_config(file: FileConfig, cli: CliOverrides) -> RuntimeConfig {
    RuntimeConfig {
        format: cli
            .format
            .or(file.format)
            .unwrap_or_else(|| "table".to_string()),
        baseline: cli.baseline.or(file.baseline),
        md_out: cli.md_out.or(file.md_out),
        agent_pack: cli.agent_pack.or(file.agent_pack),
        agent_pack_format: cli
            .agent_pack_format
            .or(file.agent_pack_format)
            .unwrap_or_else(|| "json".to_string()),
        profile: cli
            .profile
            .or(file.profile)
            .unwrap_or_else(|| "full".to_string()),
        fail_on: cli
            .fail_on
            .or(file.fail_on)
            .unwrap_or_else(|| "error".to_string()),
        timings: cli
            .timings
            .or(file.timings)
            .unwrap_or_else(|| "off".to_string()),
        include: if cli.include.is_empty() {
            file.include.unwrap_or_default()
        } else {
            cli.include
        },
        exclude: if cli.exclude.is_empty() {
            file.exclude.unwrap_or_default()
        } else {
            cli.exclude
        },
    }
}

fn resolve_config_path(config_path: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = config_path {
        return Some(path.to_path_buf());
    }

    let default = PathBuf::from("verifyos.toml");
    if default.exists() {
        Some(default)
    } else {
        None
    }
}
