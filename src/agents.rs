use crate::profiles::{rule_inventory, RuleInventoryItem};
use crate::report::AgentPack;
use std::path::Path;

const MANAGED_START: &str = "<!-- verifyos-cli:agents:start -->";
const MANAGED_END: &str = "<!-- verifyos-cli:agents:end -->";

#[derive(Debug, Clone, Default)]
pub struct CommandHints {
    pub app_path: Option<String>,
    pub baseline_path: Option<String>,
    pub agent_pack_dir: Option<String>,
    pub profile: Option<String>,
    pub shell_script: bool,
    pub fix_prompt_path: Option<String>,
    pub pr_brief_path: Option<String>,
}

pub fn write_agents_file(
    path: &Path,
    agent_pack: Option<&AgentPack>,
    agent_pack_dir: Option<&Path>,
    command_hints: Option<&CommandHints>,
) -> Result<(), miette::Report> {
    let existing = if path.exists() {
        Some(std::fs::read_to_string(path).map_err(|err| {
            miette::miette!(
                "Failed to read existing AGENTS.md at {}: {}",
                path.display(),
                err
            )
        })?)
    } else {
        None
    };

    let managed_block = build_managed_block(agent_pack, agent_pack_dir, command_hints);
    let next = merge_agents_content(existing.as_deref(), &managed_block);
    std::fs::write(path, next)
        .map_err(|err| miette::miette!("Failed to write AGENTS.md at {}: {}", path.display(), err))
}

pub fn merge_agents_content(existing: Option<&str>, managed_block: &str) -> String {
    match existing {
        None => format!("# AGENTS.md\n\n{}", managed_block),
        Some(content) => {
            if let Some((start, end)) = managed_block_range(content) {
                let mut next = String::new();
                next.push_str(&content[..start]);
                if !next.ends_with('\n') {
                    next.push('\n');
                }
                next.push_str(managed_block);
                let tail = &content[end..];
                if !tail.is_empty() && !tail.starts_with('\n') {
                    next.push('\n');
                }
                next.push_str(tail);
                next
            } else if content.trim().is_empty() {
                format!("# AGENTS.md\n\n{}", managed_block)
            } else {
                let mut next = content.trim_end().to_string();
                next.push_str("\n\n");
                next.push_str(managed_block);
                next.push('\n');
                next
            }
        }
    }
}

pub fn build_managed_block(
    agent_pack: Option<&AgentPack>,
    agent_pack_dir: Option<&Path>,
    command_hints: Option<&CommandHints>,
) -> String {
    let inventory = rule_inventory();
    let agent_pack_dir_display = agent_pack_dir
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| ".verifyos-agent".to_string());
    let mut out = String::new();
    out.push_str(MANAGED_START);
    out.push('\n');
    out.push_str("## verifyOS-cli\n\n");
    out.push_str("Use `voc` before large iOS submission changes or release builds.\n\n");
    out.push_str("### Recommended Workflow\n\n");
    out.push_str("1. Run `voc --app <path-to-.ipa-or-.app> --profile basic` for a quick gate.\n");
    out.push_str(&format!(
        "2. Run `voc --app <path-to-.ipa-or-.app> --profile full --agent-pack {} --agent-pack-format bundle` before release or when an AI agent will patch findings.\n",
        agent_pack_dir_display
    ));
    out.push_str(&format!(
        "3. Read `{}/agent-pack.md` first, then patch the highest-priority scopes.\n",
        agent_pack_dir_display
    ));
    out.push_str("4. Re-run `voc` after each fix batch until the pack is clean.\n\n");
    out.push_str("### AI Agent Rules\n\n");
    out.push_str("- Prefer `voc --profile basic` during fast inner loops and `voc --profile full` before shipping.\n");
    out.push_str(&format!(
        "- When findings exist, generate an agent bundle with `voc --agent-pack {} --agent-pack-format bundle`.\n",
        agent_pack_dir_display
    ));
    out.push_str("- Fix `high` priority findings before `medium` and `low`.\n");
    out.push_str("- Treat `Info.plist`, `entitlements`, `ats-config`, and `bundle-resources` as the main fix scopes.\n");
    out.push_str("- Re-run `voc` after edits and compare against the previous agent pack to confirm findings were actually removed.\n\n");
    if let Some(hints) = command_hints {
        append_next_commands(&mut out, hints);
    }
    if let Some(pack) = agent_pack {
        append_current_project_risks(&mut out, pack, &agent_pack_dir_display);
    }
    out.push_str("### Rule Inventory\n\n");
    out.push_str("| Rule ID | Name | Category | Severity | Default Profiles |\n");
    out.push_str("| --- | --- | --- | --- | --- |\n");
    for item in inventory {
        out.push_str(&inventory_row(&item));
    }
    out.push('\n');
    out.push_str(MANAGED_END);
    out.push('\n');
    out
}

