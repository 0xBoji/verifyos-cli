use crate::{AgentPackOutput, FailOnLevel, OutputFormat, TimingLevel};
use miette::Result;
use std::collections::HashSet;
use verifyos_cli::profiles::{available_rule_ids, normalize_rule_id, RuleSelection, ScanProfile};
use verifyos_cli::report::{AgentPackFormat, FailOn, TimingMode};

pub fn output_format_key(value: OutputFormat) -> String {
    match value {
        OutputFormat::Table => "table".to_string(),
        OutputFormat::Json => "json".to_string(),
        OutputFormat::Sarif => "sarif".to_string(),
    }
}

pub fn profile_key(value: ScanProfile) -> String {
    match value {
        ScanProfile::Basic => "basic".to_string(),
        ScanProfile::Full => "full".to_string(),
    }
}

pub fn fail_on_key(value: FailOnLevel) -> String {
    match value {
        FailOnLevel::Off => "off".to_string(),
        FailOnLevel::Error => "error".to_string(),
        FailOnLevel::Warning => "warning".to_string(),
    }
}

pub fn timing_key(value: TimingLevel) -> String {
    match value {
        TimingLevel::Summary => "summary".to_string(),
        TimingLevel::Full => "full".to_string(),
    }
}

pub fn agent_pack_format_key(value: AgentPackOutput) -> String {
    match value {
        AgentPackOutput::Json => "json".to_string(),
        AgentPackOutput::Markdown => "markdown".to_string(),
        AgentPackOutput::Bundle => "bundle".to_string(),
    }
}

pub fn parse_output_format(value: &str) -> Result<OutputFormat> {
    match value.to_ascii_lowercase().as_str() {
        "table" => Ok(OutputFormat::Table),
        "json" => Ok(OutputFormat::Json),
        "sarif" => Ok(OutputFormat::Sarif),
        _ => Err(miette::miette!(
            "Unknown output format `{}`. Expected one of: table, json, sarif",
            value
        )),
    }
}

pub fn parse_profile(value: &str) -> Result<ScanProfile> {
    match value.to_ascii_lowercase().as_str() {
        "basic" => Ok(ScanProfile::Basic),
        "full" => Ok(ScanProfile::Full),
        _ => Err(miette::miette!(
            "Unknown profile `{}`. Expected one of: basic, full",
            value
        )),
    }
}

pub fn parse_optional_cli_profile(value: Option<&str>) -> Result<Option<ScanProfile>> {
    value.map(parse_profile).transpose()
}

pub fn parse_fail_on(value: &str) -> Result<FailOn> {
    match value.to_ascii_lowercase().as_str() {
        "off" => Ok(FailOn::Off),
        "error" => Ok(FailOn::Error),
        "warning" => Ok(FailOn::Warning),
        _ => Err(miette::miette!(
            "Unknown fail-on threshold `{}`. Expected one of: off, error, warning",
            value
        )),
    }
}

pub fn parse_timing_mode(value: &str) -> Result<TimingMode> {
    match value.to_ascii_lowercase().as_str() {
        "off" => Ok(TimingMode::Off),
        "summary" => Ok(TimingMode::Summary),
        "full" => Ok(TimingMode::Full),
        _ => Err(miette::miette!(
            "Unknown timings mode `{}`. Expected one of: off, summary, full",
            value
        )),
    }
}

pub fn parse_agent_pack_format(value: &str) -> Result<AgentPackFormat> {
    match value.to_ascii_lowercase().as_str() {
        "json" => Ok(AgentPackFormat::Json),
        "markdown" => Ok(AgentPackFormat::Markdown),
        "bundle" => Ok(AgentPackFormat::Bundle),
        _ => Err(miette::miette!(
            "Unknown agent pack format `{}`. Expected one of: json, markdown, bundle",
            value
        )),
    }
}

pub fn parse_optional_output_format(value: Option<&str>) -> Result<Option<OutputFormat>> {
    value.map(parse_output_format).transpose()
}

pub fn build_rule_selection(
    profile: ScanProfile,
    include: &[String],
    exclude: &[String],
) -> Result<RuleSelection> {
    let available: HashSet<String> = available_rule_ids(profile).into_iter().collect();
    let include = normalize_requested_rules(include, &available, "--include")?;
    let exclude = normalize_requested_rules(exclude, &available, "--exclude")?;

    Ok(RuleSelection { include, exclude })
}

fn normalize_requested_rules(
    values: &[String],
    available: &HashSet<String>,
    flag_name: &str,
) -> Result<HashSet<String>> {
    let mut normalized = HashSet::new();
    for value in values {
        let normalized_id = normalize_rule_id(value);
        if !available.contains(&normalized_id) {
            let mut valid = available.iter().cloned().collect::<Vec<_>>();
            valid.sort();
            return Err(miette::miette!(
                "Rule `{}` is not available for this profile via {}. Valid values: {}",
                normalized_id,
                flag_name,
                valid.join(", ")
            ));
        }
        normalized.insert(normalized_id);
    }
    Ok(normalized)
}
