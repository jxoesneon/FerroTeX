mod build;
mod fmt;
mod workspace;
mod hover;

mod synctex;

use build::{BuildEngine, BuildRequest, BuildStatus, latexmk::LatexmkAdapter};
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

enum CompletionKind {
    None,
    Citation,
    Label,
    Environment,
    Command,
    File,
}

const COMMANDS: &[&str] = &[
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

const ENVIRONMENTS: &[&str] = &[
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

const SEMANTIC_TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::MACRO,     // 0: Commands (\foo)
    SemanticTokenType::KEYWORD,   // 1: Environment markers (\begin, \end)
    SemanticTokenType::STRING,    // 2: Arguments
    SemanticTokenType::COMMENT,   // 3: Comments
    SemanticTokenType::PARAMETER, // 4: Optional arguments
    SemanticTokenType::VARIABLE,  // 5: Labels, citations
];

const SEMANTIC_TOKEN_MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,
    SemanticTokenModifier::DEFINITION,
    SemanticTokenModifier::READONLY,
];

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<DashMap<Url, String>>,
    workspace: Arc<Workspace>,
    root_uri: Arc<Mutex<Option<Url>>>,
    syntax_diagnostics: Arc<DashMap<Url, Vec<Diagnostic>>>,
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
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "ferrotex.build".to_string(),
                        "ferrotex.synctex_forward".to_string(),
                        "ferrotex.synctex_inverse".to_string()
                    ],
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: Some(true),
                    },
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        "{".to_string(),
                        ",".to_string(),
                        "\\".to_string(),
                        "/".to_string(),
                    ]),
                    ..Default::default()
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            work_done_progress_options: WorkDoneProgressOptions::default(),
                            legend: SemanticTokensLegend {
                                token_types: SEMANTIC_TOKEN_TYPES.to_vec(),
                                token_modifiers: SEMANTIC_TOKEN_MODIFIERS.to_vec(),
                            },
                            range: Some(false),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        },
                    ),
                ),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "ferrotexd initialized!")
            .await;

        // Start watching workspace (logs and tex files)
        let client = self.client.clone();
        let root_uri = self.root_uri.clone();
        let workspace = self.workspace.clone();
        let documents = self.documents.clone();
        let syntax_diagnostics = self.syntax_diagnostics.clone();

        tokio::spawn(async move {
            if let Err(e) =
                watch_workspace(client, root_uri, workspace, documents, syntax_diagnostics).await
            {
                eprintln!("Error watching workspace: {:?}", e);
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
        if uri.path().ends_with(".bib") {
            self.workspace.update_bib(&uri, &text);
            // Re-validate all diagnostics to clear "undefined citation" errors
            self.publish_all_diagnostics().await;
        } else {
            self.workspace.update(&uri, &text);
            self.validate_document(&uri, &text).await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // With FULL sync, content changes are the full text
        if let Some(change) = params.content_changes.into_iter().last() {
            let uri = params.text_document.uri;
            let text = change.text;

            self.documents.insert(uri.clone(), text.clone());
            if uri.path().ends_with(".bib") {
                self.workspace.update_bib(&uri, &text);
                self.publish_all_diagnostics().await;
            } else {
                self.workspace.update(&uri, &text);
                self.validate_document(&uri, &text).await;
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.remove(&uri);
        // Clear diagnostics for the closed file
        self.client.publish_diagnostics(uri, vec![], None).await;
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

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let text = if let Some(t) = self.get_text(&uri) {
            t
        } else {
            return Ok(None);
        };

        let label_name = self.find_label_at_position(&text, position);

        if let Some(name) = label_name {
            let defs = self.workspace.find_definitions(&name);
            let mut locations = Vec::new();
            for (def_uri, range) in defs {
                if let Some(loc) = self.range_to_location(&def_uri, range) {
                    locations.push(loc);
                }
            }
            Ok(Some(GotoDefinitionResponse::Array(locations)))
        } else {
            Ok(None)
        }
    }

    /// Finds references to the symbol at the given position.
    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let text = if let Some(t) = self.get_text(&uri) {
            t
        } else {
            return Ok(None);
        };

        let label_name = self.find_label_at_position(&text, position);

        if let Some(name) = label_name {
            let mut locations = Vec::new();

            // 1. References
            let refs = self.workspace.find_references(&name);
            for (ref_uri, range) in refs {
                if let Some(loc) = self.range_to_location(&ref_uri, range) {
                    locations.push(loc);
                }
            }

            // 2. Include definitions if requested (usually yes)
            if params.context.include_declaration {
                let defs = self.workspace.find_definitions(&name);
                for (def_uri, range) in defs {
                    if let Some(loc) = self.range_to_location(&def_uri, range) {
                        locations.push(loc);
                    }
                }
            }

            Ok(Some(locations))
        } else {
            Ok(None)
        }
    }

    /// Renames the symbol at the given position.
    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        let text = if let Some(t) = self.get_text(&uri) {
            t
        } else {
            return Ok(None);
        };

        let label_name = self.find_label_at_position(&text, position);

        if let Some(name) = label_name {
            let mut changes: std::collections::HashMap<Url, Vec<TextEdit>> =
                std::collections::HashMap::new();

            // 1. Rename definitions
            let defs = self.workspace.find_definitions(&name);
            for (def_uri, range) in defs {
                if let Some(loc) = self.range_to_location(&def_uri, range) {
                    changes.entry(def_uri).or_default().push(TextEdit {
                        range: loc.range,
                        new_text: new_name.clone(),
                    });
                }
            }

            // 2. Rename references
            let refs = self.workspace.find_references(&name);
            for (ref_uri, range) in refs {
                if let Some(loc) = self.range_to_location(&ref_uri, range) {
                    changes.entry(ref_uri).or_default().push(TextEdit {
                        range: loc.range,
                        new_text: new_name.clone(),
                    });
                }
            }

            Ok(Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }))
        } else {
            Ok(None)
        }
    }

    /// Provides code completion suggestions.
    ///
    /// Currently supports:
    /// - Citation keys inside `\cite{...}`
    /// - Label names inside `\ref{...}`
    /// - Environment names inside `\begin{...}` / `\end{...}`
    /// - Commands (starting with `\`)
    /// - File paths inside `\input{...}` / `\include{...}`
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let text = if let Some(t) = self.get_text(&uri) {
            t
        } else {
            return Ok(None);
        };

        let line_index = LineIndex::new(&text);
        let offset = if let Some(o) = line_index.offset(line_index::LineCol {
            line: position.line,
            col: position.character,
        }) {
            ferrotex_syntax::TextSize::from(u32::from(o))
        } else {
            return Ok(None);
        };

        let parse = ferrotex_syntax::parse(&text);
        let root = parse.syntax();

        let token = match root.token_at_offset(offset) {
            rowan::TokenAtOffset::None => return Ok(None),
            rowan::TokenAtOffset::Single(t) => t,
            rowan::TokenAtOffset::Between(l, r) => {
                // Prefer the token that suggests we are extending an identifier or starting a group
                if l.kind() == SyntaxKind::Command
                    || l.kind() == SyntaxKind::Text
                    || l.text() == "{"
                    || l.text() == ","
                    || l.text() == "/"
                {
                    l
                } else {
                    r
                }
            }
        };

        let kind = determine_completion_kind(&token);

        match kind {
            CompletionKind::Citation => {
                let keys = self.workspace.get_all_citation_keys();
                let items: Vec<CompletionItem> = keys
                    .into_iter()
                    .map(|key| CompletionItem {
                        label: key,
                        kind: Some(CompletionItemKind::REFERENCE),
                        detail: Some("Citation".to_string()),
                        ..Default::default()
                    })
                    .collect();
                Ok(Some(CompletionResponse::Array(items)))
            }
            CompletionKind::Label => {
                let labels = self.workspace.get_all_labels();
                let items: Vec<CompletionItem> = labels
                    .into_iter()
                    .map(|label| CompletionItem {
                        label,
                        kind: Some(CompletionItemKind::REFERENCE),
                        detail: Some("Label".to_string()),
                        ..Default::default()
                    })
                    .collect();
                Ok(Some(CompletionResponse::Array(items)))
            }
            CompletionKind::Environment => {
                let items: Vec<CompletionItem> = ENVIRONMENTS
                    .iter()
                    .map(|&env| CompletionItem {
                        label: env.to_string(),
                        kind: Some(CompletionItemKind::SNIPPET),
                        detail: Some("Environment".to_string()),
                        ..Default::default()
                    })
                    .collect();
                Ok(Some(CompletionResponse::Array(items)))
            }
            CompletionKind::Command => {
                let items: Vec<CompletionItem> = COMMANDS
                    .iter()
                    .map(|&cmd| CompletionItem {
                        label: format!("\\{}", cmd),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some("Command".to_string()),
                        ..Default::default()
                    })
                    .collect();
                Ok(Some(CompletionResponse::Array(items)))
            }
            CompletionKind::File => {
                let items = if let Some(root_uri) = self.root_uri.lock().unwrap().as_ref() {
                    if let Ok(path) = root_uri.to_file_path() {
                        scan_files_for_completion(&path)
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };
                Ok(Some(CompletionResponse::Array(items)))
            }
            CompletionKind::None => Ok(None),
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let text = if let Some(t) = self.get_text(&uri) {
            t
        } else {
            return Ok(None);
        };

        let parse = ferrotex_syntax::parse(&text);
        let root = parse.syntax();
        let line_index = LineIndex::new(&text);

        let tokens = extract_semantic_tokens(root, &line_index);

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        })))
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let uri = params.text_document.uri;
        let text = if let Some(t) = self.get_text(&uri) {
            t
        } else {
            return Ok(None);
        };

        let parse = ferrotex_syntax::parse(&text);
        let root = parse.syntax();
        let line_index = LineIndex::new(&text);

        let ranges = extract_folding_ranges(root, &line_index);
        Ok(Some(ranges))
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = params.query;
        let symbols = self.workspace.query_symbols(&query);

        let result = symbols
            .into_iter()
            .filter_map(|(name, kind, uri, range)| {
                self.range_to_location(&uri, range).map(|loc| {
                    #[allow(deprecated)]
                    SymbolInformation {
                        name,
                        kind,
                        tags: None,
                        deprecated: None,
                        location: loc,
                        container_name: None,
                    }
                })
            })
            .collect();

        Ok(Some(result))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let text = if let Some(t) = self.get_text(&uri) {
            t
        } else {
            return Ok(None);
        };

        let parse = ferrotex_syntax::parse(&text);
        let root = parse.syntax();
        let line_index = LineIndex::new(&text);

        Ok(Some(fmt::format_document(&root, &line_index)))
    }

    async fn code_action(&self, _params: CodeActionParams) -> Result<Option<Vec<CodeActionOrCommand>>> {
        let actions = vec![
            // Stub for future: e.g. "Create missing label"
        ];
        Ok(Some(actions))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        if let Some(text) = self.documents.get(uri) {
            let parse = ferrotex_syntax::parse(&text);
            let root = parse.syntax();
            let line_index = LineIndex::new(&text);

            let offset = line_index
                .offset(line_index::LineCol {
                    line: params.text_document_position_params.position.line,
                    col: params.text_document_position_params.position.character,
                })
                .unwrap();

            return Ok(hover::find_hover(&root, offset));
        }

        Ok(None)
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        if params.command == "ferrotex.synctex_forward" {
             let args = params.arguments;
             if args.len() < 4 { return Ok(None); }
             let tex_uri_str = args[0].as_str().unwrap_or_default();
             let line = args[1].as_u64().unwrap_or(0) as u32;
             let col = args[2].as_u64().unwrap_or(0) as u32;
             let pdf_uri_str = args[3].as_str().unwrap_or_default();
             
             if let Ok(tex_url) = Url::parse(tex_uri_str) {
                 if let Ok(pdf_url) = Url::parse(pdf_uri_str) {
                     if let (Ok(tex_path), Ok(pdf_path)) = (tex_url.to_file_path(), pdf_url.to_file_path()) {
                         if let Some(res) = synctex::forward_search(&tex_path, &pdf_path, line, col) {
                             return Ok(Some(serde_json::to_value(res).unwrap()));
                         }
                     }
                 }
             }
             return Ok(None);
        }
        if params.command == "ferrotex.synctex_inverse" {
             let args = params.arguments;
             if args.len() < 4 { return Ok(None); }
             let pdf_uri_str = args[0].as_str().unwrap_or_default();
             let page = args[1].as_u64().unwrap_or(1) as u32;
             let x = args[2].as_f64().unwrap_or(0.0);
             let y = args[3].as_f64().unwrap_or(0.0);
             
             if let Ok(pdf_url) = Url::parse(pdf_uri_str) {
                 if let Ok(pdf_path) = pdf_url.to_file_path() {
                     if let Some(res) = synctex::inverse_search(&pdf_path, page, x, y) {
                         return Ok(Some(serde_json::to_value(res).unwrap()));
                     }
                 }
             }
             return Ok(None);
        }

        if params.command == "ferrotex.build" {
            let args = params.arguments;
            if args.is_empty() {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        "Build command missing file URI argument",
                    )
                    .await;
                return Ok(None);
            }

            // Extract URI from arguments
            let uri_str = args[0].as_str().unwrap_or_default();
            let uri = match Url::parse(uri_str) {
                Ok(u) => u,
                Err(_) => {
                    self.client
                        .log_message(MessageType::ERROR, "Invalid URI argument")
                        .await;
                    return Ok(None);
                }
            };

            self.client
                .log_message(MessageType::INFO, format!("Starting build for: {}", uri))
                .await;

            // --- Status Bar: Begin Progress (UX-4) ---
            let token = NumberOrString::String("ferrotex-build".to_string());

            // 1. Create Progress
            let create_params = WorkDoneProgressCreateParams {
                token: token.clone(),
            };
            let _ = self
                .client
                .send_request::<request::WorkDoneProgressCreate>(create_params)
                .await;

            // 2. Begin Progress
            let begin_params = ProgressParams {
                token: token.clone(),
                value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(
                    WorkDoneProgressBegin {
                        title: "Building PDF...".to_string(),
                        cancellable: Some(false),
                        message: Some("Resolving root...".to_string()),
                        percentage: None,
                    },
                )),
            };
            self.client
                .send_notification::<notification::Progress>(begin_params)
                .await;
            // ---------------------------------

            // --- Magic Comment Detection (UX-2) ---
            let mut build_uri = uri.clone();
            if let Some(text) = self.get_text(&uri) {
                if let Some(magic_path) = detect_magic_root(&text) {
                    if let Ok(file_path) = uri.to_file_path() {
                        if let Some(parent) = file_path.parent() {
                            let new_path = parent.join(&magic_path);
                            // Normalize if possible, but for now strict join
                            if let Ok(new_uri) = Url::from_file_path(&new_path) {
                                self.client
                                    .log_message(
                                        MessageType::INFO,
                                        format!(
                                            "Magic Root detected: Redirecting build to {}",
                                            new_uri
                                        ),
                                    )
                                    .await;
                                build_uri = new_uri;
                            }
                        }
                    }
                }
            }
            // --------------------------------------

            // Resolve workspace root
            let root_path = {
                let root = self.root_uri.lock().unwrap();
                root.as_ref().and_then(|u| u.to_file_path().ok())
            };

            let req = BuildRequest {
                document_uri: build_uri,
                workspace_root: root_path,
            };

            let engine = LatexmkAdapter;
            let result = engine.build(&req).await;

            match result {
                Ok(status) => match status {
                    BuildStatus::Success(artifact) => {
                        let msg = format!("Build Succeeded! Artifact: {:?}", artifact);
                        self.client.log_message(MessageType::INFO, &msg).await;
                        // UX-5: Success Notification
                        self.client
                            .show_message(MessageType::INFO, "Build Succeeded 🎉")
                            .await;
                    }
                    BuildStatus::Failure(log) => {
                        self.client
                            .log_message(MessageType::ERROR, "Build Failed")
                            .await;
                        // Stream stderr to log
                        for line in log.stderr.lines() {
                            self.client.log_message(MessageType::ERROR, line).await;
                        }
                        // UX-5: Failure Notification
                        self.client
                            .show_message(MessageType::ERROR, "Build Failed ❌ (Check Output)")
                            .await;

                        // BO-9: Missing Package Detection
                        // Check stdout (latexmk usually captures tex output there) and stderr
                        let combined_log = format!("{}\n{}", log.stdout, log.stderr);
                        if let Some(pkg) = detect_missing_package(&combined_log) {
                             let action = self.client.show_message_request(
                                MessageType::WARNING, 
                                format!("Package '{}' seems to be missing.", pkg), 
                                Some(vec![MessageActionItem { title: format!("Install {}", pkg), properties: Default::default() }])
                            ).await;
                            
                            if let Ok(Some(item)) = action {
                                if item.title.starts_with("Install") {
                                    self.client.show_message(MessageType::INFO, "Installing package... Check logs.").await;
                                    if install_package(&self.client, &pkg).await {
                                         self.client.show_message(MessageType::INFO, "Package installed. Try building again.").await;
                                    } else {
                                         self.client.show_message(MessageType::ERROR, "Installation failed. See logs.").await;
                                    }
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    let err_msg = format!("Build execution error: {}", e);
                    self.client.log_message(MessageType::ERROR, &err_msg).await;
                    self.client
                        .show_message(MessageType::ERROR, "Build Error 💥")
                        .await;
                }
            }

            // --- Status Bar: End Progress ---
            let end_params = ProgressParams {
                token: token.clone(),
                value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(WorkDoneProgressEnd {
                    message: None,
                })),
            };
            self.client
                .send_notification::<notification::Progress>(end_params)
                .await;
            // -------------------------------
        }
        Ok(None)
    }
}

