mod workspace;

use dashmap::DashMap;
use ferrotex_log::ir::EventPayload;
use ferrotex_log::parser::LogParser;
use ferrotex_syntax::{SyntaxKind, SyntaxNode};
use line_index::LineIndex;
use notify::{EventKind, RecursiveMode, Watcher};
use std::sync::{Arc, Mutex};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use workspace::Workspace;

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: DashMap<Url, String>,
    workspace: Workspace,
    root_uri: Arc<Mutex<Option<Url>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        {
            let mut root = self.root_uri.lock().unwrap();
            *root = params.root_uri.clone();
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_symbol_provider: Some(OneOf::Left(true)),
                document_link_provider: Some(DocumentLinkOptions {
                    resolve_provider: Some(false),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "ferrotexd initialized!")
            .await;

        // Start watching log files in the background
        let client = self.client.clone();
        let root_uri = self.root_uri.clone();
        tokio::spawn(async move {
            if let Err(e) = watch_workspace_logs(client, root_uri).await {
                eprintln!("Error watching logs: {:?}", e);
            }
        });
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        self.documents.insert(uri.clone(), text.clone());
        self.workspace.update(&uri, &text);
        self.validate_document(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // With FULL sync, content changes are the full text
        if let Some(change) = params.content_changes.into_iter().last() {
            let uri = params.text_document.uri;
            let text = change.text;

            self.documents.insert(uri.clone(), text.clone());
            self.workspace.update(&uri, &text);
            self.validate_document(&uri, &text).await;
        }
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        let uri = params.text_document.uri;
        if let Some(text) = self.documents.get(&uri) {
            let line_index = LineIndex::new(&text);
            let includes = self.workspace.get_includes(&uri);

            let links = includes
                .into_iter()
                .map(|inc| {
                    let start = line_index.line_col(inc.range.start());
                    let end = line_index.line_col(inc.range.end());

                    // Resolve target URI relative to current document
                    // This is best-effort. We assume relative paths.
                    let target = uri.join(&inc.path).ok();

                    DocumentLink {
                        range: Range {
                            start: Position {
                                line: start.line,
                                character: start.col,
                            },
                            end: Position {
                                line: end.line,
                                character: end.col,
                            },
                        },
                        target,
                        tooltip: Some(inc.path),
                        data: None,
                    }
                })
                .collect();

            Ok(Some(links))
        } else {
            Ok(None)
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        if let Some(text) = self.documents.get(&uri) {
            let parse = ferrotex_syntax::parse(&text);
            let root = parse.syntax();
            let line_index = LineIndex::new(&text);

            let mut symbols = Vec::new();
            // Simple traversal for top-level environments
            for child in root.children() {
                if let Some(symbol) = to_document_symbol(&child, &line_index) {
                    symbols.push(symbol);
                }
            }

            Ok(Some(DocumentSymbolResponse::Nested(symbols)))
        } else {
            Ok(None)
        }
    }
}

impl Backend {
    async fn validate_document(&self, uri: &Url, text: &str) {
        let parse = ferrotex_syntax::parse(text);
        let line_index = LineIndex::new(text);

        let mut diagnostics: Vec<Diagnostic> = parse
            .errors
            .into_iter()
            .map(|err| {
                let start = line_index.line_col(err.range.start());
                let end = line_index.line_col(err.range.end());

                Diagnostic {
                    range: Range {
                        start: Position {
                            line: start.line,
                            character: start.col,
                        },
                        end: Position {
                            line: end.line,
                            character: end.col,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("ferrotex-syntax".to_string()),
                    message: err.message,
                    related_information: None,
                    tags: None,
                    data: None,
                }
            })
            .collect();

        // Check for project-level errors (cycles)
        let cycles = self.workspace.detect_cycles();
        for (cycle_uri, cycle_range, message) in cycles {
            if &cycle_uri == uri {
                let start = line_index.line_col(cycle_range.start());
                let end = line_index.line_col(cycle_range.end());

                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: start.line,
                            character: start.col,
                        },
                        end: Position {
                            line: end.line,
                            character: end.col,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("ferrotex-project".to_string()),
                    message,
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

fn to_document_symbol(node: &SyntaxNode, line_index: &LineIndex) -> Option<DocumentSymbol> {
    let range = node.text_range();
    let start = line_index.line_col(range.start());
    let end = line_index.line_col(range.end());
    let lsp_range = Range {
        start: Position {
            line: start.line,
            character: start.col,
        },
        end: Position {
            line: end.line,
            character: end.col,
        },
    };

    match node.kind() {
        SyntaxKind::Environment => {
            // Extract environment name from the first Group child
            let name = node
                .children()
                .find(|c| c.kind() == SyntaxKind::Group)
                .map(|g| {
                    let text = g.text().to_string();
                    // Strip braces { }
                    if text.len() >= 2 {
                        text[1..text.len() - 1].to_string()
                    } else {
                        text
                    }
                })
                .unwrap_or_else(|| "Environment".to_string());

            #[allow(deprecated)]
            Some(DocumentSymbol {
                name,
                detail: None,
                kind: SymbolKind::CLASS, // Close enough
                tags: None,
                deprecated: None,
                range: lsp_range,
                selection_range: lsp_range, // Use full range for selection for now
                children: Some(
                    node.children()
                        .filter_map(|c| to_document_symbol(&c, line_index))
                        .collect(),
                ),
            })
        }
        SyntaxKind::Group =>
        {
            #[allow(deprecated)]
            Some(DocumentSymbol {
                name: "Group".to_string(),
                detail: None,
                kind: SymbolKind::MODULE,
                tags: None,
                deprecated: None,
                range: lsp_range,
                selection_range: lsp_range,
                children: Some(
                    node.children()
                        .filter_map(|c| to_document_symbol(&c, line_index))
                        .collect(),
                ),
            })
        }
        SyntaxKind::Section => {
            // Extract section title from the first Group child
            let name = node
                .children()
                .find(|c| c.kind() == SyntaxKind::Group)
                .map(|g| {
                    let text = g.text().to_string();
                    // Strip braces { }
                    if text.len() >= 2 {
                        text[1..text.len() - 1].to_string()
                    } else {
                        text
                    }
                })
                .unwrap_or_else(|| "Section".to_string());

            #[allow(deprecated)]
            Some(DocumentSymbol {
                name,
                detail: None,
                kind: SymbolKind::STRING,
                tags: None,
                deprecated: None,
                range: lsp_range,
                selection_range: lsp_range,
                children: Some(
                    node.children()
                        .filter_map(|c| to_document_symbol(&c, line_index))
                        .collect(),
                ),
            })
        }
        _ => None,
    }
}

async fn watch_workspace_logs(
    client: Client,
    root_uri: Arc<Mutex<Option<Url>>>,
) -> anyhow::Result<()> {
    // Get the workspace root path
    let path = {
        let root = root_uri.lock().unwrap();
        if let Some(uri) = root.as_ref() {
            if let Ok(path) = uri.to_file_path() {
                path
            } else {
                // If not a file URI (e.g. untitled), fallback to current dir
                std::env::current_dir()?
            }
        } else {
            std::env::current_dir()?
        }
    };

    client
        .log_message(MessageType::INFO, format!("Watching logs in: {:?}", path))
        .await;

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            let _ = tx.blocking_send(event);
        }
    })?;

    watcher.watch(&path, RecursiveMode::Recursive)?;

    let mut parser = LogParser::new();
    let mut file_offset: u64 = 0;

    // We only care about one log file for this MVP to avoid mixing streams
    // Let's pick the first .log file we see modified.
    let mut tracked_log: Option<std::path::PathBuf> = None;

    while let Some(event) = rx.recv().await {
        // Filter for .log files
        let log_path = event
            .paths
            .iter()
            .find(|p| p.extension().is_some_and(|ext| ext == "log"));

        if let Some(log_path) = log_path {
            // Simple logic: lock onto the first log file we see
            if tracked_log.is_none() {
                tracked_log = Some(log_path.clone());
                client
                    .log_message(MessageType::INFO, format!("Tracking log: {:?}", log_path))
                    .await;
            }

            if tracked_log.as_ref() == Some(log_path) {
                if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                    // Read and parse
                    if let Ok(mut file) = tokio::fs::File::open(log_path).await {
                        let metadata = file.metadata().await?;
                        let current_len = metadata.len();

                        if current_len > file_offset {
                            use tokio::io::{AsyncReadExt, AsyncSeekExt};
                            file.seek(std::io::SeekFrom::Start(file_offset)).await?;
                            let mut buffer = String::new();
                            file.read_to_string(&mut buffer).await?;

                            let events = parser.update(&buffer);
                            file_offset = current_len;

                            // Convert events to diagnostics
                            // Note: This is a simplified mapping. Real mapping needs state tracking (ErrorStart -> ErrorLineRef -> ...).
                            // For now, we'll map standalone Warning events and ErrorStart events.

                            let mut diagnostics = Vec::new();

                            for event in events {
                                match event.payload {
                                    EventPayload::Warning { message } => {
                                        diagnostics.push(Diagnostic {
                                            range: Range::default(), // No location info yet
                                            severity: Some(DiagnosticSeverity::WARNING),
                                            message,
                                            source: Some("ferrotexd".to_string()),
                                            ..Default::default()
                                        });
                                    }
                                    EventPayload::ErrorStart { message } => {
                                        diagnostics.push(Diagnostic {
                                            range: Range::default(),
                                            severity: Some(DiagnosticSeverity::ERROR),
                                            message,
                                            source: Some("ferrotexd".to_string()),
                                            ..Default::default()
                                        });
                                    }
                                    _ => {}
                                }
                            }

                            if !diagnostics.is_empty() {
                                // We need to map these diagnostics to the source file (.tex), not the log file.
                                // But we don't know the source file path easily yet without parsing FileEnter/FileExit.
                                // For MVP, we will publish them to the LOG FILE itself or a dummy URI, just to verify E2E.
                                // VS Code lets you publish diagnostics for any URI.

                                // Let's publish to the log file URI for now so they show up if user opens the log.
                                let uri = Url::from_file_path(log_path)
                                    .map_err(|_| anyhow::anyhow!("Invalid path"))?;
                                client.publish_diagnostics(uri, diagnostics, None).await;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: DashMap::new(),
        workspace: Workspace::new(),
        root_uri: Arc::new(Mutex::new(None)),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
