use ferrotex_package::PackageIndex;
use std::collections::HashMap;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

/// Represents the completion data available for a specific LaTeX package.
#[derive(Debug, Clone)]
pub struct PackageCompletion {
    /// List of commands provided by the package (without backslash).
    pub commands: Vec<String>,
    /// List of environments provided by the package.
    pub environments: Vec<String>,
}

lazy_static::lazy_static! {
    static ref PACKAGE_DATA: HashMap<&'static str, PackageCompletion> = {
        let mut m = HashMap::new();
        // Amsmath
        m.insert("amsmath", PackageCompletion {
            commands: vec![
                "text".into(), "tag".into(), "eqref".into(), "numberwithin".into(),
                "dddot".into(), "ddddot".into(), "boldsymbol".into(),
            ],
            environments: vec![
                "align".into(), "align*".into(),
                "gather".into(), "gather*".into(),
                "flalign".into(), "flalign*".into(),
                "alignat".into(), "alignat*".into(),
                "split".into(), "cases".into(), "matrix".into(), "pmatrix".into(), "bmatrix".into(),
            ],
        });
        // TikZ (Basic)
        m.insert("tikz", PackageCompletion {
            commands: vec![
                "draw".into(), "node".into(), "coordinate".into(), "fill".into(),
                "clip".into(), "path".into(), "usetikzlibrary".into(),
            ],
            environments: vec![
                "tikzpicture".into(), "scope".into(),
            ],
        });
        // Geometry
        m.insert("geometry", PackageCompletion {
            commands: vec!["geometry".into(), "newgeometry".into(), "restoregeometry".into()],
            environments: vec![],
        });
        // Hyperref
        m.insert("hyperref", PackageCompletion {
            commands: vec![
                "href".into(), "url".into(), "hypersetup".into(), "autorek".into(),
            ],
            environments: vec![],
        });
        // Graphicx
        m.insert("graphicx", PackageCompletion {
            commands: vec![
                "includegraphics".into(), "graphicspath".into(), "rotatebox".into(), "scalebox".into()
            ],
            environments: vec![],
        });
        m
    };
}

/// Returns a tuple of (commands, environments) completion items for the given list of packages.
///
/// This function aggregates data from:
/// 1. Static built-in package data (well-known packages).
/// 2. Dynamic package index (scanned from disk).
pub fn get_package_completions(
    packages: &[String],
    index: Option<&PackageIndex>,
    workspace: Option<&crate::workspace::Workspace>,
) -> (Vec<CompletionItem>, Vec<CompletionItem>) {
    let mut cmd_items = Vec::new();
    let mut env_items = Vec::new();
    let mut seen_cmds = std::collections::HashSet::new();
    let mut seen_envs = std::collections::HashSet::new();

    for pkg in packages {
        // 1. Try static data first
        if let Some(data) = PACKAGE_DATA.get(pkg.as_str()) {
            add_items(
                &mut cmd_items,
                &mut env_items,
                &mut seen_cmds,
                &mut seen_envs,
                pkg,
                &data.commands,
                &data.environments,
            );
        }
        // 2. Try dynamic index
        else if let Some(idx) = index {
            if let Some(data) = idx.packages.get(pkg) {
                add_items(
                    &mut cmd_items,
                    &mut env_items,
                    &mut seen_cmds,
                    &mut seen_envs,
                    pkg,
                    &data.commands,
                    &data.environments,
                );
            }
        }
    }

    // 3. Add user-defined macros from workspace
    if let Some(ws) = workspace {
        for entry in ws.indices.iter() {
            for macro_def in &entry.value().macros {
                if seen_cmds.insert(macro_def.name.trim_start_matches('\\').to_string()) {
                    cmd_items.push(CompletionItem {
                        label: macro_def.name.clone(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some(format!("User Macro (from {})", entry.key())),
                        ..Default::default()
                    });
                }
            }
        }
    }

    (cmd_items, env_items)
}

fn add_items(
    cmd_items: &mut Vec<CompletionItem>,
    env_items: &mut Vec<CompletionItem>,
    seen_cmds: &mut std::collections::HashSet<String>,
    seen_envs: &mut std::collections::HashSet<String>,
    pkg: &str,
    commands: &[String],
    environments: &[String],
) {
    for cmd in commands {
        if seen_cmds.insert(cmd.clone()) {
            cmd_items.push(CompletionItem {
                label: format!("\\{}", cmd),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!("Package: {}", pkg)),
                ..Default::default()
            });
        }
    }
    for env in environments {
        if seen_envs.insert(env.clone()) {
            env_items.push(CompletionItem {
                label: env.to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some(format!("Package: {}", pkg)),
                ..Default::default()
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_package_completions_static() {
        let packages = vec!["amsmath".to_string()];
        let (cmds, envs) = get_package_completions(&packages, None, None);

        assert!(!cmds.is_empty(), "amsmath should have commands");
        assert!(!envs.is_empty(), "amsmath should have environments");

        // Check specific command
        assert!(
            cmds.iter().any(|c| c.label == "\\text"),
            "amsmath should have \\text"
        );
        // Check specific environment
        assert!(
            envs.iter().any(|e| e.label == "align"),
            "amsmath should have align env"
        );
    }

    #[test]
    fn test_get_package_completions_unknown() {
        let packages = vec!["nonexistent-pkg".to_string()];
        let (cmds, envs) = get_package_completions(&packages, None, None);

        assert!(cmds.is_empty(), "unknown package should have no commands");
        assert!(
            envs.is_empty(),
            "unknown package should have no environments"
        );
    }

    #[test]
    fn test_get_package_completions_dynamic() {
        use ferrotex_package::{PackageIndex, PackageMetadata};

        let mut index = PackageIndex::new();
        index.insert(
            "mypkg".to_string(),
            PackageMetadata {
                commands: vec!["mycmd".to_string()],
                environments: vec!["myenv".to_string()],
            },
        );

        let packages = vec!["mypkg".to_string()];
        let (cmds, envs) = get_package_completions(&packages, Some(&index), None);

        assert!(
            cmds.iter().any(|c| c.label == "\\mycmd"),
            "dynamic pkg should have \\mycmd"
        );
        assert!(
            envs.iter().any(|e| e.label == "myenv"),
            "dynamic pkg should have myenv"
        );
    }

    #[test]
    fn test_get_package_completions_deduplication() {
        let packages = vec!["amsmath".to_string(), "amsmath".to_string()];
        let (cmds, _) = get_package_completions(&packages, None, None);

        // Count \text commands
        let text_count = cmds.iter().filter(|c| c.label == "\\text").count();
        // Since we now deduplicate, this should be exactly 1
        assert_eq!(text_count, 1, "Should deduplicate commands");
    }

    #[test]
    fn test_macro_completion() {
        use crate::workspace::Workspace;
        use tower_lsp::lsp_types::Url;

        let workspace = Workspace::new();
        let uri = Url::parse("file:///test.tex").unwrap();
        // We simulate a file with a macro definition
        workspace.update(&uri, r"\newcommand{\mycustomcmd}{Hello}");

        let packages = vec![];
        let (cmds, _) = get_package_completions(&packages, None, Some(&workspace));

        assert!(
            cmds.iter().any(|c| c.label == "\\mycustomcmd"),
            "Should find user-defined macro \\mycustomcmd"
        );
    }
}
