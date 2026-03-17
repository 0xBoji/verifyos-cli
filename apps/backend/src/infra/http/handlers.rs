use crate::app::{AppState, RateLimitError, ScanError};
use crate::domain::{ScanProfileInput, ScanRequest};
use axum::extract::{Multipart, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use serde_json::json;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use tracing::info;
use verifyos_cli::agent_assets::HANDOFF_MANIFEST_NAME;
use verifyos_cli::report::{
    apply_agent_pack_baseline, render_json, render_markdown, render_sarif, TimingMode,
};
use zip::ZipArchive;
use zip::{write::FileOptions, ZipWriter};

#[derive(Clone, Copy)]
enum ScanOutputFormat {
    Json,
    Sarif,
    Markdown,
}

fn parse_scan_format(value: &str) -> Option<ScanOutputFormat> {
    match value.to_ascii_lowercase().as_str() {
        "json" => Some(ScanOutputFormat::Json),
        "sarif" => Some(ScanOutputFormat::Sarif),
        "markdown" | "md" => Some(ScanOutputFormat::Markdown),
        _ => None,
    }
}

fn append_rule_list(values: &mut Vec<String>, input: &str) {
    for item in input.split(',') {
        let trimmed = item.trim();
        if !trimmed.is_empty() {
            values.push(trimmed.to_string());
        }
    }
}

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn scan_bundle(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Err(response) = require_rate_limit(&state, &headers).await {
        return response;
    }

    let mut request = ScanRequest {
        profile: None,
        include: Vec::new(),
        exclude: Vec::new(),
        baseline: None,
    };
    let mut temp_file: Option<NamedTempFile> = None;
    let mut project_file: Option<NamedTempFile> = None;
    let mut project_path: Option<PathBuf> = None;
    let mut project_dir: Option<tempfile::TempDir> = None;
    let mut format = ScanOutputFormat::Json;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        if name == "profile" {
            if let Ok(value) = field.text().await {
                request.profile = match value.to_lowercase().as_str() {
                    "basic" => Some(ScanProfileInput::Basic),
                    "full" => Some(ScanProfileInput::Full),
                    _ => None,
                };
            }
            continue;
        }

        if name == "include" {
            if let Ok(value) = field.text().await {
                append_rule_list(&mut request.include, &value);
            }
            continue;
        }

        if name == "exclude" {
            if let Ok(value) = field.text().await {
                append_rule_list(&mut request.exclude, &value);
            }
            continue;
        }

        if name == "format" {
            if let Ok(value) = field.text().await {
                match parse_scan_format(&value) {
                    Some(parsed) => format = parsed,
                    None => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({ "error": "format must be json, sarif, or markdown" })),
                        )
                            .into_response();
                    }
                }
            }
            continue;
        }

        if name == "baseline" {
            let bytes = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(err) => return to_error(err).into_response(),
            };
            match serde_json::from_slice(&bytes) {
                Ok(report) => request.baseline = Some(report),
                Err(err) => return to_error(err).into_response(),
            }
            continue;
        }

        if name == "bundle" {
            let mut file = match NamedTempFile::new() {
                Ok(file) => file,
                Err(err) => return to_error(err).into_response(),
            };
            let bytes = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(err) => return to_error(err).into_response(),
            };
            if let Err(err) = file.write_all(&bytes) {
                return to_error(err).into_response();
            }
            temp_file = Some(file);
            continue;
        }

        if name == "project" {
            let mut file = match NamedTempFile::new() {
                Ok(file) => file,
                Err(err) => return to_error(err).into_response(),
            };
            let bytes = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(err) => return to_error(err).into_response(),
            };
            if let Err(err) = file.write_all(&bytes) {
                return to_error(err).into_response();
            }
            project_file = Some(file);
        }
    }

    let Some(bundle) = temp_file else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "missing bundle file field" })),
        )
            .into_response();
    };

    if let Some(project_file) = project_file {
        let path = project_file.path();
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
            if ext.eq_ignore_ascii_case("zip") {
                match extract_project_zip(path) {
                    Ok((dir, project)) => {
                        project_dir = Some(dir);
                        project_path = project;
                    }
                    Err(err) => return to_error(err).into_response(),
                }
            } else if ext.eq_ignore_ascii_case("xcodeproj")
                || ext.eq_ignore_ascii_case("xcworkspace")
            {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Upload .xcodeproj/.xcworkspace as a .zip archive."
                    })),
                )
                    .into_response();
            }
        }
    }

    info!("running scan for uploaded bundle");
    let _keep_project_dir_alive = project_dir;
    match state
        .scan
        .run_scan(request, bundle.path(), project_path.as_deref())
    {
        Ok(result) => match format {
            ScanOutputFormat::Json => (StatusCode::OK, Json(result)).into_response(),
            ScanOutputFormat::Sarif => match render_sarif(&result.report) {
                Ok(body) => (
                    StatusCode::OK,
                    [(CONTENT_TYPE, "application/sarif+json")],
                    body,
                )
                    .into_response(),
                Err(err) => to_error(err).into_response(),
            },
            ScanOutputFormat::Markdown => (
                StatusCode::OK,
                [(CONTENT_TYPE, "text/markdown; charset=utf-8")],
                render_markdown(
                    &result.report,
                    result.baseline.as_ref().map(|summary| summary.suppressed),
                    TimingMode::Summary,
                ),
            )
                .into_response(),
        },
        Err(err) => (StatusCode::BAD_REQUEST, Json(error_body(err))).into_response(),
    }
}

