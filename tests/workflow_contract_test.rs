use std::fs;
use std::path::PathBuf;

use verifyos_cli::agent_assets::{
    AGENTS_FILE_NAME, AGENT_BUNDLE_DIR_NAME, FIX_PROMPT_NAME, PR_BRIEF_NAME, PR_COMMENT_NAME,
};

fn workflow_contents() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".github")
        .join("workflows")
        .join("voc-analysis.yml");
    fs::read_to_string(path).expect("workflow should be readable")
}

#[test]
fn workflow_declares_reusable_inputs_for_repair_and_comment_mode() {
    let workflow = workflow_contents();

    assert!(workflow.contains("doctor_repair:"));
    assert!(workflow.contains("comment_mode:"));
    assert!(workflow.contains("comment_plan_path:"));
    assert!(workflow.contains("DOCTOR_REPAIR=$doctor_repair"));
    assert!(workflow.contains("COMMENT_MODE=$comment_mode"));
    assert!(workflow.contains("COMMENT_PLAN_PATH=$comment_plan_path"));
}

#[test]
fn workflow_wires_doctor_repair_and_comment_mode_into_commands() {
    let workflow = workflow_contents();

    assert!(workflow.contains("config_doctor_repair"));
    assert!(workflow.contains("config_comment_mode"));
    assert!(workflow.contains("tomllib"));
    assert!(workflow.contains("doctor_repair = \",\".join(ci.get(\"doctor_repair\", []))"));
    assert!(workflow.contains("comment_mode = ci.get(\"comment_mode\", \"\")"));
    assert!(workflow.contains("doctor_cmd+=(--repair \"$DOCTOR_REPAIR\")"));
    assert!(workflow.contains("--plan-out \"$OUTPUT_DIR/repair-plan.md\""));
    assert!(workflow.contains("if [ \"$COMMENT_MODE\" = \"sticky\" ]; then"));
    assert!(workflow.contains("--from-plan"));
    assert!(workflow.contains("pr_comment_cmd+=(--plan-path \"$COMMENT_PLAN_PATH\")"));
    assert!(workflow.contains("pr_comment_cmd+=(--sticky-marker)"));
}

#[test]
fn workflow_uploads_expected_verifyos_outputs() {
    let workflow = workflow_contents();

    let expected_paths = [
        format!("${{{{ env.OUTPUT_DIR }}}}/{AGENTS_FILE_NAME}"),
        format!("${{{{ env.OUTPUT_DIR }}}}/{FIX_PROMPT_NAME}"),
        "${{ env.OUTPUT_DIR }}/repair-plan.md".to_string(),
        format!("${{{{ env.OUTPUT_DIR }}}}/{PR_BRIEF_NAME}"),
        format!("${{{{ env.OUTPUT_DIR }}}}/{PR_COMMENT_NAME}"),
        format!("${{{{ env.OUTPUT_DIR }}}}/{AGENT_BUNDLE_DIR_NAME}"),
        "${{ env.OUTPUT_DIR }}/doctor.json".to_string(),
        "${{ env.OUTPUT_DIR }}/report.sarif".to_string(),
    ];

    for expected in expected_paths {
        assert!(
            workflow.contains(&expected),
            "workflow should upload artifact path {expected}"
        );
    }
}

#[test]
fn release_workflow_renames_release_pr_branch_with_versioned_slug() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".github")
        .join("workflows")
        .join("release-plz.yml");
    let workflow = fs::read_to_string(path).expect("release workflow should be readable");
    let config =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("release-plz.toml"))
            .expect("release-plz config should be readable");

    assert!(config.contains("pr_branch_prefix = \"release-plz-\""));
    assert!(workflow.contains("Rename release PR branch"));
    assert!(workflow.contains("startswith(\"chore(release): release v\")"));
    assert!(workflow.contains("release-plz-v{os.environ['RELEASE_VERSION']}-{slug}"));
    assert!(workflow.contains("repos/$GITHUB_REPOSITORY/branches/$encoded_branch/rename"));
}
