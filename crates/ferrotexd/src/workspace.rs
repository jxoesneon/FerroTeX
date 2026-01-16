//! # Workspace Management
//!
//! Provides the central index for the FerroTeX language server, managing
//! cross-file references, label definitions, and bibliography data.
//!
//! The [`Workspace`] struct acts as a thread-safe, shared repository for
//! all document-related metadata, enabling features like "Go to Definition",
//! "Find References", and global workspace symbols.

use crate::macros::{scan_macros, MacroDef};
use dashmap::DashMap;
use ferrotex_syntax::{parse, SyntaxKind, TextRange};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use tower_lsp::lsp_types::{SymbolKind, Url};

/// The central workspace manager for the LSP server.
///
/// It maintains an in-memory index of all tracked TeX and BibTeX files.
#[derive(Debug, Default)]
pub struct Workspace {
    /// Per-file index containing includes, definitions, citations, etc.
    pub indices: DashMap<Url, FileIndex>,
    /// Bibliography index containing parsed BibTeX entries.
    bib_indices: DashMap<Url, ferrotex_syntax::bibtex::BibFile>,
    /// Explicit root overrides from `%!TEX root` comments.
    explicit_roots: DashMap<Url, String>,
}

/// The index data for a single TeX file.
#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct FileIndex {
    /// List of included files (e.g., `\input{...}`).
    pub includes: Vec<IncludeRef>,
    /// List of label definitions (e.g., `\label{...}`).
    pub definitions: Vec<LabelDef>,
    /// List of label references (e.g., `\ref{...}`).
    pub references: Vec<LabelRef>,
    /// List of citations (e.g., `\cite{...}`).
    pub citations: Vec<CitationRef>,
    /// List of bibliographies (e.g., `\bibliography{...}`).
    pub bibliographies: Vec<BibRef>,
    /// List of sections (e.g., `\section{...}`).
    pub sections: Vec<SectionDef>,
    /// List of used packages (e.g., `\usepackage{...}`).
    pub packages: Vec<String>,
    /// List of environments (e.g., `\begin{...}`).
    pub environments: Vec<EnvDef>,
    /// List of deprecated command usages.
    pub deprecated_usages: Vec<(TextRange, String)>,
    /// List of user-defined macros.
    pub macros: Vec<MacroDef>,
}

/// Represents an environment definition.
#[derive(Debug, Clone)]
pub struct EnvDef {
    /// The environment name.
    pub name: String,
    /// The range of the entire environment block.
    pub range: TextRange,
}

/// Represents an included file reference.
#[derive(Debug, Clone)]
pub struct IncludeRef {
    /// The path to the included file (as written in the source).
    pub path: String,
    /// The range of the path string in the source file.
    pub range: TextRange,
}

/// Represents a section definition.
#[derive(Debug, Clone)]
pub struct SectionDef {
    /// The section title.
    pub name: String,
    /// The range of the section title in the source file.
    pub range: TextRange,
}

/// Represents a label definition.
#[derive(Debug, Clone)]
pub struct LabelDef {
    /// The label name.
    pub name: String,
    /// The range of the label name in the source file.
    pub range: TextRange,
}

/// Represents a reference to a label.
#[derive(Debug, Clone)]
pub struct LabelRef {
    /// The referenced label name.
    pub name: String,
    /// The range of the reference name in the source file.
    pub range: TextRange,
}

/// Represents a citation.
#[derive(Debug, Clone)]
pub struct CitationRef {
    /// The citation key.
    pub key: String,
    /// The range of the citation key in the source file.
    pub range: TextRange,
}

/// Represents a bibliography file reference.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BibRef {
    /// The path to the bibliography file.
    pub path: String,
    /// The range of the path string in the source file.
    pub range: TextRange,
}

impl Workspace {
    /// Creates a new, empty workspace.
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates the index for a given TeX file.
    ///
    /// This method performs a full scan of the document text to extract:
    /// - Include references (`\input`, `\include`)
    /// - Label definitions and references
    /// - Citations and bibliographies
    /// - Section hierarchy
    /// - Used packages
    /// - Environmental scopes
    ///
    /// It also detects "magic comments" like `%!TEX root` to handle multi-file project structures.
    pub fn update(&self, uri: &Url, text: &str) {
        let (
            includes,
            definitions,
            references,
            citations,
            bibliographies,
            sections,
            packages,
            magic_root,
            deprecated_usages,
            environments,
            macros,
        ) = scan_file(text);

        if let Some(root_path) = magic_root {
            self.explicit_roots.insert(uri.clone(), root_path);
        } else {
            self.explicit_roots.remove(uri);
        }

        self.indices.insert(
            uri.clone(),
            FileIndex {
                includes,
                definitions,
                references,
                citations,
                bibliographies,
                sections,
                packages,
                environments,
                deprecated_usages,
                macros,
            },
        );
    }

    /// Updates the index for a given BibTeX file.
    ///
    /// Parses the BibTeX content and extracts entries.
    pub fn update_bib(&self, uri: &Url, text: &str) {
        let bib_file = ferrotex_syntax::bibtex::parse_bibtex(text);
        self.bib_indices.insert(uri.clone(), bib_file);
    }

    /// Removes a file from the workspace index.
    pub fn remove(&self, uri: &Url) {
        self.indices.remove(uri);
        self.bib_indices.remove(uri);
    }

    /// Retrieves the list of included files for a given document URI.
    pub fn get_includes(&self, uri: &Url) -> Vec<IncludeRef> {
        self.indices
            .get(uri)
            .map(|v| v.includes.clone())
            .unwrap_or_default()
    }

    /// Retrieves the explicit root override for a given document URI, if any.
    pub fn get_explicit_root(&self, uri: &Url) -> Option<String> {
        self.explicit_roots.get(uri).map(|v| v.value().clone())
    }

    /// Retrieves the list of used packages for a given document URI.
    ///
    /// If an explicit root is set, it also includes packages from the root.
    pub fn get_packages(&self, uri: &Url) -> Vec<String> {
        let mut packages = HashSet::new();

        // 1. Get packages from current file
        if let Some(idx) = self.indices.get(uri) {
            packages.extend(idx.packages.clone());
        }

        // 2. Get packages from explicit root (if any)
        if let Some(root_path) = self.get_explicit_root(uri) {
            #[allow(clippy::collapsible_if)]
            if let Ok(file_path) = uri.to_file_path() {
                if let Some(parent) = file_path.parent() {
                    let root_buf = parent.join(&root_path);
                    if let Ok(root_uri) = Url::from_file_path(root_buf) {
                        #[allow(clippy::collapsible_if)]
                        if let Some(idx) = self.indices.get(&root_uri) {
                            packages.extend(idx.packages.clone());
                        }
                    }
                }
            }
        }

        packages.into_iter().collect()
    }

