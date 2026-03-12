use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use miette::{IntoDiagnostic, Result};
use std::collections::HashSet;
use std::path::PathBuf;

use verifyos_cli::config::{load_file_config, resolve_runtime_config, CliOverrides};
use verifyos_cli::core::engine::Engine;
use verifyos_cli::profiles::{
    available_rule_ids, normalize_rule_id, register_rules, RuleSelection, ScanProfile,
};
use verifyos_cli::report::{
    apply_baseline, build_report, render_json, render_markdown, render_sarif, render_table,
    should_exit_with_failure, FailOn, TimingMode,
};

#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Sarif,
}

#[derive(Clone, Debug, ValueEnum)]
enum Profile {
    Basic,
    Full,
}

#[derive(Clone, Debug, ValueEnum)]
enum FailOnLevel {
    Off,
    Error,
    Warning,
}

#[derive(Clone, Debug, ValueEnum)]
enum TimingLevel {
    Summary,
    Full,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the iOS App Bundle (.ipa or .app)
    #[arg(short, long)]
    app: PathBuf,

    /// Optional config file path. If omitted, verifyos.toml is used when present
    #[arg(long)]
    config: Option<PathBuf>,

    /// Output format: table, json, sarif
    #[arg(long, value_enum)]
    format: Option<OutputFormat>,

    /// Baseline JSON file to suppress existing findings
    #[arg(long)]
    baseline: Option<PathBuf>,

    /// Write a clean Markdown report to a file (agent-friendly)
    #[arg(long)]
    md_out: Option<PathBuf>,

    /// Scan profile: basic or full
    #[arg(long, value_enum)]
    profile: Option<Profile>,

    /// Exit with code 1 when findings reach this severity threshold
    #[arg(long, value_enum)]
    fail_on: Option<FailOnLevel>,

    /// Show timing telemetry: summary or full (defaults to summary when flag is present)
    #[arg(long, value_enum, num_args = 0..=1, default_missing_value = "summary")]
    timings: Option<TimingLevel>,

    /// Only run the listed rule IDs (repeat or comma-separate)
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    include: Vec<String>,

    /// Skip the listed rule IDs (repeat or comma-separate)
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    exclude: Vec<String>,
}

fn main() -> Result<()> {
    // 1. Parse CLI arguments
    let args = Args::parse();
    let file_config = load_file_config(args.config.as_deref())?;
    let runtime = resolve_runtime_config(
        file_config,
        CliOverrides {
            format: args.format.map(output_format_key),
            baseline: args.baseline.clone(),
            md_out: args.md_out.clone(),
            profile: args.profile.map(profile_key),
            fail_on: args.fail_on.map(fail_on_key),
            timings: args.timings.map(timing_key),
            include: args.include.clone(),
            exclude: args.exclude.clone(),
        },
    );
    let output_format = parse_output_format(&runtime.format)?;
    let profile = parse_profile(&runtime.profile)?;
    let fail_on = parse_fail_on(&runtime.fail_on)?;
    let timing_mode = parse_timing_mode(&runtime.timings)?;

    // 2. Initialize spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .into_diagnostic()?,
    );
    pb.set_message("Analyzing app bundle...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // 3. Initialize Core Engine
    let mut engine = Engine::new();
    let selection = build_rule_selection(profile, &runtime.include, &runtime.exclude)?;
    register_rules(&mut engine, profile, &selection);

    // 4. Run the Engine
    let run = engine
        .run(&args.app)
        .map_err(|e| miette::miette!("Engine orchestrator failed: {}", e))?;

    // 5. Stop the spinner
    pb.finish_with_message("Analysis complete!");

    // 6. Build report and apply baseline (if any)
    let mut report = build_report(run.results, run.total_duration_ms, run.cache_stats);
    let mut suppressed = None;
    if let Some(path) = runtime.baseline {
        let baseline_raw = std::fs::read_to_string(path).into_diagnostic()?;
        let baseline: verifyos_cli::report::ReportData =
            serde_json::from_str(&baseline_raw).into_diagnostic()?;
        let summary = apply_baseline(&mut report, &baseline);
        suppressed = Some(summary.suppressed);
    }

    // 7. Render output
    match output_format {
        OutputFormat::Table => println!("{}", render_table(&report, timing_mode)),
        OutputFormat::Json => println!("{}", render_json(&report).into_diagnostic()?),
        OutputFormat::Sarif => println!("{}", render_sarif(&report).into_diagnostic()?),
    }

    if let Some(path) = runtime.md_out {
        let markdown = render_markdown(&report, suppressed, timing_mode);
        std::fs::write(path, markdown).into_diagnostic()?;
    }

    // 8. Exit with code 1 if findings meet the configured threshold
    if should_exit_with_failure(&report, fail_on) {
        std::process::exit(1);
    }

    Ok(())
}

fn output_format_key(value: OutputFormat) -> String {
    match value {
        OutputFormat::Table => "table".to_string(),
        OutputFormat::Json => "json".to_string(),
        OutputFormat::Sarif => "sarif".to_string(),
    }
}

fn profile_key(value: Profile) -> String {
    match value {
        Profile::Basic => "basic".to_string(),
        Profile::Full => "full".to_string(),
    }
}

fn fail_on_key(value: FailOnLevel) -> String {
    match value {
        FailOnLevel::Off => "off".to_string(),
        FailOnLevel::Error => "error".to_string(),
        FailOnLevel::Warning => "warning".to_string(),
    }
}

fn timing_key(value: TimingLevel) -> String {
    match value {
        TimingLevel::Summary => "summary".to_string(),
        TimingLevel::Full => "full".to_string(),
    }
}

fn parse_output_format(value: &str) -> Result<OutputFormat> {
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

fn parse_profile(value: &str) -> Result<ScanProfile> {
    match value.to_ascii_lowercase().as_str() {
        "basic" => Ok(ScanProfile::Basic),
        "full" => Ok(ScanProfile::Full),
        _ => Err(miette::miette!(
            "Unknown profile `{}`. Expected one of: basic, full",
            value
        )),
    }
}

fn parse_fail_on(value: &str) -> Result<FailOn> {
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

fn parse_timing_mode(value: &str) -> Result<TimingMode> {
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

fn build_rule_selection(
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
        let rule_id = normalize_rule_id(value);
        if !available.contains(&rule_id) {
            let mut available_ids: Vec<&str> = available.iter().map(String::as_str).collect();
            available_ids.sort_unstable();
            return Err(miette::miette!(
                "{flag_name} contains unknown rule ID `{}`. Available rule IDs for this profile: {}",
                value,
                available_ids.join(", ")
            ));
        }
        normalized.insert(rule_id);
    }

    Ok(normalized)
}
