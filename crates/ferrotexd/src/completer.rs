use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};
use std::collections::HashMap;

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
/// This function aggregates static data for well-known packages like `amsmath`, `tikz`, etc.
pub fn get_package_completions(packages: &[String]) -> (Vec<CompletionItem>, Vec<CompletionItem>) {
    let mut cmd_items = Vec::new();
    let mut env_items = Vec::new();

    for pkg in packages {
        if let Some(data) = PACKAGE_DATA.get(pkg.as_str()) {
            for cmd in &data.commands {
                cmd_items.push(CompletionItem {
                    label: format!("\\{}", cmd),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some(format!("Package: {}", pkg)),
                    ..Default::default()
                });
            }
            for env in &data.environments {
                env_items.push(CompletionItem {
                    label: env.to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    detail: Some(format!("Package: {}", pkg)),
                    ..Default::default()
                });
            }
        }
    }

    (cmd_items, env_items)
}