    /// Retrieves the list of bibliography references for a given document URI.
    #[allow(dead_code)]
    pub fn get_bibliographies(&self, uri: &Url) -> Vec<BibRef> {
        self.indices
            .get(uri)
            .map(|v| v.bibliographies.clone())
            .unwrap_or_default()
    }

    // --- Index Queries ---

    /// Returns all citation keys defined in all indexed BibTeX files.
    pub fn get_all_citation_keys(&self) -> Vec<String> {
        let referenced_bibs = self.get_referenced_bib_uris();
        let mut keys = HashSet::new();

        if referenced_bibs.is_empty() {
            for entry in self.bib_indices.iter() {
                for bib_entry in &entry.value().entries {
                    keys.insert(bib_entry.key.clone());
                }
            }
        } else {
            for uri in referenced_bibs {
                if let Some(bib_file) = self.bib_indices.get(&uri) {
                    for bib_entry in &bib_file.entries {
                        keys.insert(bib_entry.key.clone());
                    }
                }
            }
        }

        let mut keys: Vec<String> = keys.into_iter().collect();
        keys.sort();
        keys
    }

    pub fn get_referenced_bib_uris(&self) -> Vec<Url> {
        let mut uris = HashSet::new();

        for entry in self.indices.iter() {
            let base_uri = entry.key();
            for bib in &entry.value().bibliographies {
                if let Some(uri) = resolve_bib_uri(base_uri, &bib.path) {
                    uris.insert(uri);
                }
            }
        }

        uris.into_iter().collect()
    }

    /// Returns all label names defined in all indexed TeX files.
    pub fn get_all_labels(&self) -> Vec<String> {
        let mut labels = HashSet::new();
        for entry in self.indices.iter() {
            for def in &entry.value().definitions {
                labels.insert(def.name.clone());
            }
        }
        labels.into_iter().collect()
    }