fn extract_folding_ranges(root: SyntaxNode, line_index: &LineIndex) -> Vec<FoldingRange> {
    let mut ranges = Vec::new();
    for node in root.descendants() {
        match node.kind() {
            SyntaxKind::Environment | SyntaxKind::Group => {
                let text_range = node.text_range();
                let start = line_index.line_col(text_range.start());
                let end = line_index.line_col(text_range.end());

                // Only fold if it spans multiple lines
                if start.line < end.line {
                    ranges.push(FoldingRange {
                        start_line: start.line,
                        start_character: Some(start.col),
                        end_line: end.line,
                        end_character: Some(end.col),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: None,
                    });
                }
            }
            _ => {}
        }
    }
    ranges
}

fn extract_semantic_tokens(root: SyntaxNode, line_index: &LineIndex) -> Vec<SemanticToken> {
    let mut tokens = Vec::new();
    let mut prev_line = 0;
    let mut prev_start = 0;

    for event in root.preorder_with_tokens() {
        if let rowan::WalkEvent::Enter(rowan::NodeOrToken::Token(token)) = event
            && let Some((type_idx, modifier_bitset)) = classify_token(&token)
        {
            let range = token.text_range();
            let start_pos = line_index.line_col(range.start());
            let length = range.len().into();

            let delta_line = start_pos.line - prev_line;
            let delta_start = if delta_line == 0 {
                start_pos.col - prev_start
            } else {
                start_pos.col
            };

            tokens.push(SemanticToken {
                delta_line,
                delta_start,
                length,
                token_type: type_idx,
                token_modifiers_bitset: modifier_bitset,
            });

            prev_line = start_pos.line;
            prev_start = start_pos.col;
        }
    }

    tokens
}

