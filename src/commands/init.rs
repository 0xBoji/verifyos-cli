use clap::Parser;
use miette::Result;
use std::path::{Path, PathBuf};

use verifyos_cli::agent_assets::AgentAssetLayout;
use verifyos_cli::agent_io::{write_agent_pack, write_fix_prompt_file, write_next_steps_script};
use verifyos_cli::agents::{write_agents_file, CommandHints};
use verifyos_cli::config::FileConfig;
use verifyos_cli::report::{AgentPack, AgentPackFormat};

use crate::commands::support::{parse_optional_cli_profile, profile_key};
use crate::run_scan_for_agent_pack;
use verifyos_cli::profiles::ScanProfile;

#[derive(Debug, Parser)]
pub struct InitArgs {
    /// Root directory for generated init assets like AGENTS.md, agent bundle, script, and prompt
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// Path to the AGENTS.md file to create or update
    #[arg(long)]
    pub path: Option<PathBuf>,

    /// Scan an app first and inject current project risks into the managed block
    #[arg(long)]
    pub from_scan: Option<PathBuf>,

    /// Baseline JSON report used to keep only new or regressed risks in Current Project Risks
    #[arg(long)]
    pub baseline: Option<PathBuf>,

    /// Generate agent-pack.json and agent-pack.md into this directory during init
    #[arg(long)]
    pub agent_pack_dir: Option<PathBuf>,

    /// Write copy-paste follow-up commands into AGENTS.md
    #[arg(long)]
    pub write_commands: bool,

    /// Generate next-steps.sh inside --agent-pack-dir with follow-up commands
    #[arg(long)]
    pub shell_script: bool,

    /// Generate fix-prompt.md for AI agents
    #[arg(long)]
    pub fix_prompt: bool,

    /// Scan profile to use with --from-scan
    #[arg(long, value_enum)]
    pub profile: Option<ScanProfile>,
}

pub fn run(init: InitArgs, file_config: &FileConfig) -> Result<()> {
    let init_defaults = file_config.init.clone().unwrap_or_default();
    let init_profile = init
        .profile
        .or(parse_optional_cli_profile(
            init_defaults.profile.as_deref(),
        )?)
        .unwrap_or(ScanProfile::Full);
    let effective_output_dir = init
        .output_dir
        .clone()
        .or(init_defaults.output_dir.clone())
        .unwrap_or_else(|| PathBuf::from("."));
    let mut layout = AgentAssetLayout::from_output_dir(&effective_output_dir);
    let effective_agents_path = init
        .path
        .clone()
        .or(init_defaults.path.clone())
        .unwrap_or_else(|| layout.agents_path.clone());
    layout = layout.with_agents_path(&effective_agents_path);
    let effective_agent_pack_dir = init
        .agent_pack_dir
        .clone()
        .or(init_defaults.agent_pack_dir.clone())
        .unwrap_or_else(|| layout.agent_bundle_dir.clone());
    let effective_fix_prompt_path = layout.fix_prompt_path.clone();
    let write_commands = init.write_commands || init_defaults.write_commands.unwrap_or(false);
    let shell_script = init.shell_script || init_defaults.shell_script.unwrap_or(false);
    let fix_prompt = init.fix_prompt || init_defaults.fix_prompt.unwrap_or(false);
    let agent_pack = if let Some(app) = init.from_scan.as_deref() {
        Some(run_scan_for_agent_pack(
            app,
            init_profile,
            init.baseline.as_deref(),
        )?)
    } else {
        None
    };

    maybe_write_agent_bundle(
        &init,
        &init_defaults.agent_pack_dir,
        shell_script,
        &effective_agent_pack_dir,
        agent_pack.as_ref(),
    )?;

    if shell_script {
        let command_hints = build_command_hints(
            &init,
            &effective_output_dir,
            &effective_agent_pack_dir,
            &effective_fix_prompt_path,
            init_profile,
            true,
            fix_prompt,
        );
        write_next_steps_script(
            &effective_agent_pack_dir.join("next-steps.sh"),
            &command_hints,
        )?;
    }

    let command_hints = (write_commands || shell_script || fix_prompt).then(|| {
        build_command_hints(
            &init,
            &effective_output_dir,
            &effective_agent_pack_dir,
            &effective_fix_prompt_path,
            init_profile,
            shell_script,
            fix_prompt,
        )
    });

    if fix_prompt {
        let pack = agent_pack.as_ref().ok_or_else(|| {
            miette::miette!(
                "`--fix-prompt` requires `--from-scan <path>` so voc has findings to summarize"
            )
        })?;
        let mut prompt_hints = command_hints.clone().unwrap_or_default();
        prompt_hints.repair_plan_path = Some(layout.repair_plan_path.display().to_string());
        write_fix_prompt_file(&effective_fix_prompt_path, pack, &prompt_hints)?;
    }

    write_agents_file(
        &effective_agents_path,
        agent_pack.as_ref(),
        Some(&effective_agent_pack_dir),
        command_hints.as_ref(),
    )?;
    println!("Updated {}", effective_agents_path.display());
    Ok(())
}

fn maybe_write_agent_bundle(
    init: &InitArgs,
    default_agent_pack_dir: &Option<PathBuf>,
    shell_script: bool,
    effective_agent_pack_dir: &Path,
    agent_pack: Option<&AgentPack>,
) -> Result<()> {
    if let Some(pack) = agent_pack {
        if init.agent_pack_dir.is_some() || default_agent_pack_dir.is_some() || shell_script {
            write_agent_pack(effective_agent_pack_dir, pack, AgentPackFormat::Bundle)?;
        }
    }
    Ok(())
}

fn build_command_hints(
    init: &InitArgs,
    effective_output_dir: &Path,
    effective_agent_pack_dir: &Path,
    effective_fix_prompt_path: &Path,
    init_profile: ScanProfile,
    shell_script: bool,
    fix_prompt: bool,
) -> CommandHints {
    CommandHints {
        output_dir: Some(effective_output_dir.display().to_string()),
        app_path: init
            .from_scan
            .as_deref()
            .map(|path| path.display().to_string()),
        baseline_path: init
            .baseline
            .as_deref()
            .map(|path| path.display().to_string()),
        agent_pack_dir: Some(effective_agent_pack_dir.display().to_string()),
        profile: Some(profile_key(init_profile)),
        shell_script,
        fix_prompt_path: fix_prompt.then(|| effective_fix_prompt_path.display().to_string()),
        repair_plan_path: None,
        pr_brief_path: None,
        pr_comment_path: None,
    }
}