fn append_next_commands(out: &mut String, hints: &CommandHints) {
    let Some(app_path) = hints.app_path.as_deref() else {
        return;
    };

    let profile = hints.profile.as_deref().unwrap_or("full");
    let agent_pack_dir = hints.agent_pack_dir.as_deref().unwrap_or(".verifyos-agent");

    out.push_str("### Next Commands\n\n");
    out.push_str("Use these exact commands after each patch batch:\n\n");
    if hints.shell_script {
        out.push_str(&format!(
            "- Shortcut script: `{}/next-steps.sh`\n\n",
            agent_pack_dir
        ));
    }
    if let Some(prompt_path) = hints.fix_prompt_path.as_deref() {
        out.push_str(&format!("- Agent fix prompt: `{}`\n\n", prompt_path));
    }
    if let Some(pr_brief_path) = hints.pr_brief_path.as_deref() {
        out.push_str(&format!("- PR brief: `{}`\n\n", pr_brief_path));
    }
    out.push_str("```bash\n");
    out.push_str(&format!(
        "voc --app {} --profile {}\n",
        shell_quote(app_path),
        profile
    ));
    out.push_str(&format!(
        "voc --app {} --profile {} --format json > report.json\n",
        shell_quote(app_path),
        profile
    ));
    out.push_str(&format!(
        "voc --app {} --profile {} --agent-pack {} --agent-pack-format bundle\n",
        shell_quote(app_path),
        profile,
        shell_quote(agent_pack_dir)
    ));
    if let Some(baseline) = hints.baseline_path.as_deref() {
        let mut cmd = format!(
            "voc init --from-scan {} --profile {} --baseline {} --agent-pack-dir {} --write-commands",
            shell_quote(app_path),
            profile,
            shell_quote(baseline),
            shell_quote(agent_pack_dir)
        );
        if hints.shell_script {
            cmd.push_str(" --shell-script");
        }
        out.push_str(&format!("{cmd}\n"));
    } else {
        let mut cmd = format!(
            "voc init --from-scan {} --profile {} --agent-pack-dir {} --write-commands",
            shell_quote(app_path),
            profile,
            shell_quote(agent_pack_dir)
        );
        if hints.shell_script {
            cmd.push_str(" --shell-script");
        }
        out.push_str(&format!("{cmd}\n"));
    }
    out.push_str("```\n\n");
}

