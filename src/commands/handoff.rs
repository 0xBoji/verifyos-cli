use clap::Parser;
use miette::Result;
use std::path::PathBuf;

use verifyos_cli::config::FileConfig;

use crate::commands::doctor::{run as run_doctor, DoctorArgs};
use crate::{OutputFormat, Profile};

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
    pub profile: Option<Profile>,

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

    run_doctor(
        DoctorArgs {
            output_dir: Some(output_dir),
            agents: None,
            config: None,
            format: handoff.format,
            fix: true,
            from_scan: Some(handoff.from_scan),
            baseline: handoff.baseline,
            freshness_against: None,
            profile: handoff.profile,
            open_pr_brief: true,
            open_pr_comment: true,
            repair: Vec::new(),
            plan: true,
            plan_out: Some(plan_out),
        },
        file_config,
    )
}