fn classify_token(token: &ferrotex_syntax::SyntaxToken) -> Option<(u32, u32)> {
    let kind = token.kind();
    let text = token.text();
    let parent = token.parent()?;

    match kind {
        SyntaxKind::Command => {
            if text == "\\begin" || text == "\\end" {
                Some((1, 0)) // KEYWORD
            } else {
                Some((0, 0)) // MACRO
            }
        }
        SyntaxKind::Comment => Some((3, 0)), // COMMENT
        SyntaxKind::Text => {
            // Check context
            if parent.kind() == SyntaxKind::Group {
                if let Some(grandparent) = parent.parent() {
                    match grandparent.kind() {
                        SyntaxKind::Environment => Some((2, 0)), // STRING (env name)
                        SyntaxKind::LabelDefinition => Some((5, 1 << 1)), // VARIABLE | DEFINITION
                        SyntaxKind::LabelReference => Some((5, 1 << 2)), // VARIABLE | READONLY
                        SyntaxKind::Citation => Some((5, 1 << 2)), // VARIABLE | READONLY
                        SyntaxKind::Section => Some((2, 0)),     // STRING (Section title)
                        SyntaxKind::Include => Some((2, 0)),     // STRING (Path)
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                // Handle text that is directly a child of special nodes (e.g. optional args in brackets)
                // In ferrotex-syntax, items in brackets are just children of the Citation/Bib node
                match parent.kind() {
                    SyntaxKind::Citation | SyntaxKind::Bibliography => {
                        // If it's inside brackets?
                        // Simple heuristic: if it's not the command and not the brace-group
                        // But wait, ferrotex-syntax parser puts bracketed content as siblings.
                        // We can't easily check siblings here without walking.
                        // But we know 'Text' inside Citation but NOT in a Group is likely the optional arg.
                        Some((4, 0)) // PARAMETER
                    }
                    _ => None,
                }
            }
        }
        _ => None,
    }
}

fn determine_completion_kind(token: &ferrotex_syntax::SyntaxToken) -> CompletionKind {
    let kind = token.kind();
    let text = token.text();
    let parent = token.parent();

    if kind == SyntaxKind::Command || text == "\\" {
        return CompletionKind::Command;
    }

    if let Some(parent) = parent {
        if parent.kind() == SyntaxKind::Group {
            if let Some(grandparent) = parent.parent() {
                match grandparent.kind() {
                    SyntaxKind::Citation => return CompletionKind::Citation,
                    SyntaxKind::LabelReference => return CompletionKind::Label,
                    SyntaxKind::Environment => return CompletionKind::Environment,
                    SyntaxKind::Include => return CompletionKind::File,
                    _ => {}
                }
            }
        } else if kind == SyntaxKind::LBrace {
            match parent.kind() {
                SyntaxKind::Environment => return CompletionKind::Environment,
                SyntaxKind::Citation => return CompletionKind::Citation,
                SyntaxKind::LabelReference => return CompletionKind::Label,
                SyntaxKind::Include => return CompletionKind::File,
                _ => {}
            }
        }
    }

    // Fallback for Text inside Group
    if kind == SyntaxKind::Text
        && let Some(parent) = token.parent()
        && parent.kind() == SyntaxKind::Group
        && let Some(grandparent) = parent.parent()
    {
        match grandparent.kind() {
            SyntaxKind::Citation => return CompletionKind::Citation,
            SyntaxKind::LabelReference => return CompletionKind::Label,
            SyntaxKind::Environment => return CompletionKind::Environment,
            SyntaxKind::Include => return CompletionKind::File,
            _ => {}
        }
    }

    CompletionKind::None
}

fn scan_files_for_completion(root_path: &std::path::Path) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let mut stack = vec![root_path.to_path_buf()];

    // Limit depth or count to avoid infinite loops or huge delays
    let mut count = 0;
    const MAX_FILES: usize = 1000;

    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Simple skip of hidden dirs
                    if let Some(name) = path.file_name().and_then(|n| n.to_str())
                        && !name.starts_with('.')
                    {
                        stack.push(path);
                    }
                } else if let Some(ext) = path.extension()
                    && ext == "tex"
                    && let Ok(rel) = path.strip_prefix(root_path)
                    && let Some(s) = rel.to_str()
                {
                    // Strip extension for standard LaTeX includes
                    let label = s.trim_end_matches(".tex").to_string();
                    items.push(CompletionItem {
                        label,
                        kind: Some(CompletionItemKind::FILE),
                        detail: Some("File".to_string()),
                        ..Default::default()
                    });
                    count += 1;
                    if count >= MAX_FILES {
                        return items;
                    }
                }
            }
        }
    }
    items
}