pub fn render_fix_prompt(pack: &AgentPack, hints: &CommandHints) -> String {
    let mut out = String::new();
    out.push_str("# verifyOS Fix Prompt\n\n");
    out.push_str(
        "Patch the current iOS bundle risks conservatively. Prefer minimal, review-safe edits.\n\n",
    );
    if let Some(app_path) = hints.app_path.as_deref() {
        out.push_str(&format!("- App artifact: `{}`\n", app_path));
    }
    if let Some(profile) = hints.profile.as_deref() {
        out.push_str(&format!("- Scan profile: `{}`\n", profile));
    }
    if let Some(agent_pack_dir) = hints.agent_pack_dir.as_deref() {
        out.push_str(&format!("- Agent bundle: `{}`\n", agent_pack_dir));
    }
    if let Some(prompt_path) = hints.fix_prompt_path.as_deref() {
        out.push_str(&format!("- Prompt file: `{}`\n", prompt_path));
    }
    out.push('\n');

    if pack.findings.is_empty() {
        out.push_str("## Findings\n\n- No current findings. Re-run the validation commands to confirm the app is still clean.\n\n");
    } else {
        out.push_str("## Findings\n\n");
        for finding in &pack.findings {
            out.push_str(&format!(
                "- **{}** (`{}`)\n",
                finding.rule_name, finding.rule_id
            ));
            out.push_str(&format!("  - Priority: `{}`\n", finding.priority));
            out.push_str(&format!("  - Scope: `{}`\n", finding.suggested_fix_scope));
            if !finding.target_files.is_empty() {
                out.push_str(&format!(
                    "  - Target files: {}\n",
                    finding.target_files.join(", ")
                ));
            }
            out.push_str(&format!(
                "  - Why it fails review: {}\n",
                finding.why_it_fails_review
            ));
            out.push_str(&format!("  - Patch hint: {}\n", finding.patch_hint));
            out.push_str(&format!("  - Recommendation: {}\n", finding.recommendation));
        }
        out.push('\n');
    }

    out.push_str("## Done When\n\n");
    out.push_str("- The relevant files are patched without widening permissions or exceptions.\n");
    out.push_str("- `voc` no longer reports the patched findings.\n");
    out.push_str("- Updated outputs are regenerated for the next loop.\n\n");

    out.push_str("## Validation Commands\n\n");
    if let Some(app_path) = hints.app_path.as_deref() {
        let profile = hints.profile.as_deref().unwrap_or("full");
        let agent_pack_dir = hints.agent_pack_dir.as_deref().unwrap_or(".verifyos-agent");
        out.push_str("```bash\n");
        out.push_str(&format!(
            "voc --app {} --profile {}\n",
            shell_quote(app_path),
            profile
        ));
        out.push_str(&format!(
            "voc --app {} --profile {} --agent-pack {} --agent-pack-format bundle\n",
            shell_quote(app_path),
            profile,
            shell_quote(agent_pack_dir)
        ));
        out.push_str("```\n");
    }

    out
}

pub fn render_pr_brief(pack: &AgentPack, hints: &CommandHints) -> String {
    let mut out = String::new();
    out.push_str("# verifyOS PR Brief\n\n");
    out.push_str("## Summary\n\n");
    out.push_str(&format!("- Findings in scope: `{}`\n", pack.total_findings));
    if let Some(app_path) = hints.app_path.as_deref() {
        out.push_str(&format!("- App artifact: `{}`\n", app_path));
    }
    if let Some(profile) = hints.profile.as_deref() {
        out.push_str(&format!("- Scan profile: `{}`\n", profile));
    }
    if let Some(baseline) = hints.baseline_path.as_deref() {
        out.push_str(&format!("- Baseline: `{}`\n", baseline));
    }
    out.push('\n');

    out.push_str("## What Changed\n\n");
    if pack.findings.is_empty() {
        out.push_str(
            "- No new or regressed risks are currently in scope after the latest scan.\n\n",
        );
    } else {
        out.push_str(
            "- This branch still contains findings that can affect App Store review outcomes.\n",
        );
        out.push_str(
            "- The recommended patch order below is sorted for review safety and repair efficiency.\n\n",
        );
    }

    out.push_str("## Current Risks\n\n");
    if pack.findings.is_empty() {
        out.push_str("- No open findings.\n\n");
    } else {
        let mut findings = pack.findings.clone();
        findings.sort_by(|a, b| {
            priority_rank(&a.priority)
                .cmp(&priority_rank(&b.priority))
                .then_with(|| a.suggested_fix_scope.cmp(&b.suggested_fix_scope))
                .then_with(|| a.rule_id.cmp(&b.rule_id))
        });

        for finding in &findings {
            out.push_str(&format!(
                "- **{}** (`{}`)\n",
                finding.rule_name, finding.rule_id
            ));
            out.push_str(&format!("  - Priority: `{}`\n", finding.priority));
            out.push_str(&format!("  - Scope: `{}`\n", finding.suggested_fix_scope));
            if !finding.target_files.is_empty() {
                out.push_str(&format!(
                    "  - Target files: {}\n",
                    finding.target_files.join(", ")
                ));
            }
            out.push_str(&format!(
                "  - Why review cares: {}\n",
                finding.why_it_fails_review
            ));
            out.push_str(&format!("  - Patch hint: {}\n", finding.patch_hint));
        }
        out.push('\n');
    }

    out.push_str("## Validation Commands\n\n");
    if let Some(app_path) = hints.app_path.as_deref() {
        let profile = hints.profile.as_deref().unwrap_or("full");
        let agent_pack_dir = hints.agent_pack_dir.as_deref().unwrap_or(".verifyos-agent");
        out.push_str("```bash\n");
        out.push_str(&format!(
            "voc --app {} --profile {}\n",
            shell_quote(app_path),
            profile
        ));
        out.push_str(&format!(
            "voc --app {} --profile {} --agent-pack {} --agent-pack-format bundle\n",
            shell_quote(app_path),
            profile,
            shell_quote(agent_pack_dir)
        ));
        if let Some(baseline) = hints.baseline_path.as_deref() {
            out.push_str(&format!(
                "voc doctor --fix --from-scan {} --profile {} --baseline {} --output-dir .verifyos --open-pr-brief\n",
                shell_quote(app_path),
                profile,
                shell_quote(baseline)
            ));
        }
        out.push_str("```\n");
    }

    out
}