pub async fn handoff_bundle(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Err(response) = require_rate_limit(&state, &headers).await {
        return response;
    }

    let mut request = ScanRequest {
        profile: None,
        include: Vec::new(),
        exclude: Vec::new(),
        baseline: None,
    };
    let mut temp_file: Option<NamedTempFile> = None;
    let mut project_file: Option<NamedTempFile> = None;
    let mut project_path: Option<PathBuf> = None;
    let mut project_dir: Option<tempfile::TempDir> = None;
    let mut bundle_name = "app-bundle".to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        if name == "profile" {
            if let Ok(value) = field.text().await {
                request.profile = match value.to_lowercase().as_str() {
                    "basic" => Some(ScanProfileInput::Basic),
                    "full" => Some(ScanProfileInput::Full),
                    _ => None,
                };
            }
            continue;
        }

        if name == "include" {
            if let Ok(value) = field.text().await {
                append_rule_list(&mut request.include, &value);
            }
            continue;
        }

        if name == "exclude" {
            if let Ok(value) = field.text().await {
                append_rule_list(&mut request.exclude, &value);
            }
            continue;
        }

        if name == "baseline" {
            let bytes = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(err) => return to_error(err).into_response(),
            };
            match serde_json::from_slice(&bytes) {
                Ok(report) => request.baseline = Some(report),
                Err(err) => return to_error(err).into_response(),
            }
            continue;
        }

        if name == "bundle" {
            if let Some(file_name) = field.file_name() {
                bundle_name = file_name.to_string();
            }
            let mut file = match NamedTempFile::new() {
                Ok(file) => file,
                Err(err) => return to_error(err).into_response(),
            };
            let bytes = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(err) => return to_error(err).into_response(),
            };
            if let Err(err) = file.write_all(&bytes) {
                return to_error(err).into_response();
            }
            temp_file = Some(file);
            continue;
        }

        if name == "project" {
            let mut file = match NamedTempFile::new() {
                Ok(file) => file,
                Err(err) => return to_error(err).into_response(),
            };
            let bytes = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(err) => return to_error(err).into_response(),
            };
            if let Err(err) = file.write_all(&bytes) {
                return to_error(err).into_response();
            }
            project_file = Some(file);
        }
    }

    let Some(bundle) = temp_file else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "missing bundle file field" })),
        )
            .into_response();
    };

    if let Some(project_file) = project_file {
        let path = project_file.path();
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
            if ext.eq_ignore_ascii_case("zip") {
                match extract_project_zip(path) {
                    Ok((dir, project)) => {
                        project_dir = Some(dir);
                        project_path = project;
                    }
                    Err(err) => return to_error(err).into_response(),
                }
            } else if ext.eq_ignore_ascii_case("xcodeproj")
                || ext.eq_ignore_ascii_case("xcworkspace")
            {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Upload .xcodeproj/.xcworkspace as a .zip archive."
                    })),
                )
                    .into_response();
            }
        }
    }

    let output_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(err) => return to_error(err).into_response(),
    };
    let layout = verifyos_cli::agent_assets::AgentAssetLayout::from_output_dir(
        output_dir.path().join(".verifyos"),
    );
    let profile = request
        .profile
        .as_ref()
        .map(|profile| format!("{profile:?}").to_ascii_lowercase())
        .unwrap_or_else(|| "full".to_string());

    info!("building agent handoff bundle");
    let baseline = request.baseline.clone();
    let outcome = match state
        .scan
        .run_scan_report(request, bundle.path(), project_path.as_deref())
    {
        Ok(outcome) => outcome,
        Err(err) => return (StatusCode::BAD_REQUEST, Json(error_body(err))).into_response(),
    };
    let mut pack = verifyos_cli::report::build_agent_pack(&outcome.report);
    if let Some(baseline) = baseline.as_ref() {
        apply_agent_pack_baseline(&mut pack, baseline);
    }

    let hints = verifyos_cli::agents::CommandHints {
        output_dir: Some(layout.output_dir.display().to_string()),
        app_path: Some(bundle_name.clone()),
        baseline_path: None,
        agent_pack_dir: Some(layout.agent_bundle_dir.display().to_string()),
        profile: Some(profile.clone()),
        shell_script: true,
        fix_prompt_path: Some(layout.fix_prompt_path.display().to_string()),
        repair_plan_path: Some(layout.repair_plan_path.display().to_string()),
        pr_brief_path: Some(layout.pr_brief_path.display().to_string()),
        pr_comment_path: Some(layout.pr_comment_path.display().to_string()),
    };

    if let Err(err) = verifyos_cli::agents::write_agents_file(
        &layout.agents_path,
        Some(&pack),
        Some(&layout.agent_bundle_dir),
        Some(&hints),
    ) {
        return to_error(err).into_response();
    }

    if let Err(err) = verifyos_cli::agent_io::write_agent_pack(
        &layout.agent_bundle_dir,
        &pack,
        verifyos_cli::report::AgentPackFormat::Bundle,
    ) {
        return to_error(err).into_response();
    }
    if let Err(err) =
        verifyos_cli::agent_io::write_fix_prompt_file(&layout.fix_prompt_path, &pack, &hints)
    {
        return to_error(err).into_response();
    }
    if let Err(err) =
        verifyos_cli::agent_io::write_pr_brief_file(&layout.pr_brief_path, &pack, &hints)
    {
        return to_error(err).into_response();
    }
    if let Err(err) =
        verifyos_cli::agent_io::write_pr_comment_file(&layout.pr_comment_path, &pack, &hints)
    {
        return to_error(err).into_response();
    }
    if let Err(err) =
        verifyos_cli::agent_io::write_next_steps_script(&layout.next_steps_script_path, &hints)
    {
        return to_error(err).into_response();
    }

    if let Err(err) = write_repair_plan(&layout, &hints) {
        return to_error(err).into_response();
    }

    let report_json_path = layout.output_dir.join("report.json");
    let report_json = match render_json(&outcome.report) {
        Ok(report_json) => report_json,
        Err(err) => return to_error(err).into_response(),
    };
    if let Err(err) = std::fs::write(&report_json_path, report_json) {
        return to_error(err).into_response();
    }

    let manifest = HandoffManifest {
        app_path: bundle_name,
        baseline_path: None,
        profile,
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
            report_json_path.display().to_string(),
        ],
    };
    let manifest_path = layout.output_dir.join(HANDOFF_MANIFEST_NAME);
    if let Err(err) = std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| "{}".to_string()),
    ) {
        return to_error(err).into_response();
    }

    if let Err(err) = write_apply_script(output_dir.path()) {
        return to_error(err).into_response();
    }

    let _keep_project_dir_alive = project_dir;
    let zip_bytes = match zip_handoff(output_dir.path()) {
        Ok(bytes) => bytes,
        Err(err) => return to_error(err).into_response(),
    };

    (
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_DISPOSITION,
            "attachment; filename=\"verifyos-handoff.zip\"",
        )],
        zip_bytes,
    )
        .into_response()
}

