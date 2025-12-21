use dashmap::DashMap;
use ferrotex_syntax::{SyntaxKind, TextRange, parse};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use tower_lsp::lsp_types::{SymbolKind, Url};

/// The central workspace manager for the LSP server.
///
/// It maintains an in-memory index of all tracked TeX and BibTeX files.
#[derive(Debug, Default)]
pub struct Workspace {
    /// Per-file index containing includes, definitions, citations, etc.
    indices: DashMap<Url, FileIndex>,
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
    /// Parses the file content and extracts includes, labels, citations, etc.
    pub fn update(&self, uri: &Url, text: &str) {
        let (includes, definitions, references, citations, bibliographies, sections, packages, magic_root) =
            scan_file(text);

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
                if let Some(bib_file) = self.bib_indices.get(&uri)
                    && bib_file.entries.iter().any(|e| e.key == key)
                {
                    return true;
                }
            }
        }

        false
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
    Vec<String>, // packages
    Option<String>, // magic_root
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
    let re = Regex::new(r"(?m)^%\s*!TEX\s+root\s*=\s*(.+)$").unwrap();
    let magic_root = re.captures(head).map(|cap| cap[1].trim().to_string());

    let parse = parse(text);
    let root = parse.syntax();
    let mut includes = Vec::new();
    let mut defs = Vec::new();
    let mut refs = Vec::new();
    let mut citations = Vec::new();
    let mut bibs = Vec::new();
    let mut sections = Vec::new();

    for node in root.descendants() {
        match node.kind() {
            SyntaxKind::Include => {
                if let Some((name, range)) = extract_label_data(&node) {
                    includes.push(IncludeRef {
                        path: name,
                        range, // Use inner range for includes too
                    });
                }
            }
            SyntaxKind::LabelDefinition => {
                if let Some((name, range)) = extract_label_data(&node) {
                    defs.push(LabelDef {
                        name,
                        range, // Inner range
                    });
                }
            }
            SyntaxKind::LabelReference => {
                if let Some((name, range)) = extract_label_data(&node) {
                    refs.push(LabelRef {
                        name,
                        range, // Inner range
                    });
                }
            }
            SyntaxKind::Citation => {
                if let Some((keys, range)) = extract_label_data(&node) {
                    // Split keys by comma
                    for key in keys.split(',') {
                        let trimmed = key.trim();
                        if !trimmed.is_empty() {
                            citations.push(CitationRef {
                                key: trimmed.to_string(),
                                range, // Note: This uses the full range for now, we might want sub-ranges later
                            });
                        }
                    }
                }
            }
            SyntaxKind::Bibliography => {
                if let Some((paths, range)) = extract_label_data(&node) {
                    // Split paths by comma (usually bibliography takes comma-separated list)
                    for path in paths.split(',') {
                        let trimmed = path.trim();
                        if !trimmed.is_empty() {
                            bibs.push(BibRef {
                                path: trimmed.to_string(),
                                range,
                            });
                        }
                    }
                }
            }
            SyntaxKind::Section => {
                if let Some((name, range)) = extract_label_data(&node) {
                    sections.push(SectionDef { name, range });
                }
            }
            _ => {}
        }
    }
    // Scan for packages
    // Pattern: \usepackage[opt]{pkg} or \RequirePackage[opt]{pkg}
    // We ignore options for now.
    // Captures: 1=pkg
    let pkg_re = Regex::new(r"(?m)\\(?:usepackage|RequirePackage)(?:\[[^\]]*\])?\{([^}]+)\}").unwrap();
    let mut packages = Vec::new();
    for cap in pkg_re.captures_iter(text) {
        // Packages can be comma separated: \usepackage{tikz, amsmath}
        for pkg in cap[1].split(',') {
            packages.push(pkg.trim().to_string());
        }
    }

    (includes, defs, refs, citations, bibs, sections, packages, magic_root)
}

pub fn extract_group_text(node: &ferrotex_syntax::SyntaxNode) -> Option<String> {
    extract_label_data(node).map(|(name, _)| name)
}

pub fn extract_label_data(node: &ferrotex_syntax::SyntaxNode) -> Option<(String, TextRange)> {
    let group = node.children().find(|n| n.kind() == SyntaxKind::Group)?;
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