    /// Checks if a citation key exists in the workspace.
    pub fn has_citation_key(&self, key: &str) -> bool {
        let referenced_bibs = self.get_referenced_bib_uris();

        if referenced_bibs.is_empty() {
            for entry in self.bib_indices.iter() {
                if entry.value().entries.iter().any(|e| e.key == key) {
                    return true;
                }
            }
        } else {
            for uri in referenced_bibs {
                if let Some(bib_file) = self.bib_indices.get(&uri) {
                    if bib_file.entries.iter().any(|e| e.key == key) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Retrieves detailed information about a citation key for hover.
    pub fn get_citation_details(&self, key: &str) -> Option<String> {
        let referenced_bibs = self.get_referenced_bib_uris();

        let find_in_bibs = |uris: &Vec<Url>| -> Option<String> {
            for uri in uris {
                if let Some(bib_file) = self.bib_indices.get(uri) {
                    if let Some(entry) = bib_file.entries.iter().find(|e| e.key == key) {
                        // Found logic
                        let title = entry
                            .fields
                            .get("title")
                            .map(|s| s.as_str())
                            .unwrap_or("Unknown Title");
                        let author = entry
                            .fields
                            .get("author")
                            .map(|s| s.as_str())
                            .unwrap_or("Unknown Author");
                        let year = entry
                            .fields
                            .get("year")
                            .map(|s| s.as_str())
                            .unwrap_or("????");

                        return Some(format!("**{}**\n{} ({})", title, author, year));
                    }
                }
            }
            None
        };

        if !referenced_bibs.is_empty() {
            if let Some(res) = find_in_bibs(&referenced_bibs) {
                return Some(res);
            }
        }

        // Fallback: search all known bibs if not found in referenced ones (loose mode)
        let all_uris: Vec<Url> = self.bib_indices.iter().map(|e| e.key().clone()).collect();
        find_in_bibs(&all_uris)
    }

    /// Finds all definitions of a label by name.
    ///
    /// Returns a list of (File URI, Range) pairs.
    pub fn find_definitions(&self, name: &str) -> Vec<(Url, TextRange)> {
        let mut results = Vec::new();
        for entry in self.indices.iter() {
            for def in &entry.value().definitions {
                if def.name == name {
                    results.push((entry.key().clone(), def.range));
                }
            }
        }
        results
    }

    /// Finds all references to a label by name.
    ///
    /// Returns a list of (File URI, Range) pairs.
    pub fn find_references(&self, name: &str) -> Vec<(Url, TextRange)> {
        let mut results = Vec::new();
        for entry in self.indices.iter() {
            for r in &entry.value().references {
                if r.name == name {
                    results.push((entry.key().clone(), r.range));
                }
            }
        }
        results
    }

    /// Searches for symbols across the workspace matching the query string.
    ///
    /// Returns a list of (Name, Kind, File URI, Range) tuples.
    pub fn query_symbols(&self, query: &str) -> Vec<(String, SymbolKind, Url, TextRange)> {
        let mut results = Vec::new();
        let query = query.to_lowercase();

        // 1. Search TeX files (Labels and Sections)
        for entry in self.indices.iter() {
            let uri = entry.key();
            let index = entry.value();

            // Labels
            for def in &index.definitions {
                if def.name.to_lowercase().contains(&query) {
                    results.push((
                        def.name.clone(),
                        SymbolKind::CONSTANT, // Labels are like constants
                        uri.clone(),
                        def.range,
                    ));
                }
            }

            // Sections
            for section in &index.sections {
                if section.name.to_lowercase().contains(&query) {
                    results.push((
                        section.name.clone(),
                        SymbolKind::STRING, // Sections are structural/strings
                        uri.clone(),
                        section.range,
                    ));
                }
            }

            // Environments
            for env in &index.environments {
                if env.name.to_lowercase().contains(&query) {
                    results.push((
                        env.name.clone(),
                        SymbolKind::NAMESPACE,
                        uri.clone(),
                        env.range,
                    ));
                }
            }

            // Macros
            for mac in &index.macros {
                if mac.name.to_lowercase().contains(&query) {
                    results.push((
                        mac.name.clone(),
                        SymbolKind::FUNCTION,
                        uri.clone(),
                        mac.definition_range,
                    ));
                }
            }
        }

        // 2. Search BibTeX files (Entries)
        for entry in self.bib_indices.iter() {
            let uri = entry.key();
            let bib_file = entry.value();

            for bib_entry in &bib_file.entries {
                if bib_entry.key.to_lowercase().contains(&query) {
                    results.push((
                        bib_entry.key.clone(),
                        SymbolKind::CLASS, // Bib entries are like classes/records
                        uri.clone(),
                        bib_entry.range,
                    ));
                }
            }
        }

        results
    }

    // --- Diagnostics ---

    pub fn validate_bibliographies(&self) -> Vec<(Url, TextRange, String)> {
        let mut diagnostics = Vec::new();

        for entry in self.indices.iter() {
            let base_uri = entry.key();
            for bib in &entry.value().bibliographies {
                let Some(uri) = resolve_bib_uri(base_uri, &bib.path) else {
                    diagnostics.push((
                        base_uri.clone(),
                        bib.range,
                        format!("Invalid bibliography path: '{}'", bib.path),
                    ));
                    continue;
                };

                if !self.bib_indices.contains_key(&uri) {
                    diagnostics.push((
                        base_uri.clone(),
                        bib.range,
                        format!("Missing bibliography file: '{}'", bib.path),
                    ));
                }
            }
        }

        diagnostics
    }

    /// Validates citations across the workspace.
    ///
    /// Returns a list of diagnostics for undefined citations.
    pub fn validate_citations(&self) -> Vec<(Url, TextRange, String)> {
        let mut diagnostics = Vec::new();

        let referenced_bibs = self.get_referenced_bib_uris();
        if !referenced_bibs.is_empty()
            && !referenced_bibs
                .iter()
                .all(|uri| self.bib_indices.contains_key(uri))
        {
            return diagnostics;
        }

        // Check for undefined citations
        for entry in self.indices.iter() {
            for cite in &entry.value().citations {
                if !self.has_citation_key(&cite.key) {
                    diagnostics.push((
                        entry.key().clone(),
                        cite.range,
                        format!("Undefined citation: '{}'", cite.key),
                    ));
                }
            }
        }

        diagnostics
    }

    /// Validates labels across the workspace.
    ///
    /// Checks for duplicate label definitions and undefined references.
    pub fn validate_labels(&self) -> Vec<(Url, TextRange, String)> {
        let mut diagnostics = Vec::new();

        // 1. Gather all definitions to check for duplicates
        let mut defs_by_name: HashMap<String, Vec<(Url, TextRange)>> = HashMap::new();
        for entry in self.indices.iter() {
            for def in &entry.value().definitions {
                defs_by_name
                    .entry(def.name.clone())
                    .or_default()
                    .push((entry.key().clone(), def.range));
            }
        }

        // 2. Report duplicates
        for (name, locs) in &defs_by_name {
            if locs.len() > 1 {
                for (uri, range) in locs {
                    diagnostics.push((
                        uri.clone(),
                        *range,
                        format!("Duplicate label definition: '{}'", name),
                    ));
                }
            }
        }

        // 3. Check for undefined references
        for entry in self.indices.iter() {
            for r in &entry.value().references {
                if !defs_by_name.contains_key(&r.name) {
                    diagnostics.push((
                        entry.key().clone(),
                        r.range,
                        format!("Undefined reference: '{}'", r.name),
                    ));
                }
            }
        }

        diagnostics
    }

    /// Validates usage of deprecated commands.
    pub fn validate_deprecated(&self) -> Vec<(Url, TextRange, String)> {
        let mut diagnostics = Vec::new();

        for entry in self.indices.iter() {
            for (range, cmd) in &entry.value().deprecated_usages {
                diagnostics.push((
                    entry.key().clone(),
                    *range,
                    format!(
                        "Command '{}' is deprecated. Use standard LaTeX2e replacements.",
                        cmd
                    ),
                ));
            }
        }
        diagnostics
    }

    /// Detects inclusion cycles in the workspace.
    ///
    /// Performs a DFS on the inclusion graph to find cycles.
    pub fn detect_cycles(&self) -> Vec<(Url, TextRange, String)> {
        let mut cycles = Vec::new();
        // Snapshot of the graph to avoid locking issues during traversal
        // Map: Url -> Vec<(ResolvedUrl, Range, PathString)>
        let mut graph: HashMap<Url, Vec<(Url, TextRange, String)>> = HashMap::new();

        for entry in self.indices.iter() {
            let base_uri = entry.key();
            let refs = &entry.value().includes;
            let mut edges = Vec::new();
            for r in refs {
                // Best-effort resolution
                // We assume paths are relative to the document location
                if let Ok(target) = base_uri.join(&r.path) {
                    edges.push((target, r.range, r.path.clone()));
                }
            }
            graph.insert(base_uri.clone(), edges);
        }

        let nodes: Vec<Url> = graph.keys().cloned().collect();

        // Run DFS from *each* node to find all back-edges.
        for node in &nodes {
            let mut visited = HashSet::new();
            self.check_cycle_dfs(node, &graph, &mut visited, &mut Vec::new(), &mut cycles);
        }

        // Deduplicate cycles
        let mut unique_cycles = Vec::new();
        for cycle in cycles {
            let is_duplicate = unique_cycles
                .iter()
                .any(|(u, r, m)| u == &cycle.0 && r == &cycle.1 && m == &cycle.2);
            if !is_duplicate {
                unique_cycles.push(cycle);
            }
        }

        unique_cycles
    }

    #[allow(clippy::only_used_in_recursion)]
    fn check_cycle_dfs(
        &self,
        current: &Url,
        graph: &HashMap<Url, Vec<(Url, TextRange, String)>>,
        visited: &mut HashSet<Url>,
        path_stack: &mut Vec<Url>, // Gray nodes
        cycles: &mut Vec<(Url, TextRange, String)>,
    ) {
        path_stack.push(current.clone());
        visited.insert(current.clone());

        if let Some(edges) = graph.get(current) {
            for (target, range, raw_path) in edges {
                if path_stack.contains(target) {
                    // Cycle detected!
                    let msg = format!(
                        "Cycle detected: '{}' includes ancestor {}",
                        raw_path, target
                    );
                    cycles.push((current.clone(), *range, msg));
                } else if !visited.contains(target) {
                    self.check_cycle_dfs(target, graph, visited, path_stack, cycles);
                }
            }
        }

        path_stack.pop();
        // Do NOT remove from visited, to avoid re-scanning subgraphs in this DFS run.
    }
}

type ScanResult = (
    Vec<IncludeRef>,
    Vec<LabelDef>,
    Vec<LabelRef>,
    Vec<CitationRef>,
    Vec<BibRef>,
    Vec<SectionDef>,
    Vec<String>,              // packages
    Option<String>,           // magic_root
    Vec<(TextRange, String)>, // deprecated_usages
    Vec<EnvDef>,              // environments
    Vec<MacroDef>,            // macros
);

fn scan_file(text: &str) -> ScanResult {
    // Scan for magic comments in the first 1KB
    let head = if text.len() > 1024 {
        &text[..1024]
    } else {
        text
    };

    // Pattern: %!TEX root = <path>
    // Handles optional spaces around = and leading whitespace
    let re = Regex::new(r"(?mi)^%\s*!TEX\s+root\s*=\s*(.+)$").unwrap();
    let magic_root = re.captures(head).map(|cap| cap[1].trim().to_string());

    let parse = parse(text);
    let root = parse.syntax();
    let mut includes = Vec::new();
    let mut defs = Vec::new();
    let mut refs = Vec::new();
    let mut citations = Vec::new();
    let mut bibs = Vec::new();
    let mut sections = Vec::new();
    let mut deprecated_usages = Vec::new();
    let mut environments = Vec::new();

    let mut last_was_dollar = false;
    let mut last_dollar_range: Option<TextRange> = None;
    let mut opening_display_math: Option<TextRange> = None;
    let mut last_cmd: Option<(String, TextRange)> = None; // Track pending command (name, range)

    for element in root.descendants_with_tokens() {
        match element.kind() {
            SyntaxKind::Dollar => {
                last_cmd = None;
                if last_was_dollar {
                    // ... math logic ...
                    if let Some(prev_range) = last_dollar_range {
                        if prev_range.end() == element.text_range().start() {
                            let combined_range =
                                TextRange::new(prev_range.start(), element.text_range().end());
                            if let Some(opening_range) = opening_display_math {
                                let full_block_range =
                                    TextRange::new(opening_range.start(), combined_range.end());
                                deprecated_usages
                                    .push((full_block_range, "displaymath".to_string()));
                                opening_display_math = None;
                            } else {
                                opening_display_math = Some(combined_range);
                            }
                            last_was_dollar = false;
                            last_dollar_range = None;
                            continue;
                        }
                    }
                }
                last_was_dollar = true;
                last_dollar_range = Some(element.text_range());
            }
            _ => {
                last_was_dollar = false;
                last_dollar_range = None;

                if element.kind() != SyntaxKind::Whitespace
                    && element.kind() != SyntaxKind::Command
                    && element.kind() != SyntaxKind::Comment
                    && element.as_node().is_none()
                {
                    last_cmd = None;
                }

                if element.kind() == SyntaxKind::Command {
                    let text = element.to_string();
                    let deprecated = ["\\bf", "\\it", "\\sc", "\\rm", "\\sf", "\\tt", "\\sl"];
                    if deprecated.contains(&text.trim()) {
                        // trim needed?
                        // ... deprecated logic ...
                        let mut in_group = false;
                        let mut group_range = element.text_range();
                        if let Some(token) = element.as_token() {
                            if let Some(parent) = token.parent() {
                                if parent.kind() == SyntaxKind::Group {
                                    in_group = true;
                                    group_range = parent.text_range();
                                }
                            }
                        }
                        let context_marker = if in_group {
                            format!("{}:group", text)
                        } else {
                            text.clone()
                        };
                        deprecated_usages.push((
                            if in_group {
                                group_range
                            } else {
                                element.text_range()
                            },
                            context_marker,
                        ));
                    }

                    let cmd_name = if let Some(idx) = text.find(['{', '[', ' ']) {
                        &text[..idx]
                    } else {
                        text.trim()
                    };

                    let interesting_cmds = [
                        "\\section",
                        "\\subsection",
                        "\\subsubsection",
                        "\\chapter",
                        "\\paragraph",
                        "\\subparagraph",
                        "\\label",
                        "\\ref",
                        "\\cite",
                        "\\bibliography",
                        "\\include",
                        "\\input",
                    ];

                    if interesting_cmds.contains(&cmd_name) {
                        // 1. Try immediate extraction from text (e.g. \cmd{arg} as one token)
                        let mut captured_arg = None;
                        if let Some(start) = text.find('{') {
                            if let Some(end) = text.rfind('}') {
                                if end > start {
                                    captured_arg = Some(text[start + 1..end].to_string());
                                }
                            }
                        }

                        // 2. Try nested structure (children)
                        if captured_arg.is_none() {
                            if let Some(node) = element.as_node() {
                                if let Some((name, _)) = extract_label_data(node) {
                                    captured_arg = Some(name);
                                }
                            }
                        }

                        if let Some(arg) = captured_arg {
                            // Process immediately
                            let range = element.text_range();
                            match cmd_name {
                                "\\label" => defs.push(LabelDef { name: arg, range }),
                                "\\ref" => refs.push(LabelRef { name: arg, range }),
                                "\\bibliography" => {
                                    for path in arg.split(',') {
                                        bibs.push(BibRef {
                                            path: path.trim().to_string(),
                                            range,
                                        });
                                    }
                                }
                                "\\include" | "\\input" => {
                                    includes.push(IncludeRef { path: arg, range })
                                }
                                "\\cite" => {
                                    for key in arg.split(',') {
                                        citations.push(CitationRef {
                                            key: key.trim().to_string(),
                                            range,
                                        });
                                    }
                                }
                                _ => sections.push(SectionDef { name: arg, range }), // Sections
                            }
                            last_cmd = None;
                        } else {
                            // Pending for next group
                            last_cmd = Some((cmd_name.to_string(), element.text_range()));
                        }
                    } else {
                        last_cmd = None;
                    }
                } else if let Some(node) = element.as_node() {
                    match node.kind() {
                        // Redundant handlers removed.
                        SyntaxKind::Bibliography => {
                            // Handled by SyntaxKind::Command
                            last_cmd = None;
                        }
                        SyntaxKind::Group => {
                            if let Some((cmd_name, cmd_range)) = last_cmd.take() {
                                let text = node.text().to_string();
                                let content = if text.starts_with('{') && text.ends_with('}') {
                                    &text[1..text.len() - 1]
                                } else {
                                    &text
                                };
                                let arg = content.to_string();

                                match cmd_name.as_str() {
                                    "\\label" => defs.push(LabelDef {
                                        name: arg,
                                        range: cmd_range,
                                    }),
                                    "\\ref" => refs.push(LabelRef {
                                        name: arg,
                                        range: cmd_range,
                                    }),
                                    "\\bibliography" => {
                                        for path in arg.split(',') {
                                            bibs.push(BibRef {
                                                path: path.trim().to_string(),
                                                range: cmd_range,
                                            });
                                        }
                                    }
                                    "\\include" | "\\input" => includes.push(IncludeRef {
                                        path: arg,
                                        range: cmd_range,
                                    }),
                                    "\\cite" => {
                                        for key in arg.split(',') {
                                            citations.push(CitationRef {
                                                key: key.trim().to_string(),
                                                range: cmd_range,
                                            });
                                        }
                                    }
                                    _ => sections.push(SectionDef {
                                        name: arg,
                                        range: cmd_range,
                                    }), // Sections
                                }
                            }
                        }
                        SyntaxKind::Environment => {
                            if let Some((name, _range)) = extract_label_data(node) {
                                environments.push(EnvDef {
                                    name,
                                    range: node.text_range(),
                                });
                            }
                            last_cmd = None;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    // Scan for packages
    // Pattern: \usepackage[opt]{pkg} or \RequirePackage[opt]{pkg}
    // We ignore options for now.
    // Scan for packages
    let text_str = root.text().to_string();
    let re = Regex::new(r"\\usepackage(?:\[[^\]]*\])?\{([^}]+)\}").unwrap();
    let mut packages = Vec::new();

    for cap in re.captures_iter(&text_str) {
        if let Some(pkg_group_match) = cap.get(1) {
            for pkg in pkg_group_match.as_str().split(',') {
                let trimmed = pkg.trim();
                if !trimmed.is_empty() {
                    packages.push(trimmed.to_string());

                    let forbidden = ["a4wide", "times", "epsfig", "psfig"];
                    if forbidden.contains(&trimmed) {
                        // Calculate exact range of the package name
                        use ferrotex_syntax::TextSize;
                        let relative_start_in_group =
                            pkg_group_match.as_str().find(trimmed).unwrap_or(0);
                        let absolute_start = pkg_group_match.start() + relative_start_in_group;
                        let absolute_end = absolute_start + trimmed.len();

                        let range = TextRange::new(
                            TextSize::from(absolute_start as u32),
                            TextSize::from(absolute_end as u32),
                        );
                        deprecated_usages.push((range, format!("package:{}", trimmed)));
                    }
                }
            }
        }
    }

    let macros = scan_macros(&root);

    (
        includes,
        defs,
        refs,
        citations,
        bibs,
        sections,
        packages,
        magic_root,
        deprecated_usages,
        environments,
        macros,
    )
}

pub fn extract_group_text(node: &ferrotex_syntax::SyntaxNode) -> Option<String> {
    extract_label_data(node).map(|(name, _)| name)
}

pub fn extract_label_data(node: &ferrotex_syntax::SyntaxNode) -> Option<(String, TextRange)> {
    let group = if node.kind() == SyntaxKind::Group {
        node.clone()
    } else {
        node.children().find(|n| n.kind() == SyntaxKind::Group)?
    };
    let text = group.text().to_string();
    let range = group.text_range();

    // Expected format: "{...}"
    if !text.starts_with('{') {
        return None;
    }

    let content_start = 1;
    let content_end = if text.ends_with('}') {
        text.len() - 1
    } else {
        text.len()
    };

    if content_start >= content_end {
        // Empty "{}"
        use ferrotex_syntax::TextSize;
        let pos = range.start() + TextSize::from(1);
        return Some((String::new(), TextRange::new(pos, pos)));
    }

    let content = &text[content_start..content_end];
    let trimmed = content.trim();
    let trim_start = content.find(trimmed).unwrap_or(0); // byte offset inside content

    use ferrotex_syntax::TextSize;
    let final_start = range.start() + TextSize::from((content_start + trim_start) as u32);
    let final_len = TextSize::from(trimmed.len() as u32);

    Some((trimmed.to_string(), TextRange::at(final_start, final_len)))
}

fn resolve_bib_uri(base_uri: &Url, raw_path: &str) -> Option<Url> {
    let mut path = raw_path.trim().trim_matches('"').to_string();
    if path.is_empty() {
        return None;
    }

    let has_extension = std::path::Path::new(&path).extension().is_some();
    if !has_extension {
        path.push_str(".bib");
    }

    base_uri.join(&path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deprecated_command() {
        let text = r#"\section{Test} {\bf bold} text"#;
        let result = scan_file(text);
        let deprecated = result.8; // deprecated_usages
        assert!(!deprecated.is_empty(), "Should detect deprecated command");
        assert_eq!(deprecated[0].1, "\\bf:group");
    }

    #[test]
    fn test_deprecated_math_detection() {
        let text = r#"
        Text
        $$
        x = y
        $$
        End
        "#;
        let result = scan_file(text);
        let deprecated = result.8;
        assert!(
            deprecated.iter().any(|d| d.1 == "displaymath"),
            "Should detect display math block"
        );
    }

    #[test]
    fn test_scan_file_commands() {
        let text = r#"
        \section{Sec 1}
        \subsection{Sub 1}
        \subsubsection{SubSub 1}
        \paragraph{Para 1}
        \subparagraph{SubPara 1}
        \chapter{Chap 1}
        \label{lbl:1}
        \ref{lbl:1}
        \cite{ref:1}
        \bibliography{bib1}
        \include{inc1}
        \input{inp1}
        "#;
        let res = scan_file(text);

        // Sections
        assert_eq!(res.5.len(), 6); // sec, subsec, subsub, para, subpara, chapter

        // Defs
        assert_eq!(res.1.len(), 1);
        assert_eq!(res.1[0].name, "lbl:1");

        // Refs
        assert_eq!(res.2.len(), 1);
        assert_eq!(res.2[0].name, "lbl:1");

        // Citations
        assert_eq!(res.3.len(), 1);
        assert_eq!(res.3[0].key, "ref:1");

        // Bibs
        assert_eq!(res.4.len(), 1);
        assert_eq!(res.4[0].path, "bib1");

        // Includes
        assert_eq!(res.0.len(), 2);
        assert!(res.0.iter().any(|i| i.path == "inc1"));
        assert!(res.0.iter().any(|i| i.path == "inp1"));
    }

    #[test]
    fn test_scan_file_complex_packages() {
        let text = r"\usepackage[opt=1]{pkg1, pkg2} \RequirePackage{pkg3}";
        // Note: scan_file implementation regex only matches \usepackage currently?
        // Let's check regex: r"\\usepackage(?:\[[^\]]*\])?\{([^}]+)\}"
        // It does NOT match RequirePackage.

        let res = scan_file(text);
        assert!(res.6.contains(&"pkg1".to_string()));
        assert!(res.6.contains(&"pkg2".to_string()));
        assert!(!res.6.contains(&"pkg3".to_string()));
    }

    #[test]
    fn test_extract_label_data_edge_cases() {
        use ferrotex_syntax::parse;
        let text = r"{} { } {label}";
        let p = parse(text);
        let root = p.syntax();
        let children: Vec<_> = root.children().collect();

        // {}
        assert_eq!(extract_group_text(&children[0]), Some("".to_string()));

        // { }
        assert_eq!(extract_group_text(&children[1]), Some("".to_string()));

        // {label}
        assert_eq!(extract_group_text(&children[2]), Some("label".to_string()));
    }

    #[test]
    fn test_resolve_bib_uri_edge_cases() {
        let base = Url::parse("file:///root/main.tex").unwrap();

        // Empty
        assert!(resolve_bib_uri(&base, "").is_none());
        assert!(resolve_bib_uri(&base, "   ").is_none());

        // With extension
        let res = resolve_bib_uri(&base, "ref.bib").unwrap();
        assert_eq!(res.to_string(), "file:///root/ref.bib");

        // Without extension
        let res2 = resolve_bib_uri(&base, "ref").unwrap();
        assert_eq!(res2.to_string(), "file:///root/ref.bib");

        // Quotes
        let res3 = resolve_bib_uri(&base, "\"ref\"").unwrap();
        assert_eq!(res3.to_string(), "file:///root/ref.bib");
    }

    #[test]
    fn test_scan_file_environments() {
        let text = r"\begin{env1} \end{env1} \begin{env2} inner \end{env2}";
        let res = scan_file(text);
        assert_eq!(res.9.len(), 2);
        assert!(res.9.iter().any(|e| e.name == "env1"));
        assert!(res.9.iter().any(|e| e.name == "env2"));
    }

    #[test]
    fn test_obsolete_package_detection() {
        let text = r#"\usepackage{times, geometry}"#;
        let result = scan_file(text);
        let deprecated = result.8;
        assert!(
            deprecated.iter().any(|d| d.1 == "package:times"),
            "Should detect 'times' package"
        );
        assert!(
            !deprecated.iter().any(|d| d.1 == "package:geometry"),
            "Should NOT detect 'geometry' package"
        );
    }

    #[test]
    fn test_workspace_cross_file_labels() {
        let workspace = Workspace::new();
        let uri1 = Url::parse("file:///main.tex").unwrap();
        let uri2 = Url::parse("file:///sub.tex").unwrap();

        workspace.update(&uri1, r"\label{lbl1}");
        workspace.update(&uri2, r"\label{lbl2}");

        let labels = workspace.get_all_labels();
        assert_eq!(labels.len(), 2);
        assert!(labels.contains(&"lbl1".to_string()));
        assert!(labels.contains(&"lbl2".to_string()));
    }

    #[test]
    fn test_workspace_cycle_detection() {
        let workspace = Workspace::new();
        let uri1 = Url::parse("file:///a.tex").unwrap();
        let uri2 = Url::parse("file:///b.tex").unwrap();

        // A includes B, B includes A
        workspace.update(&uri1, r"\include{b.tex}");
        workspace.update(&uri2, r"\include{a.tex}");

        let cycles = workspace.detect_cycles();
        assert!(!cycles.is_empty(), "Cycle should be detected");
    }

    #[test]
    fn test_workspace_bib_indexing() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///refs.bib").unwrap();
        let text = "@article{key1, title={Title}}";

        workspace.update_bib(&uri, text);
        assert!(workspace.has_citation_key("key1"));
        assert!(!workspace.has_citation_key("key2"));
    }

    #[test]
    fn test_magic_root_detection() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///chapter.tex").unwrap();
        let text = "% !TeX root = main.tex\nContent";

        workspace.update(&uri, text);
        assert_eq!(
            workspace.get_explicit_root(&uri),
            Some("main.tex".to_string())
        );
    }

    #[test]
    fn test_workspace_sections() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///main.tex").unwrap();
        // \section should be parsed and added to sections list
        workspace.update(&uri, r"\section{Introduction}");

        let index = workspace.indices.get(&uri).unwrap();
        assert_eq!(index.sections.len(), 1);
        assert_eq!(index.sections[0].name, "Introduction");
    }

    #[test]
    fn test_macro_definition_extraction() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///macros.tex").unwrap();
        let text = r"
            \newcommand{\simple}{Hello}
            \newcommand{\withargs}[2]{Arg #1 and #2}
            \newcommand{\optarg}[2][default]{Opt #1, Mand #2}
        ";

        workspace.update(&uri, text);
        let index = workspace.indices.get(&uri).unwrap();

        assert_eq!(index.macros.len(), 3);

        let simple = index.macros.iter().find(|m| m.name == "\\simple").unwrap();
        assert_eq!(simple.args, 0);
        assert!(!simple.has_optional);

        let withargs = index
            .macros
            .iter()
            .find(|m| m.name == "\\withargs")
            .unwrap();
        assert_eq!(withargs.args, 2);
        assert!(!withargs.has_optional);

        let optarg = index.macros.iter().find(|m| m.name == "\\optarg").unwrap();
        assert_eq!(optarg.args, 2);
        assert!(optarg.has_optional);
    }
    #[test]
    fn test_workspace_full_scan() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///full.tex").unwrap();
        let text = r#"
% !TEX root = master.tex

\documentclass{article}
\usepackage{amsmath, geometry}
\usepackage[utf8]{inputenc}

\newcommand{\mycmd}{My Command}

\begin{document}
    \section{Introduction}
    \label{sec:intro}
    This is a test document with \cite{ref1, ref2}.
    See Section \ref{sec:intro}.

    \input{chapter1.tex}

    \begin{equation}
        E = mc^2
    \end{equation}

    Some deprecated usage: {\bf bold text} and $$ x = y $$ display math.

    \bibliography{refs}
\end{document}
        "#;

        workspace.update(&uri, text);

        // DEBUG: Print syntax tree
        let parse = parse(text);
        println!("Syntax Tree: {:#?}", parse.syntax());

        let index = workspace.indices.get(&uri).expect("Index not created");
        println!("Found Sections: {:?}", index.sections);

        // Verify Magic Root
        assert_eq!(
            workspace.get_explicit_root(&uri),
            Some("master.tex".to_string())
        );

        // Verify Packages
        assert!(index.packages.contains(&"amsmath".to_string()));
        assert!(index.packages.contains(&"geometry".to_string()));
        assert!(index.packages.contains(&"inputenc".to_string()));

        // Verify Sections
        assert_eq!(index.sections.len(), 1);
        assert_eq!(index.sections[0].name, "Introduction");

        // Verify Labels & Refs
        assert_eq!(index.definitions.len(), 1);
        assert_eq!(index.definitions[0].name, "sec:intro");
        assert_eq!(index.references.len(), 1);
        assert_eq!(index.references[0].name, "sec:intro");

        // Verify Citations
        assert_eq!(index.citations.len(), 2);
        assert!(index.citations.iter().any(|c| c.key == "ref1"));
        assert!(index.citations.iter().any(|c| c.key == "ref2"));

        // Verify Includes
        assert_eq!(index.includes.len(), 1);
        assert_eq!(index.includes[0].path, "chapter1.tex");

        // Verify Bibliographies
        assert_eq!(index.bibliographies.len(), 1);
        assert_eq!(index.bibliographies[0].path, "refs");

        // Verify Environments
        let eqs = index
            .environments
            .iter()
            .filter(|e| e.name == "equation")
            .count();
        assert_eq!(eqs, 1);
        let docs = index
            .environments
            .iter()
            .filter(|e| e.name == "document")
            .count();
        assert_eq!(docs, 1);

        // Verify Macros
        assert_eq!(index.macros.len(), 1);
        assert_eq!(index.macros[0].name, "\\mycmd");

        // Verify Deprecation
        // {\bf ...} creates one, $$ ... $$ creates another
        // Note: The parser might perform recovery or structure things differently
        // But scan_file looks for SyntaxKind::Dollar sequences for $$
        let bf_dep = index
            .deprecated_usages
            .iter()
            .any(|(_, msg)| msg.contains("\\bf"));
        assert!(bf_dep, "Should detect \\bf deprecated usage");

        let math_dep = index
            .deprecated_usages
            .iter()
            .any(|(_, msg)| msg == "displaymath");
        assert!(math_dep, "Should detect display math $$ usage");
    }

    #[test]
    fn test_workspace_lookup_apis() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///lookup.tex").unwrap();
        let text = r#"
\section{My Section}
\label{sec:my}
\ref{sec:my}
\newcommand{\mycmd}{cmd}
        "#;
        workspace.update(&uri, text);

        // Test find_definitions
        let defs = workspace.find_definitions("sec:my");
        assert_eq!(defs.len(), 1, "Should find definition");
        assert_eq!(defs[0].0, uri);

        // Test find_references
        let refs = workspace.find_references("sec:my");
        assert_eq!(refs.len(), 1, "Should find reference");
        assert_eq!(refs[0].0, uri);

        // Test query_symbols
        // 1. Label
        let symbols = workspace.query_symbols("sec:my");
        assert!(
            symbols
                .iter()
                .any(|(name, kind, _, _)| name == "sec:my" && *kind == SymbolKind::CONSTANT),
            "Should find label symbol"
        );

        // 2. Section
        let symbols = workspace.query_symbols("my section");
        assert!(
            symbols
                .iter()
                .any(|(name, kind, _, _)| name == "My Section" && *kind == SymbolKind::STRING),
            "Should find section symbol"
        );

        // 3. Macro
        let symbols = workspace.query_symbols("mycmd");
        assert!(
            symbols
                .iter()
                .any(|(name, kind, _, _)| name == "\\mycmd" && *kind == SymbolKind::FUNCTION),
            "Should find macro symbol"
        );
    }

    #[test]
    fn test_validate_bibliographies() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///main.tex").unwrap();
        // Invalid path
        workspace.update(&uri, r"\bibliography{missing}");
        let diags = workspace.validate_bibliographies();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].2.contains("Missing bibliography"));

        // Valid path but missing file
        let bib_uri = Url::parse("file:///refs.bib").unwrap();
        // If I update with bibliography that exists
        workspace.update_bib(&bib_uri, "");
        workspace.update(&uri, r"\bibliography{refs.bib}");
        let diags2 = workspace.validate_bibliographies();
        assert!(diags2.is_empty(), "Should resolve existing bib");
    }

    #[test]
    fn test_validate_citations_and_labels() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///doc.tex").unwrap();

        // 1. Undefined citation
        workspace.update(&uri, r"\cite{undef}");
        let diags = workspace.validate_citations();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].2.contains("Undefined citation"));

        // 2. Defined citation
        let bib_uri = Url::parse("file:///refs.bib").unwrap();
        workspace.update_bib(&bib_uri, "@article{defined, title={T}}");
        // We need to link bib to doc
        workspace.update(&uri, r"\bibliography{refs.bib} \cite{defined}");
        let diags2 = workspace.validate_citations();
        assert!(diags2.is_empty());

        // 3. Duplicate labels
        workspace.update(&uri, r"\label{dup} \label{dup}");
        let label_diags = workspace.validate_labels();
        assert!(!label_diags.is_empty());
        assert!(label_diags[0].2.contains("Duplicate label definition")); // "Duplicate label definition"

        // 4. Undefined reference
        workspace.update(&uri, r"\ref{missing}");
        let ref_diags = workspace.validate_labels();
        // Should contain duplicate label error AND undefined reference
        // workspace state has duplicated labels from prev step + undefined ref
        assert!(ref_diags
            .iter()
            .any(|d| d.2.contains("Undefined reference")));
    }

    #[test]
    fn test_get_packages_inheritance() {
        let workspace = Workspace::new();
        let root_uri = Url::parse("file:///root.tex").unwrap();
        let sub_uri = Url::parse("file:///sub.tex").unwrap();

        workspace.update(&root_uri, r"\usepackage{rootpkg}");
        workspace.update(
            &sub_uri,
            r"%!TEX root = root.tex
        \usepackage{subpkg}",
        );

        let pkgs = workspace.get_packages(&sub_uri);
        assert!(pkgs.contains(&"rootpkg".to_string()));
        assert!(pkgs.contains(&"subpkg".to_string()));
    }

    #[test]
    fn test_citation_details() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///doc.tex").unwrap();
        let bib_uri = Url::parse("file:///refs.bib").unwrap();

        workspace.update_bib(
            &bib_uri,
            "@article{key1, title={Test Title}, author={Author}, year={2023}}",
        );
        workspace.update(&uri, r"\bibliography{refs.bib}");

        let details = workspace.get_citation_details("key1");
        assert!(details.is_some());
        let d = details.unwrap();
        assert!(d.contains("Test Title"));
        assert!(d.contains("Author"));
        assert!(d.contains("2023"));

        let none = workspace.get_citation_details("missing");
        assert!(none.is_none());
    }