impl Backend {
    /// Validates a single document and updates its diagnostics.
    async fn validate_document(&self, uri: &Url, text: &str) {
        let parse = ferrotex_syntax::parse(text);
        let line_index = LineIndex::new(text);

        let diagnostics: Vec<Diagnostic> = parse
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

        self.syntax_diagnostics.insert(uri.clone(), diagnostics);

        self.publish_all_diagnostics().await;
    }

    /// Refreshes diagnostics for all open documents.
    ///
    /// This includes syntax errors, cycle detection, duplicate labels, and undefined citations.
    async fn publish_all_diagnostics(&self) {
        publish_diagnostics_logic(
            &self.client,
            &self.workspace,
            &self.documents,
            &self.syntax_diagnostics,
        )
        .await;
    }

    /// Helper to find the label name under the cursor.
    fn find_label_at_position(&self, text: &str, position: Position) -> Option<String> {
        let line_index = LineIndex::new(text);
        let offset = line_index.offset(line_index::LineCol {
            line: position.line,
            col: position.character,
        })?;

        let offset = ferrotex_syntax::TextSize::from(u32::from(offset));
        let parse = ferrotex_syntax::parse(text);
        let root = parse.syntax();

        // Use token_at_offset to find the leaf
        let token = root.token_at_offset(offset).right_biased()?;

        // Walk up to find LabelDefinition or LabelReference
        let mut node = Some(token.parent()?);
        while let Some(n) = node {
            match n.kind() {
                SyntaxKind::LabelDefinition | SyntaxKind::LabelReference => {
                    return workspace::extract_group_text(&n);
                }
                SyntaxKind::Root => break,
                _ => node = n.parent(),
            }
        }
        None
    }

