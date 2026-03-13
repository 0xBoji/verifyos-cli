use clap::Parser;
use miette::Result;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use verifyos_cli::core::engine::{Engine, EngineRun};
use verifyos_cli::profiles::{register_rules, RuleSelection, ScanProfile};
use verifyos_cli::rules::core::{RuleStatus, Severity};

#[derive(Debug, Parser)]
pub struct LspArgs {
    /// Scan profile to use for real-time diagnostics
    #[arg(long, value_enum, default_value_t = ScanProfile::Full)]
    pub profile: ScanProfile,
}

struct Backend {
    client: Client,
    profile: ScanProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DocumentKind {
    InfoPlist,
    PrivacyManifest,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "verifyOS-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
        })
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.process_document(params.text_document.uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.process_document(params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }
}

impl Backend {
    async fn process_document(&self, uri: Url) {
        let Ok(path) = uri.to_file_path() else {
            return;
        };

        let Some(kind) = document_kind(&path) else {
            self.client.publish_diagnostics(uri, Vec::new(), None).await;
            return;
        };

        let Some(scan_root) = scan_root_for_document(&path) else {
            self.client
                .publish_diagnostics(
                    uri,
                    vec![informational_diagnostic(
                        "LSP_INFO",
                        "verifyOS could not infer an app bundle root for this file yet.",
                    )],
                    None,
                )
                .await;
            return;
        };

        self.client
            .log_message(
                MessageType::INFO,
                format!("Processing verifyOS diagnostics for {}", path.display()),
            )
            .await;

        let diagnostics = match run_engine(&scan_root, self.profile) {
            Ok(run) => diagnostics_for_document(kind, &run),
            Err(err) => vec![informational_diagnostic(
                "LSP_SCAN_ERROR",
                &format!("verifyOS could not evaluate this bundle: {err}"),
            )],
        };

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

fn run_engine(scan_root: &Path, profile: ScanProfile) -> Result<EngineRun, String> {
    let mut engine = Engine::new();
    let selection = RuleSelection::default();
    register_rules(&mut engine, profile, &selection);
    engine
        .run_on_bundle(scan_root, Instant::now())
        .map_err(|err| err.to_string())
}

fn document_kind(path: &Path) -> Option<DocumentKind> {
    match path.file_name().and_then(|name| name.to_str()) {
        Some("Info.plist") => Some(DocumentKind::InfoPlist),
        Some("PrivacyInfo.xcprivacy") => Some(DocumentKind::PrivacyManifest),
        _ => None,
    }
}

fn scan_root_for_document(path: &Path) -> Option<PathBuf> {
    match document_kind(path)? {
        DocumentKind::InfoPlist | DocumentKind::PrivacyManifest => {
            path.parent().map(Path::to_path_buf)
        }
    }
}

fn diagnostics_for_document(kind: DocumentKind, run: &EngineRun) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for result in &run.results {
        let Some(report) = result.report.as_ref().ok() else {
            continue;
        };
        if report.status != RuleStatus::Fail || !rule_applies_to_document(kind, result.rule_id) {
            continue;
        }

        diagnostics.push(Diagnostic {
            range: Range::default(),
            severity: Some(to_lsp_severity(result.severity)),
            code: Some(NumberOrString::String(result.rule_id.to_string())),
            source: Some("verifyOS".to_string()),
            message: diagnostic_message(result.rule_name, report),
            ..Default::default()
        });
    }

    diagnostics
}

fn rule_applies_to_document(kind: DocumentKind, rule_id: &str) -> bool {
    match kind {
        DocumentKind::InfoPlist => matches!(
            rule_id,
            "RULE_ATS_AUDIT"
                | "RULE_ATS_GRANULARITY"
                | "RULE_BUNDLE_METADATA_CONSISTENCY"
                | "RULE_CAMERA_USAGE"
                | "RULE_DEVICE_CAPABILITIES_AUDIT"
                | "RULE_EXPORT_COMPLIANCE"
                | "RULE_INFO_PLIST_CAPABILITIES_EMPTY"
                | "RULE_INFO_PLIST_REQUIRED_KEYS"
                | "RULE_INFO_PLIST_VERSIONING"
                | "RULE_LSAPPLICATIONQUERIES_SCHEMES_AUDIT"
                | "RULE_USAGE_DESCRIPTIONS"
                | "RULE_USAGE_DESCRIPTIONS_EMPTY"
        ),
        DocumentKind::PrivacyManifest => matches!(
            rule_id,
            "RULE_PRIVACY_MANIFEST_COMPLETENESS" | "RULE_PRIVACY_SDK_CROSSCHECK"
        ),
    }
}

fn diagnostic_message(rule_name: &str, report: &verifyos_cli::rules::core::RuleReport) -> String {
    match (&report.message, &report.evidence) {
        (Some(message), Some(evidence)) => format!("{rule_name}: {message} ({evidence})"),
        (Some(message), None) => format!("{rule_name}: {message}"),
        (None, Some(evidence)) => format!("{rule_name}: {evidence}"),
        (None, None) => rule_name.to_string(),
    }
}

fn to_lsp_severity(severity: Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
        Severity::Info => DiagnosticSeverity::INFORMATION,
    }
}

