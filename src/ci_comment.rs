use crate::doctor::DoctorReport;
use crate::report::AgentPack;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

const STICKY_MARKER: &str = "<!-- voc-analysis-comment -->";

pub fn render_workflow_pr_comment(
    output_dir: &Path,
    scan_exit: i32,
    doctor_exit: i32,
    sticky_marker: bool,
) -> Result<String> {
    let comment_path = output_dir.join("pr-comment.md");
    if comment_path.exists() {
        let comment = std::fs::read_to_string(&comment_path).into_diagnostic()?;
        return Ok(with_marker(comment.trim(), sticky_marker));
    }

    let doctor_path = output_dir.join("doctor.json");
    let agent_pack_path = output_dir.join(".verifyos-agent").join("agent-pack.json");
    let findings = load_agent_pack_findings(&agent_pack_path);
    let doctor_summary = load_doctor_summary(&doctor_path);

    let body = [
        "## voc analysis",
        "",
        &format!("- Findings: **{}**", findings),
        &format!("- Scan exit code: `{}`", scan_exit),
        &format!("- Doctor exit code: `{}`", doctor_exit),
        &format!("- Assets uploaded from: `{}`", output_dir.display()),
        "- Includes: `report.sarif`, `AGENTS.md`, `fix-prompt.md`, `pr-brief.md`, `pr-comment.md`, `.verifyos-agent/`",
        &format!("- Doctor summary: {}", doctor_summary),
    ]
    .join("\n");

    Ok(with_marker(&body, sticky_marker))
}

fn with_marker(body: &str, sticky_marker: bool) -> String {
    if sticky_marker {
        format!("{STICKY_MARKER}\n{body}")
    } else {
        body.to_string()
    }
}

fn load_agent_pack_findings(path: &Path) -> usize {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<AgentPack>(&raw).ok())
        .map(|pack| pack.total_findings)
        .unwrap_or(0)
}

fn load_doctor_summary(path: &Path) -> String {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<DoctorReport>(&raw).ok())
        .map(|report| {
            report
                .checks
                .into_iter()
                .map(|item| format!("{}: {:?}", item.name, item.status).to_uppercase())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|summary| !summary.is_empty())
        .unwrap_or_else(|| "doctor report missing".to_string())
}

#[cfg(test)]
mod tests {
    use super::render_workflow_pr_comment;
    use crate::doctor::{DoctorCheck, DoctorReport, DoctorStatus};
    use crate::report::AgentPack;
    use tempfile::tempdir;

    #[test]
    fn workflow_pr_comment_prefers_existing_file() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(
            dir.path().join("pr-comment.md"),
            "## verifyOS review summary",
        )
        .expect("write comment");

        let body = render_workflow_pr_comment(dir.path(), 1, 0, true).expect("render comment");

        assert!(body.contains("<!-- voc-analysis-comment -->"));
        assert!(body.contains("## verifyOS review summary"));
        assert!(!body.contains("## voc analysis"));
    }

    #[test]
    fn workflow_pr_comment_falls_back_to_doctor_and_agent_pack() {
        let dir = tempdir().expect("temp dir");
        let agent_dir = dir.path().join(".verifyos-agent");
        std::fs::create_dir_all(&agent_dir).expect("create agent dir");
        std::fs::write(
            agent_dir.join("agent-pack.json"),
            serde_json::to_string(&AgentPack {
                generated_at_unix: 1,
                total_findings: 3,
                findings: Vec::new(),
            })
            .expect("json"),
        )
        .expect("write agent pack");
        std::fs::write(
            dir.path().join("doctor.json"),
            serde_json::to_string(&DoctorReport {
                checks: vec![DoctorCheck {
                    name: "Config".to_string(),
                    status: DoctorStatus::Pass,
                    detail: "ok".to_string(),
                }],
                repair_plan: Vec::new(),
                plan_context: None,
            })
            .expect("json"),
        )
        .expect("write doctor report");

        let body = render_workflow_pr_comment(dir.path(), 1, 0, false).expect("render comment");

        assert!(body.contains("## voc analysis"));
        assert!(body.contains("Findings: **3**"));
        assert!(body.contains("Scan exit code: `1`"));
        assert!(body.contains("Doctor exit code: `0`"));
        assert!(body.contains("CONFIG: PASS"));
    }
}
