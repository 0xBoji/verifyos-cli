use clap::Parser;
use miette::{IntoDiagnostic, Result};
use serde::Serialize;
use std::path::PathBuf;

use verifyos_cli::agent_assets::{AgentAssetLayout, HANDOFF_MANIFEST_NAME};
use verifyos_cli::config::FileConfig;
use verifyos_cli::profiles::ScanProfile;

use crate::commands::doctor::{run as run_doctor, DoctorArgs};
use crate::OutputFormat;

#[derive(Debug, Parser)]
pub struct HandoffArgs {
    /// Root directory for generated handoff assets
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// Scan an app and refresh the full handoff bundle
    #[arg(long)]
    pub from_scan: PathBuf,

    /// Baseline JSON report used to keep only new or regressed risks
    #[arg(long)]
    pub baseline: Option<PathBuf>,

    /// Scan profile to use for the handoff refresh
    #[arg(long, value_enum)]
    pub profile: Option<ScanProfile>,

    /// Output format for doctor results
    #[arg(long, value_enum)]
    pub format: Option<OutputFormat>,
}

pub fn run(handoff: HandoffArgs, file_config: &FileConfig) -> Result<()> {
    let output_dir = handoff
        .output_dir
        .clone()
        .or_else(|| {
            file_config
                .doctor
                .as_ref()
                .and_then(|doctor| doctor.output_dir.clone())
        })
        .unwrap_or_else(|| PathBuf::from("."));
    let plan_out = file_config
        .doctor
        .as_ref()
        .and_then(|doctor| doctor.plan_out.clone())
        .unwrap_or_else(|| output_dir.join("repair-plan.md"));
    let layout = AgentAssetLayout::from_output_dir(&output_dir);
    let app_path = handoff.from_scan.clone();
    let baseline_path = handoff.baseline.clone();

    run_doctor(
        DoctorArgs {
            output_dir: Some(output_dir),
            agents: None,
            config: None,
            format: handoff.format,
            fix: true,
            from_scan: Some(app_path.clone()),
            baseline: baseline_path.clone(),
            freshness_against: None,
            profile: handoff.profile,
            open_pr_brief: true,
            open_pr_comment: true,
            repair: Vec::new(),
            plan: true,
            plan_out: Some(plan_out),
        },
        file_config,
    )?;

    let manifest = HandoffManifest {
        app_path: app_path.display().to_string(),
        baseline_path: baseline_path.map(|path| path.display().to_string()),
        profile: handoff
            .profile
            .map(|profile| format!("{profile:?}").to_ascii_lowercase())
            .unwrap_or_else(|| "full".to_string()),
        output_dir: layout.output_dir.display().to_string(),
        assets: vec![
            layout.agents_path.display().to_string(),
            layout.fix_prompt_path.display().to_string(),
            layout.repair_plan_path.display().to_string(),
            layout.pr_brief_path.display().to_string(),
            layout.pr_comment_path.display().to_string(),
            layout.agent_pack_json_path.display().to_string(),
            layout.agent_pack_markdown_path.display().to_string(),
            layout.next_steps_script_path.display().to_string(),
        ],
    };
    let manifest_path = layout.output_dir.join(HANDOFF_MANIFEST_NAME);
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).into_diagnostic()?,
    )
    .into_diagnostic()?;
    Ok(())
}

#[derive(Debug, Serialize)]
struct HandoffManifest {
    app_path: String,
    baseline_path: Option<String>,
    profile: String,
    output_dir: String,
    assets: Vec<String>,
}
