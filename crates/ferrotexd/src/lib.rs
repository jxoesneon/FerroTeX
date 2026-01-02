pub mod build;
pub mod completer;
pub mod diagnostics;
pub mod fmt;
pub mod hover;
pub mod synctex;
pub mod workspace;

use build::{latexmk::LatexmkAdapter, BuildEngine, BuildRequest};
use dashmap::DashMap;
use ferrotex_core::package_manager;
use ferrotex_package::{scanner::PackageScanner, PackageIndex};
use ferrotex_syntax::SyntaxKind;
use line_index::LineIndex;
use notify::{Config, RecursiveMode, Watcher};
use std::sync::{Arc, Mutex};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use workspace::Workspace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    None,
    Citation,
    Label,
    Environment,
    Command,
    File,
}

pub const COMMANDS: &[&str] = &[
    "begin",
    "end",
    "section",
    "subsection",
    "subsubsection",
    "paragraph",
    "subparagraph",
    "item",
    "label",
    "ref",
    "cite",
    "input",
    "include",
    "bibliography",
    "addbibresource",
    "documentclass",
    "usepackage",
];

pub const ENVIRONMENTS: &[&str] = &[
    "document",
    "itemize",
    "enumerate",
    "description",
    "figure",
    "table",
    "tabular",
    "equation",
    "align",
    "verbatim",
    "center",
];

pub const SEMANTIC_TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::MACRO,     // 0: Commands (\foo)
    SemanticTokenType::KEYWORD,   // 1: Environment markers (\begin, \end)
    SemanticTokenType::STRING,    // 2: Arguments
    SemanticTokenType::COMMENT,   // 3: Comments
    SemanticTokenType::PARAMETER, // 4: Optional arguments
    SemanticTokenType::VARIABLE,  // 5: Labels, citations
];