fn informational_diagnostic(code: &str, message: &str) -> Diagnostic {
    Diagnostic {
        range: Range::default(),
        severity: Some(DiagnosticSeverity::INFORMATION),
        code: Some(NumberOrString::String(code.to_string())),
        source: Some("verifyOS".to_string()),
        message: message.to_string(),
        ..Default::default()
    }
}

#[tokio::main]
pub async fn run(args: LspArgs) -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        profile: args.profile,
    });
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        diagnostic_message, diagnostics_for_document, document_kind, scan_root_for_document,
        DocumentKind,
    };
    use std::path::{Path, PathBuf};
    use verifyos_cli::core::engine::{EngineResult, EngineRun};
    use verifyos_cli::rules::core::{
        ArtifactCacheStats, RuleCategory, RuleReport, RuleStatus, Severity,
    };

    #[test]
    fn recognizes_supported_documents() {
        assert_eq!(
            document_kind(Path::new("/tmp/Payload/App.app/Info.plist")),
            Some(DocumentKind::InfoPlist)
        );
        assert_eq!(
            document_kind(Path::new("/tmp/Payload/App.app/PrivacyInfo.xcprivacy")),
            Some(DocumentKind::PrivacyManifest)
        );
        assert_eq!(document_kind(Path::new("/tmp/Other.plist")), None);
    }

    #[test]
    fn infers_scan_root_from_supported_documents() {
        assert_eq!(
            scan_root_for_document(Path::new("/tmp/Payload/App.app/Info.plist")),
            Some(PathBuf::from("/tmp/Payload/App.app"))
        );
        assert_eq!(
            scan_root_for_document(Path::new("/tmp/Payload/App.app/PrivacyInfo.xcprivacy")),
            Some(PathBuf::from("/tmp/Payload/App.app"))
        );
    }

    #[test]
    fn lsp_diagnostics_filter_to_relevant_document_rules() {
        let run = EngineRun {
            results: vec![
                engine_result(
                    "RULE_USAGE_DESCRIPTIONS",
                    "Missing Usage Description Keys",
                    Severity::Warning,
                    RuleReport {
                        status: RuleStatus::Fail,
                        message: Some("Missing required usage description keys".to_string()),
                        evidence: Some("Missing keys: NSCameraUsageDescription".to_string()),
                    },
                ),
                engine_result(
                    "RULE_PRIVACY_SDK_CROSSCHECK",
                    "Privacy Manifest vs SDK Usage",
                    Severity::Warning,
                    RuleReport {
                        status: RuleStatus::Fail,
                        message: Some(
                            "SDKs detected but privacy manifest lacks declarations".to_string(),
                        ),
                        evidence: Some("SDK signatures: FirebaseAnalytics".to_string()),
                    },
                ),
            ],
            total_duration_ms: 42,
            cache_stats: ArtifactCacheStats::default(),
        };

        let plist = diagnostics_for_document(DocumentKind::InfoPlist, &run);
        let privacy = diagnostics_for_document(DocumentKind::PrivacyManifest, &run);

        assert_eq!(plist.len(), 1);
        assert!(plist[0].message.contains("Missing Usage Description Keys"));
        assert_eq!(privacy.len(), 1);
        assert!(privacy[0].message.contains("Privacy Manifest vs SDK Usage"));
    }

    #[test]
    fn diagnostic_message_includes_evidence_when_available() {
        let report = RuleReport {
            status: RuleStatus::Fail,
            message: Some("Missing required usage description keys".to_string()),
            evidence: Some("Missing keys: NSCameraUsageDescription".to_string()),
        };

        assert_eq!(
            diagnostic_message("Missing Usage Description Keys", &report),
            "Missing Usage Description Keys: Missing required usage description keys (Missing keys: NSCameraUsageDescription)"
        );
    }

    fn engine_result(
        rule_id: &'static str,
        rule_name: &'static str,
        severity: Severity,
        report: RuleReport,
    ) -> EngineResult {
        EngineResult {
            rule_id,
            rule_name,
            category: RuleCategory::Other,
            severity,
            recommendation: "",
            report: Ok(report),
            duration_ms: 1,
        }
    }
}