fn append_current_project_risks(out: &mut String, pack: &AgentPack, agent_pack_dir: &str) {
    out.push_str("### Current Project Risks\n\n");
    out.push_str(&format!(
        "- Agent bundle: `{}/agent-pack.json` and `{}/agent-pack.md`\n\n",
        agent_pack_dir, agent_pack_dir
    ));
    if pack.findings.is_empty() {
        out.push_str(
            "- No new or regressed risks after applying the latest scan context. Re-run `voc` before release to keep this section fresh.\n\n",
        );
        return;
    }

    let mut findings = pack.findings.clone();
    findings.sort_by(|a, b| {
        priority_rank(&a.priority)
            .cmp(&priority_rank(&b.priority))
            .then_with(|| a.suggested_fix_scope.cmp(&b.suggested_fix_scope))
            .then_with(|| a.rule_id.cmp(&b.rule_id))
    });

    out.push_str("| Priority | Rule ID | Scope | Why it matters |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    for finding in &findings {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} |\n",
            finding.priority,
            finding.rule_id,
            finding.suggested_fix_scope,
            finding.why_it_fails_review
        ));
    }
    out.push('\n');

    out.push_str("#### Suggested Patch Order\n\n");
    for finding in &findings {
        out.push_str(&format!(
            "- **{}** (`{}`)\n",
            finding.rule_name, finding.rule_id
        ));
        out.push_str(&format!("  - Priority: `{}`\n", finding.priority));
        out.push_str(&format!(
            "  - Fix scope: `{}`\n",
            finding.suggested_fix_scope
        ));
        if !finding.target_files.is_empty() {
            out.push_str(&format!(
                "  - Target files: {}\n",
                finding.target_files.join(", ")
            ));
        }
        out.push_str(&format!(
            "  - Why it fails review: {}\n",
            finding.why_it_fails_review
        ));
        out.push_str(&format!("  - Patch hint: {}\n", finding.patch_hint));
    }
    out.push('\n');
}

fn priority_rank(priority: &str) -> u8 {
    match priority {
        "high" => 0,
        "medium" => 1,
        "low" => 2,
        _ => 3,
    }
}

fn inventory_row(item: &RuleInventoryItem) -> String {
    format!(
        "| `{}` | {} | `{:?}` | `{:?}` | `{}` |\n",
        item.rule_id,
        item.name,
        item.category,
        item.severity,
        item.default_profiles.join(", ")
    )
}