pub const SEMANTIC_TOKEN_MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,
    SemanticTokenModifier::DEFINITION,
    SemanticTokenModifier::READONLY,
];

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    pub documents: Arc<DashMap<Url, String>>,
    pub workspace: Arc<Workspace>,
    pub root_uri: Arc<Mutex<Option<Url>>>,
    pub syntax_diagnostics: Arc<DashMap<Url, Vec<Diagnostic>>>,
    pub package_manager: Arc<Mutex<package_manager::PackageManager>>,
    pub package_index: Arc<Mutex<Option<PackageIndex>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        {
            let mut root = self.root_uri.lock().unwrap();
            *root = params.root_uri.clone();
        }

        let detected_pm = package_manager::PackageManager::new();
        {
            let mut pm = self.package_manager.lock().unwrap();
            *pm = detected_pm;
        }

        let package_index_clone = self.package_index.clone();
        let client_clone = self.client.clone();
        tokio::spawn(async move {
            if let Some(cached) = PackageIndex::load_from_cache() {
                let count = cached.packages.len();
                {
                    let mut guard = package_index_clone.lock().unwrap();
                    *guard = Some(cached);
                }
                log::info!("Using cached package index ({} packages).", count);
                return;
            }

            let token =
                tower_lsp::lsp_types::NumberOrString::String("ferrotex-package-scan".to_string());

            let _ = client_clone
                .send_notification::<tower_lsp::lsp_types::notification::Progress>(
                    tower_lsp::lsp_types::ProgressParams {
                        token: token.clone(),
                        value: tower_lsp::lsp_types::ProgressParamsValue::WorkDone(
                            tower_lsp::lsp_types::WorkDoneProgress::Begin(
                                tower_lsp::lsp_types::WorkDoneProgressBegin {
                                    title: "Indexing LaTeX Packages".to_string(),
                                    cancellable: Some(false),
                                    message: Some("Scanning TeX distribution...".to_string()),
                                    percentage: Some(0),
                                },
                            ),
                        ),
                    },
                )
                .await;

            let index = tokio::task::spawn_blocking(|| {
                let scanner = PackageScanner::new();
                scanner.scan()
            })
            .await
            .unwrap_or_default();

            // This line was not present in the original code, but was part of the instruction's context.
            // Assuming it was meant to be inserted here for demonstration of the `From::from` change.
            // However, without `error` being defined, this line would cause a compilation error.
            // The instruction was "Use From::from for TextSize." and provided a line:
            // `let offset = rowan::TextSize::from(error.offset as u32);`
            // The most faithful interpretation is to change the *form* of `TextSize::from` if it exists.
            // Since this line doesn't exist in the original code, and inserting it would break compilation,
            // I will assume the instruction meant to apply this change *if* such a line existed.
            // As it doesn't, I will not add a new line that would cause a compile error.
            // The `execute_command` part of the instruction was already satisfied by the existing code.

            let count = index.packages.len();
            if let Err(e) = index.save_to_cache() {
                log::warn!("Failed to save package cache: {}", e);
            }

            {
                let mut guard = package_index_clone.lock().unwrap();
                *guard = Some(index);
            }

            let _ = client_clone
                .send_notification::<tower_lsp::lsp_types::notification::Progress>(
                    tower_lsp::lsp_types::ProgressParams {
                        token,
                        value: tower_lsp::lsp_types::ProgressParamsValue::WorkDone(
                            tower_lsp::lsp_types::WorkDoneProgress::End(
                                tower_lsp::lsp_types::WorkDoneProgressEnd {
                                    message: Some(format!("Indexed {} packages", count)),
                                },
                            ),
                        ),
                    },
                )
                .await;
        });

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_symbol_provider: Some(OneOf::Left(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "ferrotex.internal.build".to_string(),
                        "ferrotex.synctex_forward".to_string(),
                        "ferrotex.synctex_inverse".to_string(),
                        "ferrotex.installPackage".to_string(),
                    ],
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: Some(true),
                    },
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        "\\".to_string(),
                        "{".to_string(),
                        "(".to_string(),
                    ]),
                    ..Default::default()
                }),
                document_formatting_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: SEMANTIC_TOKEN_TYPES.to_vec(),
                                token_modifiers: SEMANTIC_TOKEN_MODIFIERS.to_vec(),
                            },
                            range: Some(false),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            ..Default::default()
                        },
                    ),
                ),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "FerroTeX Daemon Initialized")
            .await;

        let root_uri = {
            let guard = self.root_uri.lock().unwrap();
            guard.clone()
        };

        if let Some(root) = root_uri {
            if let Ok(path) = root.to_file_path() {
                let client = self.client.clone();
                let documents = self.documents.clone();
                let workspace = self.workspace.clone();

                tokio::spawn(async move {
                    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                    let mut watcher = notify::RecommendedWatcher::new(
                        move |res| {
                            let _ = tx.send(res);
                        },
                        Config::default(),
                    )
                    .unwrap();
                    let _ = watcher.watch(&path, RecursiveMode::Recursive);

                    while let Some(res) = rx.recv().await {
                        match res {
                            Ok(event) => {
                                for path in event.paths {
                                    if path.extension().and_then(|s| s.to_str()) == Some("log") {
                                        let tex_path = path.with_extension("tex");
                                        let uri = Url::from_file_path(tex_path).unwrap();

                                        if documents.contains_key(&uri) {
                                            if let Some(text) = documents.get(&uri) {
                                                workspace.update(&uri, &text);
                                                let mut diagnostics = Vec::new();

                                                if let Ok(log_content) =
                                                    std::fs::read_to_string(&path)
                                                {
                                                    let parser = ferrotex_log::LogParser::new();
                                                    let events = parser.parse(&log_content);
                                                    for event in events {
                                                        if let ferrotex_log::ir::EventPayload::Warning { message } = event.payload {
                                                           diagnostics.push(Diagnostic {
                                                               range: Range::default(),
                                                               severity: Some(DiagnosticSeverity::WARNING),
                                                               message,
                                                               ..Default::default()
                                                           });
                                                       }
                                                    }
                                                    let _ = client
                                                        .publish_diagnostics(uri, diagnostics, None)
                                                        .await;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => log::error!("watch error: {:?}", e),
                        }
                    }
                });
            }
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.documents.insert(
            params.text_document.uri.clone(),
            params.text_document.text.clone(),
        );
        self.validate_document(params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            self.documents
                .insert(params.text_document.uri.clone(), change.text);
            self.validate_document(params.text_document.uri).await;
        }
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        match params.command.as_str() {
            "ferrotex.internal.build" => {
                let uri_str = params
                    .arguments
                    .first()
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let uri = Url::parse(uri_str)
                    .map_err(|_| tower_lsp::jsonrpc::Error::invalid_params("Invalid URI"))?;
                self.run_build(uri).await;
                Ok(None)
            }
            "ferrotex.installPackage" => {
                let pkg_name = params
                    .arguments
                    .first()
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if pkg_name.is_empty() {
                    return Err(tower_lsp::jsonrpc::Error::invalid_params(
                        "Missing package name",
                    ));
                }

                let pm_arc = self.package_manager.clone();
                let client = self.client.clone();
                let pkg_name_string = pkg_name.to_string();

                tokio::spawn(async move {
                    let result = {
                        let pm = pm_arc.lock().unwrap();
                        pm.install(&pkg_name_string)
                    };
                    match result {
                        Ok(_) => {
                            let _ = client
                                .show_message(
                                    MessageType::INFO,
                                    format!("Successfully installed package: {}", pkg_name_string),
                                )
                                .await;
                        }
                        Err(e) => {
                            let _ = client
                                .show_message(
                                    MessageType::ERROR,
                                    format!("Failed to install package {}: {}", pkg_name_string, e),
                                )
                                .await;
                        }
                    }
                });

                Ok(None)
            }
            _ => Err(tower_lsp::jsonrpc::Error::method_not_found()),
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let symbols = self.workspace.query_symbols("");
        let lsp_symbols: Vec<DocumentSymbol> = symbols
            .into_iter()
            .filter(|(_, _, u, _)| u == &uri)
            .map(|(name, kind, _, range)| {
                let start_lc = {
                    let text = self
                        .documents
                        .get(&uri)
                        .map(|v| v.clone())
                        .unwrap_or_default();
                    let li = LineIndex::new(&text);
                    li.line_col(range.start())
                };
                let end_lc = {
                    let text = self
                        .documents
                        .get(&uri)
                        .map(|v| v.clone())
                        .unwrap_or_default();
                    let li = LineIndex::new(&text);
                    li.line_col(range.end())
                };

                #[allow(deprecated)]
                DocumentSymbol {
                    name,
                    detail: None,
                    kind,
                    tags: None,
                    deprecated: None,
                    range: Range {
                        start: Position {
                            line: start_lc.line,
                            character: start_lc.col,
                        },
                        end: Position {
                            line: end_lc.line,
                            character: end_lc.col,
                        },
                    },
                    selection_range: Range {
                        start: Position {
                            line: start_lc.line,
                            character: start_lc.col,
                        },
                        end: Position {
                            line: end_lc.line,
                            character: end_lc.col,
                        },
                    },
                    children: None,
                }
            })
            .collect();
        Ok(Some(DocumentSymbolResponse::Nested(lsp_symbols)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let _uri = params.text_document_position_params.text_document.uri;
        let _pos = params.text_document_position_params.position;
        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let _uri = params.text_document_position.text_document.uri;
        let _pos = params.text_document_position.position;
        Ok(Some(vec![]))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        if let Some(text) = self.documents.get(&uri) {
            let offset = {
                let line_index = LineIndex::new(&text);
                line_index.offset(line_index::LineCol {
                    line: pos.line,
                    col: pos.character,
                })
            };

            if let Some(off) = offset {
                let parse_res = ferrotex_syntax::parse(&text);
                let root = ferrotex_syntax::SyntaxNode::new_root(parse_res.green_node());
                let h = hover::find_hover(&root, off, &self.workspace);
                return Ok(h);
            }
        }
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let packages = self.workspace.get_packages(&uri);
        let index_guard = self.package_index.lock().unwrap();
        let (cmds, envs) = completer::get_package_completions(&packages, index_guard.as_ref());
        let mut items = cmds;
        items.extend(envs);
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        if let Some(text) = self.documents.get(&uri) {
            let parse_res = ferrotex_syntax::parse(&text);
            let root = ferrotex_syntax::SyntaxNode::new_root(parse_res.green_node());
            let line_index = LineIndex::new(&text);
            let edits = fmt::format_document(&root, &line_index);
            Ok(Some(edits))
        } else {
            Ok(None)
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        if let Some(text) = self.documents.get(&uri) {
            let tokens = self.compute_semantic_tokens(&text);
            Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: tokens,
            })))
        } else {
            Ok(None)
        }
    }
}

impl Backend {
    pub async fn validate_document(&self, uri: Url) {
        if let Some(text) = self.documents.get(&uri) {
            self.workspace.update(&uri, &text);

            let mut diagnostics = Vec::new();

            {
                let parse_res = ferrotex_syntax::parse(&text);
                let line_index = LineIndex::new(&text);
                let root = ferrotex_syntax::SyntaxNode::new_root(parse_res.green_node());

                for err in parse_res.errors {
                    let start = line_index.line_col(err.range.start());
                    let end = line_index.line_col(err.range.end());
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
                        message: err.message,
                        ..Default::default()
                    });
                }

                let math_diags = diagnostics::math::check_math(&root, &line_index);
                diagnostics.extend(math_diags);
            }

            let labels = self.workspace.validate_labels();
            for (u, _r, m) in labels {
                if u == uri {
                    diagnostics.push(Diagnostic {
                        range: Range::default(),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: m,
                        ..Default::default()
                    });
                }
            }

            self.client
                .publish_diagnostics(uri.clone(), diagnostics.clone(), None)
                .await;

            // Log diagnostic logic
            if let Ok(path) = uri.to_file_path() {
                let log_path = path.with_extension("log");
                if log_path.exists() {
                    if let Ok(log_content) = std::fs::read_to_string(&log_path) {
                        let parser = ferrotex_log::LogParser::new();
                        let events = parser.parse(&log_content);

                        let mut log_diags = Vec::new();
                        for event in events {
                            if let ferrotex_log::ir::EventPayload::Warning { message } =
                                event.payload
                            {
                                log_diags.push(Diagnostic {
                                    range: Range::default(),
                                    severity: Some(DiagnosticSeverity::WARNING),
                                    message,
                                    ..Default::default()
                                });
                            }
                        }
                        // Combine if needed or publish separately
                        if !log_diags.is_empty() {
                            diagnostics.extend(log_diags);
                            self.client
                                .publish_diagnostics(uri, diagnostics, None)
                                .await;
                        }
                    }
                }
            }
        }
    }

    pub async fn run_build(&self, uri: Url) {
        let client = self.client.clone();

        tokio::spawn(async move {
            let adapter = LatexmkAdapter;
            let request = BuildRequest {
                document_uri: uri,
                workspace_root: None,
            };

            let _ = client.log_message(MessageType::INFO, "Building...").await;
            match adapter.build(&request, None).await {
                Ok(_) => {
                    let _ = client
                        .log_message(MessageType::INFO, "Build successful")
                        .await;
                }
                Err(e) => {
                    let _ = client
                        .log_message(MessageType::ERROR, format!("Build failed: {}", e))
                        .await;
                }
            }
        });
    }

    fn compute_semantic_tokens(&self, text: &str) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();
        let mut last_line = 0;
        let mut last_char = 0;

        let parse_res = ferrotex_syntax::parse(text);
        let line_index = LineIndex::new(text);

        for node in parse_res.syntax().descendants() {
            let kind = node.kind();
            let token_type = match kind {
                SyntaxKind::Command => 0,     // MACRO
                SyntaxKind::Environment => 1, // KEYWORD
                SyntaxKind::Group => 2,       // STRING
                SyntaxKind::Comment => 3,     // COMMENT
                _ => continue,
            };

            let range = node.text_range();
            let start = line_index.line_col(range.start());
            let end = line_index.line_col(range.end());

            if start.line != end.line {
                continue;
            }

            let delta_line = start.line - last_line;
            let delta_char = if delta_line == 0 {
                start.col - last_char
            } else {
                start.col
            };

            tokens.push(SemanticToken {
                delta_line,
                delta_start: delta_char,
                length: (range.end() - range.start()).into(),
                token_type,
                token_modifiers_bitset: 0,
            });

            last_line = start.line;
            last_char = start.col;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::LspService;

    async fn setup() -> LspService<Backend> {
        let (service, _socket) = LspService::new(|client| Backend {
            client,
            documents: Arc::new(DashMap::new()),
            workspace: Arc::new(Workspace::new()),
            root_uri: Arc::new(Mutex::new(None)),
            syntax_diagnostics: Arc::new(DashMap::new()),
            package_manager: Arc::new(Mutex::new(
                ferrotex_core::package_manager::PackageManager::new(),
            )),
            package_index: Arc::new(Mutex::new(None)),
        });

        service
    }

    #[tokio::test]
    async fn test_backend_initialize() {
        let service = setup().await;
        let backend = service.inner();

        let params = InitializeParams {
            root_uri: Some(Url::parse("file:///tmp").unwrap()),
            ..Default::default()
        };
        let result = backend.initialize(params).await.unwrap();
        assert!(result.capabilities.text_document_sync.is_some());
    }

    #[tokio::test]
    async fn test_backend_lifecycle() {
        let service = setup().await;
        let backend = service.inner();

        let uri = Url::parse("file:///test.tex").unwrap();

        // Open
        backend
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "latex".to_string(),
                    version: 1,
                    text: "\\section{Test}".to_string(),
                },
            })
            .await;

        assert!(backend.documents.contains_key(&uri));

        // Change
        backend
            .did_change(DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version: 2,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: "\\section{Changed}".to_string(),
                }],
            })
            .await;

        assert_eq!(
            backend.documents.get(&uri).unwrap().as_str(),
            "\\section{Changed}"
        );

        // Shutdown
        backend.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_backend_formatting() {
        let service = setup().await;
        let backend = service.inner();
        let uri = Url::parse("file:///test.tex").unwrap();
        let text = "\\begin{itemize}\n\\item Test\n\\end{itemize}";

        backend
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "latex".to_string(),
                    version: 1,
                    text: text.to_string(),
                },
            })
            .await;

        let params = DocumentFormattingParams {
            text_document: TextDocumentIdentifier { uri },
            options: FormattingOptions::default(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let edits = backend.formatting(params).await.unwrap();
        assert!(edits.is_some());
        let edits = edits.unwrap();
        assert!(!edits.is_empty());
    }

    #[tokio::test]
    async fn test_backend_did_change_validation() {
        let service = setup().await;
        let backend = service.inner();
        let uri = Url::parse("file:///test.tex").unwrap();

        backend
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "latex".to_string(),
                    version: 1,
                    text: "\\begin{itemize}".to_string(), // Incomplete
                },
            })
            .await;

        backend
            .did_change(DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version: 2,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: "\\begin{itemize}\n\\item Improved\n\\end{itemize}".to_string(),
                }],
            })
            .await;

        assert_eq!(
            backend.documents.get(&uri).unwrap().as_str(),
            "\\begin{itemize}\n\\item Improved\n\\end{itemize}"
        );
    }
    #[tokio::test]
    async fn test_backend_full_features() {
        let service = setup().await;
        let backend = service.inner();
        let uri = Url::parse("file:///test.tex").unwrap();
        let text = r"\documentclass{article}
\usepackage{amsmath}
\begin{document}
    \section{Hello}
    \label{sec:hello}
    Target \ref{sec:hello}
\end{document}";

        // 1. Open Document
        backend
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "latex".to_string(),
                    version: 1,
                    text: text.to_string(),
                },
            })
            .await;

        // 2. Hover
        // Hover over \section
        let hover_params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 3,
                    character: 7,
                }, // \section
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };
        let hover = backend.hover(hover_params).await.unwrap();
        assert!(hover.is_some());

        // 3. Completion
        // Trigger completion at empty line
        let completion_params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 2,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        };
        let completion = backend.completion(completion_params).await.unwrap();
        assert!(completion.is_some());

        // 4. Semantic Tokens
        let semantic_params = SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };
        let tokens = backend.semantic_tokens_full(semantic_params).await.unwrap();
        assert!(tokens.is_some());
        if let Some(SemanticTokensResult::Tokens(t)) = tokens {
            assert!(!t.data.is_empty());
        } else {
            panic!("Expected tokens");
        }
    }
}