    #[test]
    fn test_scan_file_commands_split_args() {
        let text = r"\section {Split Title} \label {split:lbl}";
        let res = scan_file(text);

        assert_eq!(res.5.len(), 1);
        assert_eq!(res.5[0].name, "Split Title");

        assert_eq!(res.1.len(), 1);
        assert_eq!(res.1[0].name, "split:lbl");
    }

    #[test]
    fn test_workspace_diamond_dependency() {
        let workspace = Workspace::new();
        let uri_a = Url::parse("file:///a.tex").unwrap();
        let uri_b = Url::parse("file:///b.tex").unwrap();
        let uri_c = Url::parse("file:///c.tex").unwrap();

        workspace.update(&uri_a, r"\include{b.tex} \include{c.tex}");
        workspace.update(&uri_b, r"\include{c.tex}");
        workspace.update(&uri_c, "Content");

        let cycles = workspace.detect_cycles();
        assert!(cycles.is_empty(), "Diamond dependency is not a cycle");
    }

    #[test]
    fn test_validate_empty_bibliography() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///main.tex").unwrap();
        workspace.update(&uri, r"\bibliography{ }");
        let diags = workspace.validate_bibliographies();
        // scan_file parses "\bibliography{ }" -> bibs entry with path "".
        // resolve_bib_uri("") returns None.
        // So we expect "Invalid bibliography path".
        assert!(!diags.is_empty());
        assert!(diags[0].2.contains("Invalid bibliography path"));
    }

    #[test]
    fn test_validate_citations_missing_bib() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///doc.tex").unwrap();

        // We reference a bib that doesn't exist, and use a citation
        workspace.update(&uri, r"\bibliography{missing} \cite{something}");

        let diags = workspace.validate_citations();
        // Should be empty because we suppress citation errors if bib is missing
        assert!(diags.is_empty());
    }

    #[test]
    fn test_workspace_remove_file() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///temp.tex").unwrap();
        workspace.update(&uri, r"\label{lost}");

        assert!(workspace.indices.contains_key(&uri));
        assert!(workspace.get_all_labels().contains(&"lost".to_string()));

        workspace.remove(&uri);
        assert!(!workspace.indices.contains_key(&uri));
        assert!(!workspace.get_all_labels().contains(&"lost".to_string()));
    }

    #[test]
    fn test_validate_deprecated_diagnostics() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///legacy.tex").unwrap();
        let text = r"Old style {\bf bold}";
        workspace.update(&uri, text);

        let diags = workspace.validate_deprecated();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].0, uri);
        assert!(diags[0].2.contains("deprecated"));
    }

    #[test]
    fn test_scan_file_detached_args() {
        // Try to force parser to produce detached command/group nodes
        // Using comments usually breaks string scan logic
        // if string scan blindly regexes? No, string scan uses find('{')
        // \section % comment
        // {Title}
        let text = "\\section % comment\n{Title}";
        let res = scan_file(text);

        assert_eq!(res.5.len(), 1);
        assert_eq!(res.5[0].name, "Title");
    }

    #[test]
    fn test_obsolete_package_offset() {
        // Test offset calculation when obsolete package is NOT at start
        let text = r"\usepackage{ valid, times }";
        let res = scan_file(text);

        // Should detect times
        let deprecated = res.8;
        assert_eq!(deprecated.len(), 1);
        assert!(deprecated[0].1.contains("package:times"));

        // Verify range is correct (not 0)
        let range = deprecated[0].0;
        assert!(u32::from(range.start()) > 10);
    }

    #[test]
    fn test_workspace_getters() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///main.tex").unwrap();

        // 1. Includes
        workspace.update(&uri, r"\include{chap1} \bibliography{refs}");
        let incs = workspace.get_includes(&uri);
        assert_eq!(incs.len(), 1);
        assert_eq!(incs[0].path, "chap1");

        // 2. Bibliographies
        let bibs = workspace.get_bibliographies(&uri);
        assert_eq!(bibs.len(), 1);
        assert_eq!(bibs[0].path, "refs");

        // 3. All Citations (with empty workspace first)
        assert!(workspace.get_all_citation_keys().is_empty());

        // 4. Add BibTeX
        let bib_uri = Url::parse("file:///refs.bib").unwrap();
        workspace.update_bib(&bib_uri, "@article{k1, t={T}}");
        // Update main to ref bib
        workspace.update(&uri, r"\bibliography{refs.bib}");

        let keys = workspace.get_all_citation_keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "k1");
    }

    #[test]
    fn test_scan_file_edge_large_input() {
        // Create large input > 1024 bytes to trigger truncation check in magic root scan
        let mut text = String::with_capacity(2048);
        for _ in 0..1100 {
            text.push(' ');
        }
        text.push_str("% !TeX root = hidden.tex"); // This should be ignored as it's too far

        let res = scan_file(&text);
        assert_eq!(res.7, None, "Magic root should be ignored if not in header");

        // But if at start, it works
        let text2 = "% !TeX root = visible.tex\n".to_string() + &text;
        let res2 = scan_file(&text2);
        assert_eq!(res2.7, Some("visible.tex".to_string()));
    }

    #[test]
    fn test_query_symbols_bibtex() {
        let workspace = Workspace::new();
        let uri = Url::parse("file:///refs.bib").unwrap();
        workspace.update_bib(&uri, "@article{knuth, title={The Art}}");

        let symbols = workspace.query_symbols("knuth");
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].0, "knuth");
        assert_eq!(symbols[0].1, SymbolKind::CLASS);
    }

    #[test]
    fn test_workspace_citations_unreferenced() {
        // Test get_all_citation_keys when there are NO referenced bibs
        // This hits the "referenced_bibs.is_empty()" branch in get_all_citation_keys
        let workspace = Workspace::new();
        // Add a bib file but DO NOT reference it in any tex file
        let bib_uri = Url::parse("file:///orphan.bib").unwrap();
        workspace.update_bib(&bib_uri, "@article{orphan_key, title={Orphan}}");

        // Should still find the key by scanning all known bibs
        let keys = workspace.get_all_citation_keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "orphan_key");

        // Also test has_citation_key fallback
        assert!(workspace.has_citation_key("orphan_key"));
    }

    #[test]
    fn test_scan_file_malformed_braces() {
        // Test scanner resilience with weird brace patterns
        // 1. Closing brace without opening
        let text1 = "some text } more text";
        let res1 = scan_file(text1);
        assert!(res1.0.is_empty());

        // 2. Opening brace without command
        let text2 = "{ pure group }";
        let res2 = scan_file(text2);
        assert!(res2.0.is_empty());

        // 3. Command with detached malformed arg
        // \cmd } {
        let text3 = "\\cmd } {";
        _ = scan_file(text3); // Should not panic

        // 4. Broken package scan
        let text4 = "\\usepackage{incomplete";
        let res4 = scan_file(text4);
        assert!(res4.6.is_empty());
    }
}
