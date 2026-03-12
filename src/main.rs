use clap::{Parser, Subcommand, ValueEnum};
use comfy_table::Table;
use indicatif::{ProgressBar, ProgressStyle};
use miette::{IntoDiagnostic, Result};
use std::collections::HashSet;
use std::path::PathBuf;

use verifyos_cli::agents::write_agents_file;
use verifyos_cli::config::{load_file_config, resolve_runtime_config, CliOverrides};
use verifyos_cli::core::engine::Engine;
use verifyos_cli::profiles::{
    available_rule_ids, normalize_rule_id, register_rules, rule_detail, rule_inventory,
    RuleDetailItem, RuleInventoryItem, RuleSelection, ScanProfile,
};
use verifyos_cli::report::{
    apply_agent_pack_baseline, apply_baseline, build_agent_pack, build_report,
    render_agent_pack_markdown, render_json, render_markdown, render_sarif, render_table,
    should_exit_with_failure, AgentPackFormat, FailOn, TimingMode,
};

const HELP_BANNER: &str = r#"
██    ██  ██████   ██████
██    ██ ██    ██ ██
██    ██ ██    ██ ██
 ██  ██  ██    ██ ██
  ████    ██████   ██████

verify-OS
"#;

#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Sarif,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
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

#[derive(Clone, Debug, ValueEnum)]
enum AgentPackOutput {
    Json,
    Markdown,
    Bundle,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None,
    before_help = HELP_BANNER,
    subcommand_negates_reqs = true
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the iOS App Bundle (.ipa or .app)
    #[arg(short, long, required_unless_present_any = ["list_rules", "show_rule"])]
    app: Option<PathBuf>,

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

    /// Write a machine-readable fix pack for AI agents
    #[arg(long)]
    agent_pack: Option<PathBuf>,

    /// Agent pack output format: json, markdown, bundle
    #[arg(long, value_enum)]
    agent_pack_format: Option<AgentPackOutput>,

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

    /// List all available rules and exit
    #[arg(long)]
    list_rules: bool,

    /// Show details for a single rule ID and exit
    #[arg(long)]
    show_rule: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create or update AGENTS.md with verifyOS-cli guidance
    Init(InitArgs),
}

#[derive(Debug, Parser)]
struct InitArgs {
    /// Path to the AGENTS.md file to create or update
    #[arg(long, default_value = "AGENTS.md")]
    path: PathBuf,

    /// Scan an app first and inject current project risks into the managed block
    #[arg(long)]
    from_scan: Option<PathBuf>,

    /// Baseline JSON report used to keep only new or regressed risks in Current Project Risks
    #[arg(long)]
    baseline: Option<PathBuf>,

    /// Scan profile to use with --from-scan
    #[arg(long, value_enum, default_value = "full")]
    profile: Profile,
}