fn error_body(err: ScanError) -> serde_json::Value {
    json!({ "error": err.to_string() })
}

fn rate_limit_error_response(err: RateLimitError) -> axum::response::Response {
    match err {
        RateLimitError::Exceeded => (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({ "error": "rate limit exceeded" })),
        )
            .into_response(),
    }
}

async fn require_rate_limit(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), axum::response::Response> {
    let ip = client_ip(headers).unwrap_or_else(|| "unknown".to_string());
    match state.rate_limit.check(&ip).await {
        Ok(_) => Ok(()),
        Err(err) => Err(rate_limit_error_response(err)),
    }
}

fn client_ip(headers: &HeaderMap) -> Option<String> {
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            if let Some(first) = value.split(',').next() {
                let trimmed = first.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn to_error(err: impl std::fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": err.to_string() })),
    )
}

#[derive(Debug, Serialize)]
struct HandoffManifest {
    app_path: String,
    baseline_path: Option<String>,
    profile: String,
    output_dir: String,
    assets: Vec<String>,
}

fn write_repair_plan(
    layout: &verifyos_cli::agent_assets::AgentAssetLayout,
    hints: &verifyos_cli::agents::CommandHints,
) -> Result<(), Box<dyn std::error::Error>> {
    let policy =
        verifyos_cli::agent_assets::RepairPolicy::new(std::collections::HashSet::new(), true, true);
    let plan = verifyos_cli::agent_assets::build_repair_plan(layout, &policy);
    let mut out = String::new();
    out.push_str("# verifyOS Repair Plan\n\n## Context\n\n");
    if let Some(app_path) = hints.app_path.as_deref() {
        out.push_str(&format!(
            "- Source: `fresh-scan`\n- Scan artifact: `{}`\n",
            app_path
        ));
    } else {
        out.push_str("- Source: `fresh-scan`\n");
    }
    out.push_str("\n## Planned Outputs\n\n");
    for item in plan {
        out.push_str(&format!("- **{}**\n", item.target));
        out.push_str(&format!("  - Path: `{}`\n", item.path));
        out.push_str(&format!("  - Reason: {}\n", item.reason));
    }

    std::fs::write(&layout.repair_plan_path, out)?;
    Ok(())
}

fn write_apply_script(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cp -R "$SCRIPT_DIR/.verifyos" "$ROOT/.verifyos"
cp "$SCRIPT_DIR/AGENTS.md" "$ROOT/AGENTS.md"

ZIP_NAME="verifyos-handoff.zip"
if [ -f "$SCRIPT_DIR/$ZIP_NAME" ]; then
  rm -f "$SCRIPT_DIR/$ZIP_NAME"
fi

echo "verifyOS handoff installed into $ROOT"
"#;

    std::fs::write(root.join("apply-handoff.sh"), script)?;
    Ok(())
}

fn zip_handoff(root: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);
        let options = FileOptions::default();
        add_dir_to_zip(&mut zip, root, root, options)?;
        zip.finish()?;
    }
    Ok(buffer.into_inner())
}

