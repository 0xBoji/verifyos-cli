use clap::{Parser, Subcommand, ValueEnum};
use comfy_table::Table;
use indicatif::{ProgressBar, ProgressStyle};
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

mod commands;

use commands::analyze_size::{run as run_analyze_size_command, AnalyzeSizeArgs};
use commands::doctor::{run as run_doctor_command, DoctorArgs};
use commands::handoff::{run as run_handoff_command, HandoffArgs};
use commands::init::{run as run_init_command, InitArgs};
use commands::lsp::{run as run_lsp_command, LspArgs};
use commands::pr_comment::{run as run_pr_comment_command, PrCommentArgs};
use commands::support::{
    agent_pack_format_key, build_rule_selection, fail_on_key, output_format_key,
    parse_agent_pack_format, parse_fail_on, parse_output_format, parse_profile, parse_timing_mode,
    profile_key, timing_key,
};

use verifyos_cli::config::{load_file_config, resolve_runtime_config, CliOverrides};
use verifyos_cli::core::engine::Engine;
use verifyos_cli::profiles::{
    register_rules, rule_detail, rule_inventory, RuleDetailItem, RuleInventoryItem, RuleSelection,
    ScanProfile,
};
use verifyos_cli::report::{
    apply_agent_pack_baseline, apply_baseline, build_agent_pack, build_report,
    render_agent_pack_markdown, render_json, render_markdown, render_sarif, render_table,
    should_exit_with_failure, AgentPackFormat,
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
    profile: Option<ScanProfile>,

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
    /// Inspect IPA/app bundle size hotspots and category breakdowns
    AnalyzeSize(AnalyzeSizeArgs),
    /// Verify verifyOS-cli config and generated agent assets
    Doctor(DoctorArgs),
    /// Refresh the full agent handoff bundle from a fresh scan
    Handoff(HandoffArgs),
    /// Render a sticky PR comment body from an output directory
    PrComment(PrCommentArgs),
    /// Start the verifyOS Language Server (LSP)
    Lsp(LspArgs),
}

fn main() -> Result<()> {
    // 1. Parse CLI arguments
    let args = Args::parse();
    let file_config = load_file_config(args.config.as_deref())?;
    if let Some(Commands::Init(init)) = args.command {
        return run_init_command(init, &file_config);
    }
    if let Some(Commands::AnalyzeSize(analyze_size)) = args.command {
        return run_analyze_size_command(analyze_size);
    }
    if let Some(Commands::Doctor(doctor)) = args.command {
        return run_doctor_command(doctor, &file_config);
    }
    if let Some(Commands::Handoff(handoff)) = args.command {
        return run_handoff_command(handoff, &file_config);
    }
    if let Some(Commands::PrComment(pr_comment)) = args.command {
        return run_pr_comment_command(pr_comment);
    }
    if let Some(Commands::Lsp(lsp)) = args.command {
        return run_lsp_command(lsp);
    }

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
    profile: ScanProfile,
    baseline_path: Option<&std::path::Path>,
) -> Result<verifyos_cli::report::AgentPack> {
    let mut engine = Engine::new();
    let selection = RuleSelection::default();
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
        "Default ScanProfiles",
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
    table.add_row(vec![
        "Default ScanProfiles",
        &item.default_profiles.join(", "),
    ]);
    table.add_row(vec!["Recommendation", item.recommendation.as_str()]);
    table.to_string()
}
