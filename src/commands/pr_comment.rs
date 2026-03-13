use clap::Parser;
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

use verifyos_cli::ci_comment::render_workflow_pr_comment;

#[derive(Debug, Parser)]
pub struct PrCommentArgs {
    /// Output root that contains doctor.json, pr-comment.md, and .verifyos-agent/
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// Optional file path to write the generated comment body
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Scan exit code to include in fallback summaries
    #[arg(long, default_value_t = 0)]
    pub scan_exit: i32,

    /// Doctor exit code to include in fallback summaries
    #[arg(long, default_value_t = 0)]
    pub doctor_exit: i32,

    /// Prefix the body with the sticky comment marker used by GitHub workflows
    #[arg(long)]
    pub sticky_marker: bool,
}

pub fn run(pr_comment: PrCommentArgs) -> Result<()> {
    let output_dir = pr_comment.output_dir.unwrap_or_else(|| PathBuf::from("."));
    let body = render_workflow_pr_comment(
        &output_dir,
        pr_comment.scan_exit,
        pr_comment.doctor_exit,
        pr_comment.sticky_marker,
    )?;
    if let Some(path) = pr_comment.output.as_deref() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).into_diagnostic()?;
        }
        std::fs::write(path, body).into_diagnostic()?;
    } else {
        println!("{body}");
    }
    Ok(())
}