fn add_dir_to_zip(
    zip: &mut ZipWriter<&mut std::io::Cursor<Vec<u8>>>,
    root: &Path,
    path: &Path,
    options: FileOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let name = entry_path.strip_prefix(root)?.to_string_lossy();
        if entry_path.is_dir() {
            zip.add_directory(name.to_string(), options)?;
            add_dir_to_zip(zip, root, &entry_path, options)?;
        } else {
            zip.start_file(name.to_string(), options)?;
            let mut file = std::fs::File::open(entry_path)?;
            std::io::copy(&mut file, zip)?;
        }
    }
    Ok(())
}

fn extract_project_zip(
    path: &Path,
) -> Result<(tempfile::TempDir, Option<PathBuf>), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let out_path = dir.path().join(entry.name());
        if entry.name().ends_with('/') {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut outfile = std::fs::File::create(&out_path)?;
        std::io::copy(&mut entry, &mut outfile)?;
    }

    let project = find_project_path(dir.path());
    Ok((dir, project))
}

fn find_project_path(root: &Path) -> Option<PathBuf> {
    let mut queue = vec![root.to_path_buf()];
    let mut project = None;
    while let Some(dir) = queue.pop() {
        let entries = std::fs::read_dir(&dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("xcworkspace"))
                {
                    return Some(path);
                }
                if project.is_none()
                    && path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("xcodeproj"))
                {
                    project = Some(path.clone());
                }
                queue.push(path);
            }
        }
    }
    project
}
