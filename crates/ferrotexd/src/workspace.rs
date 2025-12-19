use dashmap::DashMap;
use ferrotex_syntax::{parse, SyntaxKind};
use std::collections::{HashMap, HashSet};
use tower_lsp::lsp_types::Url;

#[derive(Debug, Default)]
pub struct Workspace {
    /// Map from Document URI to list of raw included paths found in the document
    includes: DashMap<Url, Vec<IncludeRef>>,
}

#[derive(Debug, Clone)]
pub struct IncludeRef {
    pub path: String,
    pub range: ferrotex_syntax::TextRange,
}

impl Workspace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&self, uri: &Url, text: &str) {
        let includes = scan_includes(text);
        self.includes.insert(uri.clone(), includes);
    }

    pub fn get_includes(&self, uri: &Url) -> Vec<IncludeRef> {
        self.includes
            .get(uri)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn detect_cycles(&self) -> Vec<(Url, ferrotex_syntax::TextRange, String)> {
        let mut cycles = Vec::new();
        // Snapshot of the graph to avoid locking issues during traversal
        // Map: Url -> Vec<(ResolvedUrl, Range, PathString)>
        let mut graph: HashMap<Url, Vec<(Url, ferrotex_syntax::TextRange, String)>> =
            HashMap::new();

        for entry in self.includes.iter() {
            let base_uri = entry.key();
            let refs = entry.value();
            let mut edges = Vec::new();
            for r in refs {
                // Best-effort resolution
                // We assume paths are relative to the document location
                // This mirrors the logic in document_link
                if let Ok(target) = base_uri.join(&r.path) {
                    edges.push((target, r.range, r.path.clone()));
                }
            }
            graph.insert(base_uri.clone(), edges);
        }

        let nodes: Vec<Url> = graph.keys().cloned().collect();

        // Run DFS from *each* node to find all back-edges.
        // This is O(V*(V+E)), which is acceptable for typical workspace sizes.
        for node in &nodes {
            let mut visited = HashSet::new();
            self.check_cycle_dfs(node, &graph, &mut visited, &mut Vec::new(), &mut cycles);
        }

        // Deduplicate cycles
        // We can't use HashSet directly because TextRange doesn't implement Hash (it comes from rowan)
        // So we sort and dedup manually or just check before pushing?
        // Checking before pushing is O(N^2) in number of cycles. Cycles are rare.
        // But TextRange implements Eq.

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

    fn check_cycle_dfs(
        &self,
        current: &Url,
        graph: &HashMap<Url, Vec<(Url, ferrotex_syntax::TextRange, String)>>,
        visited: &mut HashSet<Url>,
        path_stack: &mut Vec<Url>, // Gray nodes
        cycles: &mut Vec<(Url, ferrotex_syntax::TextRange, String)>,
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

fn scan_includes(text: &str) -> Vec<IncludeRef> {
    let parse = parse(text);
    let root = parse.syntax();
    let mut refs = Vec::new();

    for node in root.descendants() {
        if node.kind() == SyntaxKind::Include {
            // Structure: Include -> Group -> { Text }
            // We want to extract the text inside the group.
            if let Some(group) = node.children().find(|n| n.kind() == SyntaxKind::Group) {
                // The group contains text tokens and whitespace. We should concatenate them or find the main text.
                // For simplicity, let's just grab the text of the group excluding the braces.
                // Or better, iterate over children of the group (which are tokens/nodes).

                let mut path = String::new();
                for child in group.children_with_tokens() {
                    match child.kind() {
                        SyntaxKind::Text | SyntaxKind::Whitespace => {
                            path.push_str(&child.to_string());
                        }
                        _ => {}
                    }
                }

                // Trim braces if they are included in the children loop?
                // Wait, `Group` children usually are the content *inside* the braces if the parser handles it that way?
                // Let's check the parser.rs implementation of `parse_group`.
                // It does `builder.start_node(Group)`, bumps `{`, loops, bumps `}`, finishes.
                // So the `{` and `}` are children tokens of the Group node.

                let raw_text = path;
                // Remove leading { and trailing }
                let trimmed = raw_text.trim();
                let clean_path = if trimmed.starts_with('{') && trimmed.ends_with('}') {
                    &trimmed[1..trimmed.len() - 1]
                } else {
                    trimmed
                };

                refs.push(IncludeRef {
                    path: clean_path.trim().to_string(),
                    range: node.text_range(),
                });
            }
        }
    }
    refs
}