fn managed_block_range(content: &str) -> Option<(usize, usize)> {
    let start = content.find(MANAGED_START)?;
    let end_marker = content.find(MANAGED_END)?;
    Some((start, end_marker + MANAGED_END.len()))
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "/._-".contains(ch))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

#[cfg(test)]
mod tests {
    use super::{build_managed_block, merge_agents_content, CommandHints};
    use crate::report::{AgentFinding, AgentPack};
    use crate::rules::core::{RuleCategory, Severity};
    use std::path::Path;

    #[test]
    fn merge_agents_content_creates_new_file_when_missing() {
        let block = build_managed_block(None, None, None);
        let merged = merge_agents_content(None, &block);

        assert!(merged.starts_with("# AGENTS.md"));
        assert!(merged.contains("## verifyOS-cli"));
        assert!(merged.contains("RULE_PRIVACY_MANIFEST"));
    }

    #[test]
    fn merge_agents_content_replaces_existing_managed_block() {
        let block = build_managed_block(None, None, None);
        let existing = r#"# AGENTS.md

Custom note

<!-- verifyos-cli:agents:start -->
old block
<!-- verifyos-cli:agents:end -->

Keep this
"#;

        let merged = merge_agents_content(Some(existing), &block);

        assert!(merged.contains("Custom note"));
        assert!(merged.contains("Keep this"));
        assert!(!merged.contains("old block"));
        assert_eq!(
            merged.matches("<!-- verifyos-cli:agents:start -->").count(),
            1
        );
    }

    #[test]
    fn build_managed_block_includes_current_project_risks_when_scan_exists() {
        let pack = AgentPack {
            generated_at_unix: 0,
            total_findings: 1,
            findings: vec![AgentFinding {
                rule_id: "RULE_USAGE_DESCRIPTIONS".to_string(),
                rule_name: "Missing required usage description keys".to_string(),
                severity: Severity::Warning,
                category: RuleCategory::Privacy,
                priority: "medium".to_string(),
                message: "Missing NSCameraUsageDescription".to_string(),
                evidence: None,
                recommendation: "Add usage descriptions".to_string(),
                suggested_fix_scope: "Info.plist".to_string(),
                target_files: vec!["Info.plist".to_string()],
                patch_hint: "Update Info.plist".to_string(),
                why_it_fails_review: "Protected APIs require usage strings.".to_string(),
            }],
        };

        let block = build_managed_block(Some(&pack), Some(Path::new(".verifyos-agent")), None);

        assert!(block.contains("### Current Project Risks"));
        assert!(block.contains("#### Suggested Patch Order"));
        assert!(block.contains("`RULE_USAGE_DESCRIPTIONS`"));
        assert!(block.contains("Info.plist"));
        assert!(block.contains(".verifyos-agent/agent-pack.md"));
    }

    #[test]
    fn build_managed_block_includes_next_commands_when_requested() {
        let hints = CommandHints {
            app_path: Some("examples/bad_app.ipa".to_string()),
            baseline_path: Some("baseline.json".to_string()),
            agent_pack_dir: Some(".verifyos-agent".to_string()),
            profile: Some("basic".to_string()),
            shell_script: true,
            fix_prompt_path: Some(".verifyos-agent/fix-prompt.md".to_string()),
            pr_brief_path: Some(".verifyos-agent/pr-brief.md".to_string()),
        };

        let block = build_managed_block(None, Some(Path::new(".verifyos-agent")), Some(&hints));

        assert!(block.contains("### Next Commands"));
        assert!(block.contains("voc --app examples/bad_app.ipa --profile basic"));
        assert!(block.contains("--baseline baseline.json"));
        assert!(block.contains("--write-commands"));
        assert!(block.contains(".verifyos-agent/next-steps.sh"));
        assert!(block.contains("--shell-script"));
        assert!(block.contains(".verifyos-agent/fix-prompt.md"));
        assert!(block.contains(".verifyos-agent/pr-brief.md"));
    }
}
