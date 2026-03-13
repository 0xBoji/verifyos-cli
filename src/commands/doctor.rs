use clap::Parser;
use comfy_table::{Cell, Color, Table};
use miette::{IntoDiagnostic, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use verifyos_cli::agent_assets::{build_repair_plan, AgentAssetLayout, RepairPolicy, RepairTarget};
use verifyos_cli::agents::{write_agents_file, CommandHints};
use verifyos_cli::config::FileConfig;
use verifyos_cli::doctor::{
    detect_freshness_source_path, run_doctor, DoctorPlanContext, DoctorReport, DoctorStatus,
};
use verifyos_cli::report::AgentPackFormat;

use crate::{
    empty_agent_pack, infer_existing_command_hints, load_agent_pack, parse_optional_cli_profile,
    parse_optional_output_format, profile_key, run_scan_for_agent_pack, write_agent_pack,
    write_fix_prompt_file, write_next_steps_script, write_pr_brief_file, write_pr_comment_file,
    OutputFormat, Profile,
};

#[derive(Debug, Clone)]
struct DoctorRepairOptions {
    profile: Profile,
    policy: RepairPolicy,
}

#[derive(Debug, Parser)]
pub struct DoctorArgs {
    /// Root directory that contains AGENTS.md and generated init assets
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// Explicit AGENTS.md path to check
    #[arg(long)]
    pub agents: Option<PathBuf>,

    /// Explicit config path to validate
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Output format for doctor results
    #[arg(long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Repair a broken or missing agent setup under the chosen output root
    #[arg(long)]
    pub fix: bool,

    /// Scan an app first and repair agent assets with current findings
    #[arg(long)]
    pub from_scan: Option<PathBuf>,

    /// Baseline JSON report used with --from-scan to keep only new or regressed risks
    #[arg(long)]
    pub baseline: Option<PathBuf>,

    /// Compare asset freshness against this specific report file instead of auto-detecting report.json/report.sarif
    #[arg(long)]
    pub freshness_against: Option<PathBuf>,

    /// Scan profile to use with --from-scan
    #[arg(long, value_enum)]
    pub profile: Option<Profile>,

    /// Generate pr-brief.md for PR review and agent handoff
    #[arg(long)]
    pub open_pr_brief: bool,

    /// Generate pr-comment.md for sticky PR comments or manual GitHub updates
    #[arg(long)]
    pub open_pr_comment: bool,

    /// Only repair selected outputs (repeat or comma-separate)
    #[arg(long, value_enum, value_delimiter = ',', num_args = 1..)]
    pub repair: Vec<RepairTarget>,

    /// Show which assets would be rebuilt for the current fix/repair settings
    #[arg(long)]
    pub plan: bool,
}

pub fn run(doctor: DoctorArgs, file_config: &FileConfig) -> Result<()> {
    let doctor_defaults = file_config.doctor.clone().unwrap_or_default();
    let doctor_format = doctor
        .format
        .or(parse_optional_output_format(
            doctor_defaults.format.as_deref(),
        )?)
        .unwrap_or(OutputFormat::Table);
    let doctor_profile = doctor
        .profile
        .or(parse_optional_cli_profile(
            doctor_defaults.profile.as_deref(),
        )?)
        .unwrap_or(Profile::Full);
    let output_dir = doctor
        .output_dir
        .clone()
        .or(doctor_defaults.output_dir.clone())
        .unwrap_or_else(|| PathBuf::from("."));
    let mut layout = AgentAssetLayout::from_output_dir(&output_dir);
    let agents_path = doctor
        .agents
        .clone()
        .or(doctor_defaults.agents.clone())
        .unwrap_or_else(|| layout.agents_path.clone());
    layout = layout.with_agents_path(&agents_path);
    let repair_targets = if doctor.repair.is_empty() {
        parse_repair_targets(doctor_defaults.repair.as_deref())?
    } else {
        doctor.repair.iter().copied().collect()
    };
    let should_fix =
        doctor.fix || doctor_defaults.fix.unwrap_or(false) || !repair_targets.is_empty();
    let open_pr_brief = doctor.open_pr_brief || doctor_defaults.open_pr_brief.unwrap_or(false);
    let open_pr_comment =
        doctor.open_pr_comment || doctor_defaults.open_pr_comment.unwrap_or(false);
    let freshness_against = doctor
        .freshness_against
        .clone()
        .or(doctor_defaults.freshness_against.clone());

    if should_fix {
        let repair_options = DoctorRepairOptions {
            profile: doctor_profile,
            policy: RepairPolicy::new(repair_targets.clone(), open_pr_brief, open_pr_comment),
        };
        repair_doctor_setup(
            &layout,
            doctor.from_scan.as_deref(),
            doctor.baseline.as_deref(),
            &repair_options,
        )?;
    }

    let mut report = run_doctor(
        doctor.config.as_deref(),
        &layout.agents_path,
        freshness_against.as_deref(),
    );
    if doctor.plan {
        let policy = RepairPolicy::new(repair_targets, open_pr_brief, open_pr_comment);
        report.repair_plan = build_repair_plan(&layout, &policy);
        report.plan_context = Some(build_plan_context(
            &layout,
            doctor.from_scan.as_deref(),
            doctor.baseline.as_deref(),
            freshness_against.as_deref(),
            &policy,
        ));
    }
    render_doctor_report(&report, doctor_format)?;
    if report.has_failures() {
        std::process::exit(1);
    }
    Ok(())
}

fn build_plan_context(
    layout: &AgentAssetLayout,
    from_scan: Option<&Path>,
    baseline_path: Option<&Path>,
    freshness_against: Option<&Path>,
    policy: &RepairPolicy,
) -> DoctorPlanContext {
    let repair_targets = if policy.repairs_all() {
        vec!["all".to_string()]
    } else {
        let mut targets: Vec<String> = policy
            .repair_targets()
            .iter()
            .copied()
            .map(|target| target.key().to_string())
            .collect();
        targets.sort();
        targets
    };

    DoctorPlanContext {
        source: if from_scan.is_some() {
            "fresh-scan".to_string()
        } else {
            "existing-assets".to_string()
        },
        scan_artifact: from_scan.map(|path| path.display().to_string()),
        baseline_path: baseline_path.map(|path| path.display().to_string()),
        freshness_source: detect_freshness_source_path(&layout.output_dir, freshness_against)
            .map(|path| path.display().to_string()),
        repair_targets,
    }
}

fn parse_repair_targets(raw: Option<&[String]>) -> Result<HashSet<RepairTarget>> {
    let mut targets = HashSet::new();
    for value in raw.unwrap_or_default() {
        targets.insert(parse_repair_target(value)?);
    }
    Ok(targets)
}

fn parse_repair_target(value: &str) -> Result<RepairTarget> {
    match value.to_ascii_lowercase().as_str() {
        "agents" => Ok(RepairTarget::Agents),
        "agent-bundle" => Ok(RepairTarget::AgentBundle),
        "fix-prompt" => Ok(RepairTarget::FixPrompt),
        "pr-brief" => Ok(RepairTarget::PrBrief),
        "pr-comment" => Ok(RepairTarget::PrComment),
        _ => Err(miette::miette!(
            "Unknown repair target `{value}`. Expected one of: agents, agent-bundle, fix-prompt, pr-brief, pr-comment"
        )),
    }
}

fn repair_doctor_setup(
    layout: &AgentAssetLayout,
    from_scan: Option<&Path>,
    baseline_path: Option<&Path>,
    options: &DoctorRepairOptions,
) -> Result<()> {
    std::fs::create_dir_all(&layout.output_dir).into_diagnostic()?;
    std::fs::create_dir_all(&layout.agent_bundle_dir).into_diagnostic()?;

    let inferred_hints = infer_existing_command_hints(layout);
    let should_open_pr_brief = options.policy.open_pr_brief
        || inferred_hints.pr_brief_path.is_some()
        || options.policy.should_repair_pr_brief();
    let should_open_pr_comment = options.policy.open_pr_comment
        || inferred_hints.pr_comment_path.is_some()
        || options.policy.should_repair_pr_comment();

    let pack = if let Some(app_path) = from_scan {
        run_scan_for_agent_pack(app_path, options.profile, baseline_path)?
    } else {
        load_agent_pack(&layout.agent_pack_json_path).unwrap_or_else(empty_agent_pack)
    };

    let command_hints = CommandHints {
        output_dir: Some(layout.output_dir.display().to_string()),
        app_path: Some(
            from_scan
                .map(|path| path.display().to_string())
                .or(inferred_hints.app_path)
                .unwrap_or_else(|| "<path-to-.ipa-or-.app>".to_string()),
        ),
        baseline_path: baseline_path
            .map(|path| path.display().to_string())
            .or(inferred_hints.baseline_path),
        agent_pack_dir: Some(layout.agent_bundle_dir.display().to_string()),
        profile: Some(
            from_scan
                .map(|_| profile_key(options.profile))
                .or(inferred_hints.profile)
                .unwrap_or_else(|| profile_key(options.profile)),
        ),
        shell_script: true,
        fix_prompt_path: Some(layout.fix_prompt_path.display().to_string()),
        pr_brief_path: should_open_pr_brief.then(|| layout.pr_brief_path.display().to_string()),
        pr_comment_path: should_open_pr_comment
            .then(|| layout.pr_comment_path.display().to_string()),
    };

    if options.policy.should_repair_agents() {
        write_agents_file(
            &layout.agents_path,
            Some(&pack),
            Some(&layout.agent_bundle_dir),
            Some(&command_hints),
        )?;
    }
    if options.policy.should_repair_bundle() {
        write_agent_pack(&layout.agent_bundle_dir, &pack, AgentPackFormat::Bundle)?;
        write_next_steps_script(&layout.next_steps_script_path, &command_hints)?;
    }
    if options.policy.should_repair_fix_prompt() {
        write_fix_prompt_file(&layout.fix_prompt_path, &pack, &command_hints)?;
    }
    if options.policy.should_repair_pr_brief() && should_open_pr_brief {
        write_pr_brief_file(&layout.pr_brief_path, &pack, &command_hints)?;
    }
    if options.policy.should_repair_pr_comment() && should_open_pr_comment {
        write_pr_comment_file(&layout.pr_comment_path, &pack, &command_hints)?;
    }

    Ok(())
}

fn render_doctor_report(report: &DoctorReport, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Table => {
            let mut table = Table::new();
            table.set_header(vec!["Check", "Status", "Detail"]);
            for item in &report.checks {
                let status = match item.status {
                    DoctorStatus::Pass => Cell::new("PASS").fg(Color::Green),
                    DoctorStatus::Warn => Cell::new("WARN").fg(Color::Yellow),
                    DoctorStatus::Fail => Cell::new("FAIL").fg(Color::Red),
                };
                table.add_row(vec![Cell::new(&item.name), status, Cell::new(&item.detail)]);
            }
            println!("{table}");
            if let Some(plan_context) = &report.plan_context {
                println!("\nPlan context:");
                println!("- source: {}", plan_context.source);
                if let Some(scan_artifact) = &plan_context.scan_artifact {
                    println!("- scan artifact: {scan_artifact}");
                }
                if let Some(baseline_path) = &plan_context.baseline_path {
                    println!("- baseline: {baseline_path}");
                }
                if let Some(freshness_source) = &plan_context.freshness_source {
                    println!("- freshness source: {freshness_source}");
                }
                println!(
                    "- repair targets: {}",
                    plan_context.repair_targets.join(", ")
                );
            }
            if !report.repair_plan.is_empty() {
                println!("\nRepair plan:");
                for item in &report.repair_plan {
                    println!("- {} -> {} ({})", item.target, item.path, item.reason);
                }
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(report).into_diagnostic()?
            );
        }
        OutputFormat::Sarif => {
            return Err(miette::miette!(
                "`doctor` supports only table or json output"
            ));
        }
    }
    Ok(())
}
