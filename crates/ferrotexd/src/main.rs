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
                document_link_provider: Some(DocumentLinkOptions {
                    resolve_provider: Some(false),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
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
}

impl Backend {
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

    async fn publish_all_diagnostics(&self) {
        publish_diagnostics_logic(
            &self.client,
            &self.workspace,
            &self.documents,
            &self.syntax_diagnostics,
        )
        .await;
    }

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

    // Helper to get text for a URI (open or read from disk)
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

async fn publish_diagnostics_logic(
    client: &Client,
    workspace: &Workspace,
    documents: &DashMap<Url, String>,
    syntax_diagnostics: &DashMap<Url, Vec<Diagnostic>>,
) {
    // 1. Collect all project-level diagnostics
    let cycles = workspace.detect_cycles();
    let label_errors = workspace.validate_labels();

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
                            if let Ok(mut file) = tokio::fs::File::open(&path).await {
                                if let Ok(metadata) = file.metadata().await {
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
                                                                source: Some(
                                                                    "ferrotexd".to_string(),
                                                                ),
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
                                                                source: Some(
                                                                    "ferrotexd".to_string(),
                                                                ),
                                                                ..Default::default()
                                                            });
                                                        }
                                                        _ => {}
                                                    }
                                                }

                                                if !diagnostics.is_empty() {
                                                    if let Ok(uri) = Url::from_file_path(&path) {
                                                        client
                                                            .publish_diagnostics(
                                                                uri,
                                                                diagnostics,
                                                                None,
                                                            )
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
        documents: Arc::new(DashMap::new()),
        workspace: Arc::new(Workspace::new()),
        root_uri: Arc::new(Mutex::new(None)),
        syntax_diagnostics: Arc::new(DashMap::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
