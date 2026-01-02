mod build;
mod completer;
mod diagnostics;
mod fmt;
mod hover;
mod workspace;

mod synctex;

use build::{BuildEngine, BuildRequest, BuildStatus, latexmk::LatexmkAdapter};
use dashmap::DashMap;

use ferrotex_core::package_manager;

use ferrotex_package::{PackageIndex, scanner::PackageScanner};
use ferrotex_log::parser::LogParser;
use ferrotex_syntax::{SyntaxKind, SyntaxNode};
use line_index::LineIndex;
use notify::{EventKind, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
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
    package_manager: Arc<Mutex<package_manager::PackageManager>>,
    package_index: Arc<Mutex<Option<PackageIndex>>>, // Add index field
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        {
            let mut root = self.root_uri.lock().unwrap();
            *root = params.root_uri.clone();
        }
        
        // Detect package manager asynchronously on initialization
        let detected_pm = package_manager::PackageManager::new();
        {
            let mut pm = self.package_manager.lock().unwrap();
            *pm = detected_pm;
        }

        // Start background package scan (with caching and progress)
        let package_index_clone = self.package_index.clone();
        let client_clone = self.client.clone();
        tokio::spawn(async move {
            // Try loading from cache first
            if let Some(cached) = PackageIndex::load_from_cache() {
                let count = cached.packages.len();
                {
                    let mut guard = package_index_clone.lock().unwrap();
                    *guard = Some(cached);
                }
                log::info!("Using cached package index ({} packages).", count);
                return;
            }

            // Cache miss: perform full scan with progress
            let token = tower_lsp::lsp_types::NumberOrString::String("ferrotex-package-scan".to_string());
            
            // Begin progress
            client_clone.send_notification::<tower_lsp::lsp_types::notification::Progress>(
                tower_lsp::lsp_types::ProgressParams {
                    token: token.clone(),
                    value: tower_lsp::lsp_types::ProgressParamsValue::WorkDone(
                        tower_lsp::lsp_types::WorkDoneProgress::Begin(
                            tower_lsp::lsp_types::WorkDoneProgressBegin {
                                title: "Indexing LaTeX Packages".to_string(),
                                cancellable: Some(false),
                                message: Some("Scanning TeX distribution...".to_string()),
                                percentage: Some(0),
                            }
                        )
                    ),
                }
            ).await;

            log::info!("Cache miss. Starting background package scan...");
            let index = tokio::task::spawn_blocking(|| {
                let scanner = PackageScanner::new();
                scanner.scan()
            }).await.unwrap_or_default();
            
            let count = index.packages.len();
            
            // Save to cache for next time
            if let Err(e) = index.save_to_cache() {
                log::warn!("Failed to save package cache: {}", e);
            }
            
            {
                let mut guard = package_index_clone.lock().unwrap();
                *guard = Some(index);
            }
            
            // End progress
            client_clone.send_notification::<tower_lsp::lsp_types::notification::Progress>(
                tower_lsp::lsp_types::ProgressParams {
                    token,
                    value: tower_lsp::lsp_types::ProgressParamsValue::WorkDone(
                        tower_lsp::lsp_types::WorkDoneProgress::End(
                            tower_lsp::lsp_types::WorkDoneProgressEnd {
                                message: Some(format!("Indexed {} packages", count)),
                            }
                        )
                    ),
                }
            ).await;
            
            log::info!("Package scan complete. Indexed {} packages.", count);
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
                let mut items: Vec<CompletionItem> = ENVIRONMENTS
                    .iter()
                    .map(|&env| CompletionItem {
                        label: env.to_string(),
                        kind: Some(CompletionItemKind::SNIPPET),
                        detail: Some("Environment".to_string()),
                        ..Default::default()
                    })
                    .collect();

                // Dynamic Package Completions (CS-3)
                let packages = self.workspace.get_packages(&uri);
                let package_index = self.package_index.lock().unwrap();
                let (_, dyn_envs) = completer::get_package_completions(&packages, package_index.as_ref());
                items.extend(dyn_envs);

                Ok(Some(CompletionResponse::Array(items)))
            }
            CompletionKind::Command => {
                let mut items: Vec<CompletionItem> = COMMANDS
                    .iter()
                    .map(|&cmd| CompletionItem {
                        label: format!("\\{}", cmd),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some("Command".to_string()),
                        ..Default::default()
                    })
                    .collect();

                // Dynamic Package Completions (CS-3)
                let packages = self.workspace.get_packages(&uri);
                let package_index = self.package_index.lock().unwrap();
                let (dyn_cmds, _) = completer::get_package_completions(&packages, package_index.as_ref());
                items.extend(dyn_cmds);

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

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<Vec<CodeActionOrCommand>>> {
        let uri = params.text_document.uri;
        let mut actions = Vec::new();

        for diagnostic in params.context.diagnostics {
            if diagnostic.source.as_deref() == Some("ferrotex-deprecated") {
                // Determine the command from the text to provide the correct fix
                let cmd_name = if let Some(text) = self.get_text(&uri) {
                let line_index = LineIndex::new(&text);
                let start = line_index.offset(line_index::LineCol {
                    line: diagnostic.range.start.line,
                    col: diagnostic.range.start.character,
                });
                let end = line_index.offset(line_index::LineCol {
                    line: diagnostic.range.end.line,
                    col: diagnostic.range.end.character,
                });

                if let (Some(start), Some(end)) = (start, end) {
                    text[usize::from(start)..usize::from(end)].to_string()
                } else {
                    continue;
                }
            } else {
                continue;
            };

            // Handle Display Math ($$...$$)
            if diagnostic.message.to_lowercase().contains("display math") {
                // Extract the text content to replace $$...$$ with \[...\]
                if let Some(text) = self.get_text(&uri) {
                    let line_index = LineIndex::new(&text);
                    let start = line_index.offset(line_index::LineCol {
                        line: diagnostic.range.start.line,
                        col: diagnostic.range.start.character,
                    });
                    let end = line_index.offset(line_index::LineCol {
                        line: diagnostic.range.end.line,
                        col: diagnostic.range.end.character,
                    });

                    if let (Some(start), Some(end)) = (start, end) {
                        let block_text = &text[usize::from(start)..usize::from(end)];
                        // Extract content between $$ and $$
                        // Format: $$<content>$$
                        if block_text.len() >= 4 && block_text.starts_with("$$") && block_text.ends_with("$$") {
                            let content = &block_text[2..block_text.len()-2];
                            let new_text = format!("\\[{}\\]", content);
                            
                            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                title: "Convert to \\[...\\] (LaTeX2e display math)".to_string(),
                                kind: Some(CodeActionKind::QUICKFIX),
                                diagnostics: Some(vec![diagnostic.clone()]),
                                edit: Some(WorkspaceEdit {
                                    changes: Some(std::collections::HashMap::from([(
                                        uri.clone(),
                                        vec![TextEdit {
                                            range: diagnostic.range,
                                            new_text,
                                        }],
                                    )])),
                                    ..Default::default()
                                }),
                                is_preferred: Some(true),
                                ..Default::default()
                            }));
                        }
                    }
                }
                continue;
            }

            // Handle Packages
            if diagnostic.message.to_lowercase().contains("package") {
                let replacement = match cmd_name.as_str() {
                    "times" => "mathptmx",
                    "psfig" | "epsfig" => "graphicx",
                    "a4wide" => "geometry",
                    _ => continue,
                };

                let title = format!("Replace package '{}' with '{}'", cmd_name, replacement);
                let edit = WorkspaceEdit {
                    changes: Some(std::collections::HashMap::from([(
                        uri.clone(),
                        vec![TextEdit {
                            range: diagnostic.range,
                            new_text: replacement.to_string(),
                        }],
                    )])),
                    ..Default::default()
                };

                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title,
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic.clone()]),
                    edit: Some(edit),
                    is_preferred: Some(true),
                    ..Default::default()
                }));
                continue;
            }

            // Handle Font Commands (context-aware)
            // Check if this is a font command with context
            let (base_cmd, in_group) = if cmd_name.contains(":group") {
                (cmd_name.split(':').next().unwrap_or(""), true)
            } else {
                (cmd_name.as_str(), false)
            };
            
            // Define mappings for deprecated font commands
            let font_mappings: std::collections::HashMap<&str, (&str, &str)> = [
                ("\\bf", ("\\bfseries", "\\textbf")),
                ("\\it", ("\\itshape", "\\textit")),
                ("\\sc", ("\\scshape", "\\textsc")),
                ("\\rm", ("\\rmfamily", "\\textrm")),
                ("\\sf", ("\\sffamily", "\\textsf")),
                ("\\tt", ("\\ttfamily", "\\texttt")),
                ("\\sl", ("\\slshape", "\\textsl")),
            ].iter().cloned().collect();
            
            if let Some((declaration_cmd, semantic_cmd)) = font_mappings.get(base_cmd) {
                if in_group {
                    // For {\bf text}, offer two options:
                    // 1. Replace \bf with \bfseries (declaration style)
                    // 2. Convert to \textbf{text} (semantic style - preferred)
                    
                    if let Some(text) = self.get_text(&uri) {
                        let line_index = LineIndex::new(&text);
                        let start = line_index.offset(line_index::LineCol {
                            line: diagnostic.range.start.line,
                            col: diagnostic.range.start.character,
                        });
                        let end = line_index.offset(line_index::LineCol {
                            line: diagnostic.range.end.line,
                            col: diagnostic.range.end.character,
                        });

                        if let (Some(start), Some(end)) = (start, end) {
                            let group_text = &text[usize::from(start)..usize::from(end)];
                            
                            // Expected format: {\cmd content}
                            if group_text.starts_with('{') && group_text.ends_with('}') {
                                let inner = &group_text[1..group_text.len()-1].trim_start();
                                
                                // Option 1: Semantic (preferred)
                                if let Some(content) = inner.strip_prefix(base_cmd).map(|s| s.trim()) {
                                    let semantic_replacement = format!("{}{{{}}}",  semantic_cmd, content);
                                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                        title: format!("Convert to {} (semantic, recommended)", semantic_cmd),
                                        kind: Some(CodeActionKind::QUICKFIX),
                                        diagnostics: Some(vec![diagnostic.clone()]),
                                        edit: Some(WorkspaceEdit {
                                            changes: Some(std::collections::HashMap::from([(
                                                uri.clone(),
                                                vec![TextEdit {
                                                    range: diagnostic.range,
                                                    new_text: semantic_replacement,
                                                }],
                                            )])),
                                            ..Default::default()
                                        }),
                                        is_preferred: Some(true),
                                        ..Default::default()
                                    }));
                                    
                                    // Option 2: Declaration (alternative)
                                    let declaration_replacement = group_text.replacen(base_cmd, declaration_cmd, 1);
                                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                        title: format!("Replace {} with {}", base_cmd, declaration_cmd),
                                        kind: Some(CodeActionKind::QUICKFIX),
                                        diagnostics: Some(vec![diagnostic.clone()]),
                                        edit: Some(WorkspaceEdit {
                                            changes: Some(std::collections::HashMap::from([(
                                                uri.clone(),
                                                vec![TextEdit {
                                                    range: diagnostic.range,
                                                    new_text: declaration_replacement,
                                                }],
                                            )])),
                                            ..Default::default()
                                        }),
                                        is_preferred: Some(false),
                                        ..Default::default()
                                    }));
                                }
                            }
                        }
                    }
                } else {
                    // Standalone command, just replace with declaration
                    let replacement = *declaration_cmd;
                    let title = format!("Replace '{}' with '{}'", base_cmd, replacement);
                    let edit = WorkspaceEdit {
                        changes: Some(std::collections::HashMap::from([(
                            uri.clone(),
                            vec![TextEdit {
                                range: diagnostic.range,
                                new_text: replacement.to_string(),
                            }],
                        )])),
                        ..Default::default()
                    };

                    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                        title,
                        kind: Some(CodeActionKind::QUICKFIX),
                        diagnostics: Some(vec![diagnostic.clone()]),
                        edit: Some(edit),
                        is_preferred: Some(true),
                        ..Default::default()
                    }));
                }
                continue;
            }
        }
    }

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

            return Ok(hover::find_hover(&root, offset, &self.workspace));
        }

        Ok(None)
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        // Handle package installation command
        if params.command == "ferrotex.installPackage" {
            if params.arguments.is_empty() {
                self.client.log_message(MessageType::ERROR, "Install package command missing package name").await;
                return Ok(None);
            }
            
            let package_name = params.arguments[0].as_str().unwrap_or_default();
            self.client.log_message(MessageType::INFO, format!("Installing package '{}'...", package_name)).await;
            
            let pm = self.package_manager.lock().unwrap().clone();
            match pm.install(package_name) {
                Ok(status) => {
                    match status.state {
                        package_manager::InstallState::Complete => {
                            self.client.show_message(MessageType::INFO, format!("Successfully installed package '{}'", package_name)).await;
                            return Ok(Some(serde_json::json!({"success": true})));
                        }
                        package_manager::InstallState::Failed => {
                            let err = status.message.unwrap_or_else(|| "Unknown error".to_string());
                            self.client.show_message(MessageType::ERROR, format!("Failed to install '{}': {}", package_name, err)).await;
                            return Ok(Some(serde_json::json!({"success": false, "error": err})));
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    self.client.show_message(MessageType::ERROR, format!("Installation error: {}", e)).await;
                    return Ok(Some(serde_json::json!({"success": false, "error": e.to_string()})));
                }
            }
        }
        
        if params.command == "ferrotex.synctex_forward" {
             let args = params.arguments;
             if args.len() < 4 { return Ok(None); }
             let tex_uri_str = args[0].as_str().unwrap_or_default();
             let line = args[1].as_u64().unwrap_or(0) as u32;
             let col = args[2].as_u64().unwrap_or(0) as u32;
             let pdf_uri_str = args[3].as_str().unwrap_or_default();
             
             #[allow(clippy::collapsible_if)]
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
             
             #[allow(clippy::collapsible_if)]
             if let Ok(pdf_url) = Url::parse(pdf_uri_str) {
                 if let Ok(pdf_path) = pdf_url.to_file_path() {
                     if let Some(res) = synctex::inverse_search(&pdf_path, page, x, y) {
                         return Ok(Some(serde_json::to_value(res).unwrap()));
                     }
                 }
             }
             return Ok(None);
        }

        if params.command == "ferrotex.internal.build" {
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
            // --- Magic Comment Detection (UX-2) --
            // Refactored to use Workspace Index (v0.15.0)
            let mut build_uri = uri.clone();
            #[allow(clippy::collapsible_if)]
            if let Some(magic_path) = self.workspace.get_explicit_root(&uri) {
                if let Ok(file_path) = uri.to_file_path() {
                     if let Some(parent) = file_path.parent() {
                        let new_path = parent.join(&magic_path);
                        if let Ok(new_uri) = Url::from_file_path(&new_path) {
                            self.client
                                .log_message(
                                    MessageType::INFO,
                                    format!(
                                        "Magic Root detected (Index): Redirecting build to {}",
                                        new_uri
                                    ),
                                )
                                .await;
                            build_uri = new_uri;
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

            // Setup Log Streaming (BO-2)
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
            let client_clone = self.client.clone();
            
            // Spawn a task to forward logs to the client
            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    #[derive(serde::Serialize, serde::Deserialize)]
                    struct LogParams {
                        message: String,
                    }
                    enum LogNotification {}
                    impl tower_lsp::lsp_types::notification::Notification for LogNotification {
                        type Params = LogParams;
                        const METHOD: &'static str = "$/ferrotex/log";
                    }

                    client_clone
                        .send_notification::<LogNotification>(LogParams { message: msg })
                        .await;
                }
            });

            // Callback to feed the channel
            let log_callback = Box::new(move |msg: String| {
                let _ = tx.send(msg);
            });

            // Engine Selection Logic (Zero-Config)
            // 1. Check for valid 'latexmk'. Logic: It's the standard.
            // 2. Fallback to 'tectonic' if latexmk missing (likely our downloaded one).
            
            // Note: In production we should maybe cache this decision or respect config.
            // For now, heuristic check is acceptable.
            
            // Create an enum-based engine to work around async trait object limitations
            enum BuildEngineImpl {
                Latexmk(LatexmkAdapter),
                #[cfg(feature = "use-tectonic")]
                Tectonic(build::tectonic::TectonicAdapter),
            }
            
            impl BuildEngineImpl {
                fn name(&self) -> &str {
                    match self {
                        Self::Latexmk(e) => e.name(),
                        #[cfg(feature = "use-tectonic")]
                        Self::Tectonic(e) => e.name(),
                    }
                }
                
                async fn build(
                    &self,
                    request: &BuildRequest,
                    log_callback: Option<Box<dyn Fn(String) + Send + Sync>>,
                ) -> anyhow::Result<BuildStatus> {
                    match self {
                        Self::Latexmk(e) => e.build(request, log_callback).await,
                        #[cfg(feature = "use-tectonic")]
                        Self::Tectonic(e) => e.build(request, log_callback).await,
                    }
                }
            }
            
            let engine = if std::process::Command::new("latexmk").arg("-v").stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().map(|s| s.success()).unwrap_or(false) {
                BuildEngineImpl::Latexmk(LatexmkAdapter)
            } else {
                #[cfg(feature = "use-tectonic")]
                {
                    BuildEngineImpl::Tectonic(build::tectonic::TectonicAdapter)
                }
                #[cfg(not(feature = "use-tectonic"))]
                {
                    // Fallback to latexmk if tectonic is disabled, effective "default"
                    BuildEngineImpl::Latexmk(LatexmkAdapter)
                }
            };
            
            self.client
                .log_message(MessageType::INFO, format!("Using Build Engine: {}", engine.name()))
                .await;

            let result = engine.build(&req, Some(log_callback)).await;

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
                        
                        // Parse Diagnostics from Tectonic Output (UX-ZeroConfig)
                        if engine.name() == "tectonic" {
                             let diagnostics = parse_tectonic_diagnostics(&format!("{}\n{}", log.stdout, log.stderr), &uri);
                             for (doc_uri, diags) in diagnostics {
                                 // We need to merge with existing diagnostics? 
                                 // For now, just publish. The user will see them.
                                 self.client.publish_diagnostics(doc_uri, diags, None).await;
                             }
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
                            
                            #[allow(clippy::collapsible_if)]
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
        if let rowan::WalkEvent::Enter(rowan::NodeOrToken::Token(token)) = event {
            if let Some((type_idx, modifier_bitset)) = classify_token(&token)
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
    if kind == SyntaxKind::Text {
        if let Some(parent) = token.parent() {
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
            }
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
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !name.starts_with('.') {
                            stack.push(path);
                        }
                    }
                } else if let Some(ext) = path.extension() {
                    if ext == "tex" {
                        if let Ok(rel) = path.strip_prefix(root_path) {
                            if let Some(s) = rel.to_str() {
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

    // Deprecated Commands (Lint)
    let deprecated_warnings = workspace.validate_deprecated();
    for (uri, range, msg) in deprecated_warnings {
        project_diags
            .entry(uri)
            .or_default()
            .push((range, msg, "ferrotex-deprecated".into()));
    }

    // 2. Prepare diagnostics for all OPEN documents
    // We iterate over documents map to ensure we have text for line index calculation
    let mut pub_list = Vec::new();
    let mut published_uris = HashSet::new();

    for entry in documents.iter() {
        let uri = entry.key();
        let text = entry.value();
        let line_index = LineIndex::new(text);

        // Start with cached syntax diagnostics
        let mut diags = syntax_diagnostics
            .get(uri)
            .map(|v| v.clone())
            .unwrap_or_default();

        // 1.5. Run Math Semantics Analysis (PhD Implementation)
        let parse = ferrotex_syntax::parse(text);
        let root = SyntaxNode::new_root(parse.green_node());
        let math_diags = crate::diagnostics::math::check_math(&root, &line_index);
        diags.extend(math_diags);

        // Append project diagnostics
        if let Some(proj_list) = project_diags.get(uri) {
            for (range, msg, source) in proj_list {
                let start = line_index.line_col(range.start());
                let end = line_index.line_col(range.end());

                let severity = if source == "ferrotex-deprecated" {
                    DiagnosticSeverity::WARNING
                } else {
                    DiagnosticSeverity::ERROR
                };

                let mut message = msg.clone();
                if source == "ferrotex-deprecated" {
                     if msg == "displaymath" {
                         message = "Use '\\[ ... \\]' instead of '$$ ... $$' for display math (LaTeX2e standard).".to_string();
                     } else if msg.starts_with("package:") {
                         let pkg = msg.strip_prefix("package:").unwrap_or("unknown");
                         message = format!("Package '{}' is obsolete/deprecated.", pkg);
                     } else if msg.contains(":group") {
                         // Font command in group context
                         let cmd = msg.split(':').next().unwrap_or("");
                         message = format!("Font command '{}' is deprecated. Use LaTeX2e equivalents.", cmd);
                     } else {
                         message = format!("Command '{}' is deprecated. Use modern LaTeX2e equivalent.", msg);
                     }
                }

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
                    severity: Some(severity),
                    code: Some(NumberOrString::String("deprecated".to_string())),
                    code_description: None,
                    source: Some(source.clone()),
                    message,
                    related_information: None,
                    tags: if source == "ferrotex-deprecated" { Some(vec![DiagnosticTag::DEPRECATED]) } else { None },
                    data: None,
                });
            }
        }

        published_uris.insert(uri.clone());
        pub_list.push((uri.clone(), diags));
    }

    // 3. Publish diagnostics for indexed workspace files that are NOT open
    // This enables continuous monitoring even for files not currently in the editor
    for (uri, proj_list) in project_diags.iter() {
        // Skip if already published for open document
        if published_uris.contains(uri) {
            continue;
        }

        // Try to read file content from disk for line index calculation
        if let Ok(file_path) = uri.to_file_path() {
            if let Ok(text) = tokio::fs::read_to_string(&file_path).await {
                let line_index = LineIndex::new(&text);
                let mut diags = Vec::new();

                for (range, msg, source) in proj_list {
                    let start = line_index.line_col(range.start());
                    let end = line_index.line_col(range.end());

                    let severity = if source == "ferrotex-deprecated" {
                        DiagnosticSeverity::WARNING
                    } else {
                        DiagnosticSeverity::ERROR
                    };

                    let mut message = msg.clone();
                    if source == "ferrotex-deprecated" {
                         if msg == "displaymath" {
                             message = "Use '\\[ ... \\]' instead of '$$ ... $$' for display math (LaTeX2e standard).".to_string();
                         } else if msg.starts_with("package:") {
                             let pkg = msg.strip_prefix("package:").unwrap_or("unknown");
                             message = format!("Package '{}' is obsolete/deprecated.", pkg);
                         } else {
                             message = format!("Command '{}' is deprecated. Use modern LaTeX2e equivalent.", msg);
                         }
                    }

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
                        severity: Some(severity),
                        code: Some(NumberOrString::String("deprecated".to_string())),
                        code_description: None,
                        source: Some(source.clone()),
                        message,
                        related_information: None,
                        tags: if source == "ferrotex-deprecated" { Some(vec![DiagnosticTag::DEPRECATED]) } else { None },
                        data: None,
                    });
                }

                pub_list.push((uri.clone(), diags));
            }
        }
    }

    // 4. Publish
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
                    if ext == "tex" {
                        if let Ok(uri) = Url::from_file_path(&path) {
                        // Only index if not already open (though at startup, usually nothing is open yet)
                        if !documents.contains_key(&uri) {
                            if let Ok(text) = std::fs::read_to_string(&path) {
                                workspace.update(&uri, &text);
                                scan_count += 1;
                            }
                        }
                    }
                } else if ext == "bib" {
                        if let Ok(uri) = Url::from_file_path(&path) {
                            if !documents.contains_key(&uri) {
                                if let Ok(text) = std::fs::read_to_string(&path) {
                                    workspace.update_bib(&uri, &text);
                                    scan_count += 1;
                                }
                            }
                        }
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
                                if !documents.contains_key(&uri) {
                                    if let Ok(text) = tokio::fs::read_to_string(&path).await {
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
                                if !documents.contains_key(&uri) {
                                    if let Ok(text) = tokio::fs::read_to_string(&path).await {
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
                             // Read and parse log incrementally
                             let mut file_handle = None;
                             if let Ok(f) = tokio::fs::File::open(&path).await {
                                 file_handle = Some(f);
                             }

                             if let Some(mut file) = file_handle {
                                 // Check for truncation
                                 if let Ok(metadata) = file.metadata().await {
                                     if metadata.len() < file_offset {
                                         file_offset = 0;
                                         parser = LogParser::new();
                                     }
                                 }

                                 if file.seek(std::io::SeekFrom::Start(file_offset)).await.is_ok() {
                                     let mut buffer = String::new();
                                     if file.read_to_string(&mut buffer).await.is_ok()
                                         && !buffer.is_empty() {
                                             file_offset += buffer.len() as u64;
                                             let events = parser.update(&buffer);

                                             // Map log path to source file URI (Heuristic)
                                             let tex_path = path.with_extension("tex");
                                             let mut target_uri = Url::from_file_path(&tex_path).ok();

                                             if !tex_path.exists() {
                                                 if let Some(parent) = path.parent() {
                                                     if let Some(grandparent) = parent.parent() {
                                                         if let Some(file_name) = path.file_name() {
                                                             let parent_tex = grandparent.join(file_name).with_extension("tex");
                                                             if parent_tex.exists() {
                                                                 target_uri = Url::from_file_path(&parent_tex).ok();
                                                             }
                                                         }
                                                     }
                                                 }
                                             }
                                             
                                             // Fallback
                                             if target_uri.is_none() {
                                                 target_uri = Url::from_file_path(&path).ok();
                                             }

                                             let mut diagnostics_map: std::collections::HashMap<Url, Vec<Diagnostic>> = std::collections::HashMap::new();

                                             for event in events {
                                                 let (severity, raw_message) = match event.payload {
                                                     ferrotex_log::ir::EventPayload::ErrorStart { message } => 
                                                         (DiagnosticSeverity::ERROR, message),
                                                     ferrotex_log::ir::EventPayload::Warning { message } => 
                                                         (DiagnosticSeverity::WARNING, message),
                                                     _ => continue,
                                                 };

                                                 let mut msg = raw_message.clone();
                                                 if let Some(exp) = diagnostics::error_index::explain(&raw_message) {
                                                     msg.push_str(&format!("\n\n💡 {}: {}", exp.summary, exp.description));
                                                 }

                                                 let diag = Diagnostic {
                                                     range: Range::default(),
                                                     severity: Some(severity),
                                                     code: None,
                                                     code_description: None,
                                                     source: Some("ferrotex-log".to_string()),
                                                     message: msg,
                                                     related_information: None,
                                                     tags: None,
                                                     data: None,
                                                 };

                                                 if let Some(uri) = &target_uri {
                                                     diagnostics_map.entry(uri.clone()).or_default().push(diag);
                                                 }
                                             }

                                             for (uri, diags) in diagnostics_map {
                                                client.publish_diagnostics(uri, diags, None).await;
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

/// Detects if an error message indicates a missing package file.
/// Returns the filename (e.g., "tikz.sty") if found.
fn _detect_missing_package_file(message: &str) -> Option<String> {
    // Pattern: "File 'xxx.sty' not found"
    if let Some(start_idx) = message.find("File '") {
        let rest = &message[start_idx + 6..];
        if let Some(end_idx) = rest.find("' not found") {
            let filename = rest[..end_idx].trim();
            if filename.ends_with(".sty") {
                return Some(filename.to_string());
            }
        }
    }
    None
}

/// Scans the first 5 lines of the document for a magic comment like `%!TEX root = ...`.
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

/// Parses Tectonic's `error: file:line: message` format into LSP Diagnostics.
fn parse_tectonic_diagnostics(output: &str, base_uri: &Url) -> std::collections::HashMap<Url, Vec<Diagnostic>> {
    let mut map: std::collections::HashMap<Url, Vec<Diagnostic>> = std::collections::HashMap::new();
    
    // Tectonic format: "error: main.tex:15: Missing $ inserted"
    // Also "warning: ..."
    
    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() < 4 { continue; }
        
        let severity_str = parts[0].trim();
        let filename = parts[1].trim();
        let lineno_str = parts[2].trim();
        let message = parts[3].trim();
        
        let severity = match severity_str {
            "error" => DiagnosticSeverity::ERROR,
            "warning" => DiagnosticSeverity::WARNING,
             _ => {
                 // Sometimes it starts with "[error] ..." or similar if piped? 
                 // But Tectonic raw output is usually clear.
                 // We'll skip if it doesn't match known prefixes.
                 if line.contains("error:") { DiagnosticSeverity::ERROR } else { continue }
             }
        };
        
        // Resolve URI
        // Assuming filename is relative to the document being built
        // If base_uri is "file:///demo/main.tex", and filename is "main.tex" -> match.
        // If filename is "chapters/intro.tex" -> resolve.
        
        let target_uri = if let Ok(base_path) = base_uri.to_file_path() {
            if let Some(parent) = base_path.parent() {
                 let path = parent.join(filename);
                 Url::from_file_path(path).ok()
            } else {
                None
            }
        } else {
             None
        };
        
        if let Some(uri) = target_uri {
             let line_num: u32 = lineno_str.parse::<u32>().unwrap_or(1).saturating_sub(1); // 0-indexed
             
             // Enhance error messages with helpful suggestions
             let enhanced_message = if message.contains("Undefined control sequence") {
                 // Try to extract the undefined command
                 if let Some(command) = crate::diagnostics::error_index::extract_undefined_command(message) {
                     if let Some(suggestion) = crate::diagnostics::error_index::suggest_package(&command) {
                         format!("{}\n\n{}", message, suggestion)
                     } else {
                         // No known package, provide general help
                         format!("{}\n\n💡 Check: spelling, \\usepackage{{...}}, or define with \\newcommand", message)
                     }
                 } else {
                     message.to_string()
                 }
             } else if let Some(explanation) = crate::diagnostics::error_index::explain(message) {
                 // Use human-readable explanation for other errors
                 format!("{}\n\n💡 {}: {}", message, explanation.summary, explanation.description)
             } else {
                 message.to_string()
             };
             
             let diag = Diagnostic {
                range: Range {
                    start: Position { line: line_num, character: 0 },
                    end: Position { line: line_num, character: 100 }, // Highlight whole line?
                },
                severity: Some(severity),
                code: None,
                code_description: None,
                source: Some("tectonic".to_string()),
                message: enhanced_message,
                related_information: None,
                tags: None,
                data: None, 
             };
             
             map.entry(uri).or_default().push(diag);
        }
    }
    
    map
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
        package_manager: Arc::new(Mutex::new(package_manager::PackageManager::new())),
        package_index: Arc::new(Mutex::new(None)), // Initialize as None
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused
}

