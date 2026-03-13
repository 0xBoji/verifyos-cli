use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const AGENTS_FILE_NAME: &str = "AGENTS.md";
pub const AGENT_BUNDLE_DIR_NAME: &str = ".verifyos-agent";
pub const AGENT_PACK_JSON_NAME: &str = "agent-pack.json";
pub const AGENT_PACK_MARKDOWN_NAME: &str = "agent-pack.md";
pub const NEXT_STEPS_SCRIPT_NAME: &str = "next-steps.sh";
pub const FIX_PROMPT_NAME: &str = "fix-prompt.md";
pub const PR_BRIEF_NAME: &str = "pr-brief.md";
pub const PR_COMMENT_NAME: &str = "pr-comment.md";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, ValueEnum)]
pub enum RepairTarget {
    Agents,
    AgentBundle,
    FixPrompt,
    PrBrief,
    PrComment,
}

impl RepairTarget {
    pub fn key(self) -> &'static str {
        match self {
            RepairTarget::Agents => "agents",
            RepairTarget::AgentBundle => "agent-bundle",
            RepairTarget::FixPrompt => "fix-prompt",
            RepairTarget::PrBrief => "pr-brief",
            RepairTarget::PrComment => "pr-comment",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepairPlanItem {
    pub target: String,
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentAssetLayout {
    pub output_dir: PathBuf,
    pub agents_path: PathBuf,
    pub agent_bundle_dir: PathBuf,
    pub agent_pack_json_path: PathBuf,
    pub agent_pack_markdown_path: PathBuf,
    pub next_steps_script_path: PathBuf,
    pub fix_prompt_path: PathBuf,
    pub pr_brief_path: PathBuf,
    pub pr_comment_path: PathBuf,
}

impl AgentAssetLayout {
    pub fn new(output_dir: impl Into<PathBuf>, agents_path: impl Into<PathBuf>) -> Self {
        let output_dir = output_dir.into();
        let agents_path = agents_path.into();
        let agent_bundle_dir = output_dir.join(AGENT_BUNDLE_DIR_NAME);

        Self {
            output_dir: output_dir.clone(),
            agents_path,
            agent_pack_json_path: agent_bundle_dir.join(AGENT_PACK_JSON_NAME),
            agent_pack_markdown_path: agent_bundle_dir.join(AGENT_PACK_MARKDOWN_NAME),
            next_steps_script_path: agent_bundle_dir.join(NEXT_STEPS_SCRIPT_NAME),
            fix_prompt_path: output_dir.join(FIX_PROMPT_NAME),
            pr_brief_path: output_dir.join(PR_BRIEF_NAME),
            pr_comment_path: output_dir.join(PR_COMMENT_NAME),
            agent_bundle_dir,
        }
    }

    pub fn from_output_dir(output_dir: impl Into<PathBuf>) -> Self {
        let output_dir = output_dir.into();
        Self::new(output_dir.clone(), output_dir.join(AGENTS_FILE_NAME))
    }

    pub fn with_agents_path(&self, agents_path: impl Into<PathBuf>) -> Self {
        Self::new(self.output_dir.clone(), agents_path.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairPolicy {
    repair_all: bool,
    repair_targets: HashSet<RepairTarget>,
    pub open_pr_brief: bool,
    pub open_pr_comment: bool,
}

impl RepairPolicy {
    pub fn new(
        repair_targets: HashSet<RepairTarget>,
        open_pr_brief: bool,
        open_pr_comment: bool,
    ) -> Self {
        let repair_all = repair_targets.is_empty();
        Self {
            repair_all,
            repair_targets,
            open_pr_brief,
            open_pr_comment,
        }
    }

    pub fn repair_targets(&self) -> &HashSet<RepairTarget> {
        &self.repair_targets
    }

    pub fn repairs_all(&self) -> bool {
        self.repair_all
    }

    pub fn should_repair_agents(&self) -> bool {
        self.repair_all || self.repair_targets.contains(&RepairTarget::Agents)
    }

    pub fn should_repair_bundle(&self) -> bool {
        self.repair_all || self.repair_targets.contains(&RepairTarget::AgentBundle)
    }

    pub fn should_repair_fix_prompt(&self) -> bool {
        self.repair_all || self.repair_targets.contains(&RepairTarget::FixPrompt)
    }

    pub fn should_include_pr_brief(&self) -> bool {
        self.open_pr_brief || self.repair_targets.contains(&RepairTarget::PrBrief)
    }

    pub fn should_include_pr_comment(&self) -> bool {
        self.open_pr_comment || self.repair_targets.contains(&RepairTarget::PrComment)
    }

    pub fn should_repair_pr_brief(&self) -> bool {
        self.repair_all || self.repair_targets.contains(&RepairTarget::PrBrief)
    }

    pub fn should_repair_pr_comment(&self) -> bool {
        self.repair_all || self.repair_targets.contains(&RepairTarget::PrComment)
    }
}

pub fn build_repair_plan(layout: &AgentAssetLayout, policy: &RepairPolicy) -> Vec<RepairPlanItem> {
    let mut plan = Vec::new();

    if policy.should_repair_agents() {
        plan.push(RepairPlanItem {
            target: RepairTarget::Agents.key().to_string(),
            path: layout.agents_path.display().to_string(),
            reason: "refresh managed AGENTS.md block".to_string(),
        });
    }

    if policy.should_repair_bundle() {
        plan.push(RepairPlanItem {
            target: RepairTarget::AgentBundle.key().to_string(),
            path: layout.agent_bundle_dir.display().to_string(),
            reason: "rebuild agent-pack files and next-steps.sh".to_string(),
        });
    }

    if policy.should_repair_fix_prompt() {
        plan.push(RepairPlanItem {
            target: RepairTarget::FixPrompt.key().to_string(),
            path: layout.fix_prompt_path.display().to_string(),
            reason: "refresh AI fix prompt".to_string(),
        });
    }

    if policy.should_include_pr_brief() {
        plan.push(RepairPlanItem {
            target: RepairTarget::PrBrief.key().to_string(),
            path: layout.pr_brief_path.display().to_string(),
            reason: "refresh PR handoff brief".to_string(),
        });
    }

    if policy.should_include_pr_comment() {
        plan.push(RepairPlanItem {
            target: RepairTarget::PrComment.key().to_string(),
            path: layout.pr_comment_path.display().to_string(),
            reason: "refresh sticky PR comment draft".to_string(),
        });
    }

    plan
}

pub fn relative_to_agents(agents_path: &Path, asset_path: &Path) -> String {
    agents_path
        .parent()
        .and_then(|parent| asset_path.strip_prefix(parent).ok())
        .unwrap_or(asset_path)
        .display()
        .to_string()
}