fn main() -> Result<()> {
    // 1. Parse CLI arguments
    let args = Args::parse();
    if let Some(Commands::Init(init)) = args.command {
        let agent_pack = if let Some(app) = init.from_scan.as_deref() {
            Some(run_scan_for_agent_pack(
                app,
                init.profile,
                init.baseline.as_deref(),
            )?)
        } else {
            None
        };
        write_agents_file(&init.path, agent_pack.as_ref())?;
        println!("Updated {}", init.path.display());
        return Ok(());
    }

    let file_config = load_file_config(args.config.as_deref())?;
    let runtime = resolve_runtime_config(
        file_config,
        CliOverrides {
            format: args.format.map(output_format_key),
            baseline: args.baseline.clone(),
            md_out: args.md_out.clone(),
            agent_pack: args.agent_pack.clone(),
            agent_pack_format: args.agent_pack_format.map(agent_pack_format_key),
            profile: args.profile.map(profile_key),
            fail_on: args.fail_on.map(fail_on_key),
            timings: args.timings.map(timing_key),
            include: args.include.clone(),
            exclude: args.exclude.clone(),
        },
    );
    let output_format = parse_output_format(&runtime.format)?;
    if args.list_rules {
        render_rule_inventory(output_format)?;
        return Ok(());
    }
    if let Some(rule_id) = args.show_rule.as_deref() {
        render_rule_detail(rule_id, output_format)?;
        return Ok(());
    }
    let profile = parse_profile(&runtime.profile)?;
    let fail_on = parse_fail_on(&runtime.fail_on)?;
    let timing_mode = parse_timing_mode(&runtime.timings)?;
    let agent_pack_format = parse_agent_pack_format(&runtime.agent_pack_format)?;

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
        .run(args.app.expect("app is required unless list-rules"))
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

    if let Some(path) = runtime.agent_pack {
        let agent_pack = build_agent_pack(&report);
        write_agent_pack(&path, &agent_pack, agent_pack_format)?;
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

fn agent_pack_format_key(value: AgentPackOutput) -> String {
    match value {
        AgentPackOutput::Json => "json".to_string(),
        AgentPackOutput::Markdown => "markdown".to_string(),
        AgentPackOutput::Bundle => "bundle".to_string(),
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

fn scan_profile_from_cli(value: Profile) -> ScanProfile {
    match value {
        Profile::Basic => ScanProfile::Basic,
        Profile::Full => ScanProfile::Full,
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

fn parse_agent_pack_format(value: &str) -> Result<AgentPackFormat> {
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

fn write_agent_pack(
    path: &std::path::Path,
    agent_pack: &verifyos_cli::report::AgentPack,
    format: AgentPackFormat,
) -> Result<()> {
    match format {
        AgentPackFormat::Json => {
            let json = serde_json::to_string_pretty(agent_pack).into_diagnostic()?;
            std::fs::write(path, json).into_diagnostic()?;
        }
        AgentPackFormat::Markdown => {
            let markdown = render_agent_pack_markdown(agent_pack);
            std::fs::write(path, markdown).into_diagnostic()?;
        }
        AgentPackFormat::Bundle => {
            std::fs::create_dir_all(path).into_diagnostic()?;
            let json_path = path.join("agent-pack.json");
            let markdown_path = path.join("agent-pack.md");
            let json = serde_json::to_string_pretty(agent_pack).into_diagnostic()?;
            let markdown = render_agent_pack_markdown(agent_pack);
            std::fs::write(json_path, json).into_diagnostic()?;
            std::fs::write(markdown_path, markdown).into_diagnostic()?;
        }
    }

    Ok(())
}

fn run_scan_for_agent_pack(
    app_path: &std::path::Path,
    profile: Profile,
    baseline_path: Option<&std::path::Path>,
) -> Result<verifyos_cli::report::AgentPack> {
    let mut engine = Engine::new();
    let selection = RuleSelection::default();
    let profile = scan_profile_from_cli(profile);
    register_rules(&mut engine, profile, &selection);

    let run = engine
        .run(app_path)
        .map_err(|e| miette::miette!("Engine orchestrator failed: {}", e))?;
    let report = build_report(run.results, run.total_duration_ms, run.cache_stats);
    let mut agent_pack = build_agent_pack(&report);
    if let Some(path) = baseline_path {
        let baseline_raw = std::fs::read_to_string(path).into_diagnostic()?;
        let baseline: verifyos_cli::report::ReportData =
            serde_json::from_str(&baseline_raw).into_diagnostic()?;
        apply_agent_pack_baseline(&mut agent_pack, &baseline);
    }
    Ok(agent_pack)
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

fn render_rule_inventory(output_format: OutputFormat) -> Result<()> {
    let inventory = rule_inventory();
    match output_format {
        OutputFormat::Table => {
            println!("{}", render_rule_inventory_table(&inventory));
            Ok(())
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&inventory).into_diagnostic()?
            );
            Ok(())
        }
        OutputFormat::Sarif => Err(miette::miette!(
            "`--list-rules` supports only table or json output"
        )),
    }
}

fn render_rule_inventory_table(items: &[RuleInventoryItem]) -> String {
    let mut table = Table::new();
    table.set_header(vec![
        "Rule ID",
        "Name",
        "Category",
        "Severity",
        "Default Profiles",
    ]);

    for item in items {
        table.add_row(vec![
            item.rule_id.clone(),
            item.name.clone(),
            format!("{:?}", item.category),
            format!("{:?}", item.severity),
            item.default_profiles.join(", "),
        ]);
    }

    table.to_string()
}

fn render_rule_detail(rule_id: &str, output_format: OutputFormat) -> Result<()> {
    let Some(detail) = rule_detail(rule_id) else {
        return Err(miette::miette!("Unknown rule ID `{}`", rule_id));
    };

    match output_format {
        OutputFormat::Table => {
            println!("{}", render_rule_detail_table(&detail));
            Ok(())
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&detail).into_diagnostic()?
            );
            Ok(())
        }
        OutputFormat::Sarif => Err(miette::miette!(
            "`--show-rule` supports only table or json output"
        )),
    }
}

fn render_rule_detail_table(item: &RuleDetailItem) -> String {
    let mut table = Table::new();
    table.set_header(vec!["Field", "Value"]);
    table.add_row(vec!["Rule ID", item.rule_id.as_str()]);
    table.add_row(vec!["Name", item.name.as_str()]);
    table.add_row(vec!["Category", &format!("{:?}", item.category)]);
    table.add_row(vec!["Severity", &format!("{:?}", item.severity)]);
    table.add_row(vec!["Default Profiles", &item.default_profiles.join(", ")]);
    table.add_row(vec!["Recommendation", item.recommendation.as_str()]);
    table.to_string()
}
