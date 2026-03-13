use clap::Parser;
use miette::Result;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use verifyos_cli::core::engine::Engine;
use verifyos_cli::profiles::{register_rules, RuleSelection, ScanProfile};

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

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }
}

impl Backend {
    async fn process_document(&self, uri: Url) {
        let path_result = uri.to_file_path();
        if let Ok(path) = path_result {
            // Check if we can find a root (e.g. nearest .ipa or just follow the path)
            // For LSP simplicity, if it's an Info.plist, we might need a dummy engine run or mock context
            // In a real scenario, we'd find the project root and locate the built artifact

            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Processing diagnostic for {}", path.display()),
                )
                .await;

            let mut engine = Engine::new();
            let selection = RuleSelection::default(); // Allow everything by default in LSP
            register_rules(&mut engine, self.profile, &selection);

            // Note: Currently Engine::run requires an .ipa path.
            // In the future, we could add Engine::run_on_bundle(path)
            // For now, if it's a .plist, we provide a warning that LSP feedback works best with built IPAs

            let diagnostics = if path.extension().and_then(|s| s.to_str()) == Some("plist") {
                vec![Diagnostic {
                    range: Range::default(),
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    code: Some(NumberOrString::String("LSP_INFO".to_string())),
                    source: Some("verifyOS".to_string()),
                    message: "verifyOS real-time diagnostics are best visualized after building the app bundle.".to_string(),
                    ..Default::default()
                }]
            } else {
                Vec::new()
            };

            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
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