    /// Converts a text range (syntax-level) to an LSP location.
    fn range_to_location(&self, uri: &Url, range: ferrotex_syntax::TextRange) -> Option<Location> {
        let text = self.get_text(uri)?;
        let line_index = LineIndex::new(&text);
        let start = line_index.line_col(range.start());
        let end = line_index.line_col(range.end());

        Some(Location {
            uri: uri.clone(),
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
        })
    }

    /// Helper to get text for a URI (open or read from disk)
    fn get_text(&self, uri: &Url) -> Option<String> {
        if let Some(text) = self.documents.get(uri) {
            Some(text.clone())
        } else if let Ok(path) = uri.to_file_path() {
            std::fs::read_to_string(path).ok()
        } else {
            None
        }
    }
}

/// Core logic for aggregating and publishing diagnostics.
async fn publish_diagnostics_logic(
    client: &Client,
    workspace: &Workspace,
    documents: &DashMap<Url, String>,
    syntax_diagnostics: &DashMap<Url, Vec<Diagnostic>>,
) {
    // 1. Collect all project-level diagnostics
    let cycles = workspace.detect_cycles();
    let label_errors = workspace.validate_labels();
    let bibliography_errors = workspace.validate_bibliographies();
    let citation_errors = workspace.validate_citations();

    // Group by URI: Map<Url, Vec<(Range, Message, Source)>>
    let mut project_diags: std::collections::HashMap<
        Url,
        Vec<(ferrotex_syntax::TextRange, String, String)>,
    > = std::collections::HashMap::new();

    for (uri, range, msg) in cycles {
        project_diags
            .entry(uri)
            .or_default()
            .push((range, msg, "ferrotex-project".into()));
    }
    for (uri, range, msg) in label_errors {
        project_diags
            .entry(uri)
            .or_default()
            .push((range, msg, "ferrotex-labels".into()));
    }
    for (uri, range, msg) in bibliography_errors {
        project_diags
            .entry(uri)
            .or_default()
            .push((range, msg, "ferrotex-bibliography".into()));
    }
    for (uri, range, msg) in citation_errors {
        project_diags
            .entry(uri)
            .or_default()
            .push((range, msg, "ferrotex-citations".into()));
    }

    // 2. Prepare diagnostics for all OPEN documents
    // We iterate over documents map to ensure we have text for line index calculation
    let mut pub_list = Vec::new();

    for entry in documents.iter() {
        let uri = entry.key();
        let text = entry.value();
        let line_index = LineIndex::new(text);

        // Start with cached syntax diagnostics
        let mut diags = syntax_diagnostics
            .get(uri)
            .map(|v| v.clone())
            .unwrap_or_default();

        // Append project diagnostics
        if let Some(proj_list) = project_diags.get(uri) {
            for (range, msg, source) in proj_list {
                let start = line_index.line_col(range.start());
                let end = line_index.line_col(range.end());

                diags.push(Diagnostic {
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
                    source: Some(source.clone()),
                    message: msg.clone(),
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }

        pub_list.push((uri.clone(), diags));
    }

    // 3. Publish
    for (uri, diags) in pub_list {
        client.publish_diagnostics(uri, diags, None).await;
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

/// Watches the workspace for changes and updates the index.
///
/// This function:
/// 1. Scans the workspace directory for existing `.tex` and `.bib` files.
/// 2. Sets up a file watcher to react to `Create`, `Modify`, and `Remove` events.
/// 3. Parses log files to provide real-time feedback (if a log file is tracked).
async fn watch_workspace(
    client: Client,
    root_uri: Arc<Mutex<Option<Url>>>,
    workspace: Arc<Workspace>,
    documents: Arc<DashMap<Url, String>>,
    syntax_diagnostics: Arc<DashMap<Url, Vec<Diagnostic>>>,
) -> anyhow::Result<()> {
    // Get the workspace root path
    let path = {
        let root = root_uri.lock().unwrap();
        if let Some(uri) = root.as_ref() {
            if let Ok(path) = uri.to_file_path() {
                path
            } else {
                std::env::current_dir()?
            }
        } else {
            std::env::current_dir()?
        }
    };

    client
        .log_message(
            MessageType::INFO,
            format!("Scanning and watching workspace in: {:?}", path),
        )
        .await;

    // 1. Setup Watcher (before scan to capture events during scan)
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            let _ = tx.blocking_send(event);
        }
    })?;

    watcher.watch(&path, RecursiveMode::Recursive)?;

    // 2. Initial Scan
    let scan_start = std::time::Instant::now();
    let mut scan_count = 0;
    let mut stack = vec![path.clone()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip hidden directories like .git
                    if !path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.starts_with('.'))
                        .unwrap_or(false)
                    {
                        stack.push(path);
                    }
                } else if let Some(ext) = path.extension() {
                    if ext == "tex"
                        && let Ok(uri) = Url::from_file_path(&path)
                    {
                        // Only index if not already open (though at startup, usually nothing is open yet)
                        if !documents.contains_key(&uri)
                            && let Ok(text) = std::fs::read_to_string(&path)
                        {
                            workspace.update(&uri, &text);
                            scan_count += 1;
                        }
                    } else if ext == "bib"
                        && let Ok(uri) = Url::from_file_path(&path)
                        && !documents.contains_key(&uri)
                        && let Ok(text) = std::fs::read_to_string(&path)
                    {
                        workspace.update_bib(&uri, &text);
                        scan_count += 1;
                    }
                }
            }
        }
    }
    client
        .log_message(
            MessageType::INFO,
            format!("Scanned {} files in {:?}", scan_count, scan_start.elapsed()),
        )
        .await;

    // Publish initial diagnostics after scan
    publish_diagnostics_logic(&client, &workspace, &documents, &syntax_diagnostics).await;

    let mut parser = LogParser::new();
    let mut file_offset: u64 = 0;
    let mut tracked_log: Option<std::path::PathBuf> = None;

    while let Some(event) = rx.recv().await {
        // Handle events
        for path in event.paths {
            if let Some(ext) = path.extension() {
                if ext == "tex" {
                    // ... existing tex logic ...
                    if let Ok(uri) = Url::from_file_path(&path) {
                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                // If file is open in editor, ignore disk events (editor handles it)
                                if !documents.contains_key(&uri)
                                    && let Ok(text) = tokio::fs::read_to_string(&path).await
                                {
                                    workspace.update(&uri, &text);
                                    // Re-validate workspace
                                    publish_diagnostics_logic(
                                        &client,
                                        &workspace,
                                        &documents,
                                        &syntax_diagnostics,
                                    )
                                    .await;
                                }
                            }
                            EventKind::Remove(_) => {
                                // Remove from workspace index
                                workspace.remove(&uri);
                                // Re-validate workspace
                                publish_diagnostics_logic(
                                    &client,
                                    &workspace,
                                    &documents,
                                    &syntax_diagnostics,
                                )
                                .await;
                            }
                            _ => {}
                        }
                    }
                } else if ext == "bib" {
                    if let Ok(uri) = Url::from_file_path(&path) {
                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                if !documents.contains_key(&uri)
                                    && let Ok(text) = tokio::fs::read_to_string(&path).await
                                {
                                    workspace.update_bib(&uri, &text);
                                    publish_diagnostics_logic(
                                        &client,
                                        &workspace,
                                        &documents,
                                        &syntax_diagnostics,
                                    )
                                    .await;
                                }
                            }
                            EventKind::Remove(_) => {
                                workspace.remove(&uri);
                                publish_diagnostics_logic(
                                    &client,
                                    &workspace,
                                    &documents,
                                    &syntax_diagnostics,
                                )
                                .await;
                            }
                            _ => {}
                        }
                    }
                } else if ext == "log" {
                    // Handle .log change (existing logic)
                    if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                        // ... existing log logic ...
                        if tracked_log.is_none() {
                            tracked_log = Some(path.clone());
                            client
                                .log_message(MessageType::INFO, format!("Tracking log: {:?}", path))
                                .await;
                        }

                        if tracked_log.as_ref() == Some(&path) {
                            // Read and parse
                            if let Ok(mut file) = tokio::fs::File::open(&path).await
                                && let Ok(metadata) = file.metadata().await
                            {
                                let current_len = metadata.len();

                                if current_len > file_offset {
                                    use tokio::io::{AsyncReadExt, AsyncSeekExt};
                                    if (file.seek(std::io::SeekFrom::Start(file_offset)).await)
                                        .is_ok()
                                    {
                                        let mut buffer = String::new();
                                        if (file.read_to_string(&mut buffer).await).is_ok() {
                                            let events = parser.update(&buffer);
                                            file_offset = current_len;

                                            let mut diagnostics = Vec::new();

                                            for event in events {
                                                match event.payload {
                                                    EventPayload::Warning { message } => {
                                                        diagnostics.push(Diagnostic {
                                                            range: Range::default(),
                                                            severity: Some(
                                                                DiagnosticSeverity::WARNING,
                                                            ),
                                                            message,
                                                            source: Some("ferrotexd".to_string()),
                                                            ..Default::default()
                                                        });
                                                    }
                                                    EventPayload::ErrorStart { message } => {
                                                        diagnostics.push(Diagnostic {
                                                            range: Range::default(),
                                                            severity: Some(
                                                                DiagnosticSeverity::ERROR,
                                                            ),
                                                            message,
                                                            source: Some("ferrotexd".to_string()),
                                                            ..Default::default()
                                                        });
                                                    }
                                                    _ => {}
                                                }
                                            }

                                            if !diagnostics.is_empty()
                                                && let Ok(uri) = Url::from_file_path(&path)
                                            {
                                                client
                                                    .publish_diagnostics(uri, diagnostics, None)
                                                    .await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Scans the first 5 lines of the document for a magic comment like `%!TEX root = ...`.
///
/// Returns the path relative to the current file if found.
#[allow(dead_code)]
fn detect_magic_root(text: &str) -> Option<std::path::PathBuf> {
    for line in text.lines().take(5) {
        let line = line.trim_start();
        if line.starts_with('%') {
            // Strip %
            let content = line[1..].trim_start();
            // Check for !TEX root or ! TeX root
            if content.starts_with('!') {
                let content = content[1..].trim_start();
                if content.to_ascii_lowercase().starts_with("tex root") {
                    // Extract value after =
                    if let Some(eq_idx) = content.find('=') {
                        let path_str = content[eq_idx + 1..].trim();
                        if !path_str.is_empty() {
                            return Some(std::path::PathBuf::from(path_str));
                        }
                    }
                }
            }
        }
    }
    None
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Arc::new(DashMap::new()),
        workspace: Arc::new(Workspace::new()),
        root_uri: Arc::new(Mutex::new(None)),
        syntax_diagnostics: Arc::new(DashMap::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_root_detection() {
        let cases = vec![
            ("%!TEX root = ../main.tex", Some("../main.tex")),
            ("% !TeX root=  main.tex", Some("main.tex")),
            ("%!TEX root=subdir/main.tex", Some("subdir/main.tex")),
            ("% Regular comment", None),
            ("No comment at all", None),
        ];

        for (input, expected) in cases {
            assert_eq!(
                detect_magic_root(input).map(|p| p.to_string_lossy().to_string()),
                expected.map(|s| s.to_string())
            );
        }
    }

    #[test]
    fn test_magic_root_limit() {
        // Line 1-5 (first 5 lines) are safe. Line 6 is ignored.
        // lines() iterator: 1, 2, 3, 4, 5. So text with 5 leading newlines means the magic comment is on 6th line.
        let text = "\n\n\n\n\n%!TEX root = hidden.tex";
        assert_eq!(detect_magic_root(text), None);

        let valid_text = "\n\n\n\n%!TEX root = visible.tex"; // On 5th line
        assert_eq!(
            detect_magic_root(valid_text).map(|p| p.to_string_lossy().to_string()),
            Some("visible.tex".to_string())
        );
    }
}

/// Analyzes the build log to identify a missing LaTeX package.
///
/// Looks for the standard `! LaTeX Error: File 'foo.sty' not found.` pattern.
fn detect_missing_package(log: &str) -> Option<String> {
    for line in log.lines() {
        if let Some(idx) = line.find("! LaTeX Error: File '") {
            let rest = &line[idx + 21..];
            if let Some(end_idx) = rest.find("'") {
                let filename = &rest[..end_idx];
                if filename.ends_with(".sty") {
                    return Some(filename.trim_end_matches(".sty").to_string());
                }
            }
        }
    }
    None
}

/// Installs a package using `tlmgr`.
///
/// Returns `true` if installation succeeded.
async fn install_package(client: &Client, package: &str) -> bool {
    client.log_message(MessageType::INFO, format!("Attempting to install package '{}'...", package)).await;
    
    // Use tokio Command if available, but for simplicity/mvp std might suffice if short lived.
    // But async is better. backend implies tokio runtime.
    let output = match tokio::process::Command::new("tlmgr")
        .arg("install")
        .arg(package)
        .output()
        .await 
    {
        Ok(o) => o,
        Err(e) => {
             client.log_message(MessageType::ERROR, format!("Failed to execute tlmgr: {}", e)).await;
             return false;
        }
    };

    if output.status.success() {
         client.log_message(MessageType::INFO, format!("Successfully installed '{}'.", package)).await;
         true
    } else {
         let stderr = String::from_utf8_lossy(&output.stderr);
         client.log_message(MessageType::ERROR, format!("tlmgr failed: {}", stderr)).await;
         false
    }
}
