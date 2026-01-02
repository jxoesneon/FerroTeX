use crate::{PackageIndex, PackageMetadata};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct PackageScanner {
    tex_root: Option<PathBuf>,
}

impl Default for PackageScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageScanner {
    pub fn new() -> Self {
        Self {
            tex_root: Self::find_tex_root(),
        }
    }

    /// Attempts to find the TeX distribution root.
    fn find_tex_root() -> Option<PathBuf> {
        // Simple Heuristics for now
        let candidates = [
            "/usr/local/texlive/2023/texmf-dist/tex/latex",
            "/usr/local/texlive/2024/texmf-dist/tex/latex",
            "/usr/share/texlive/texmf-dist/tex/latex",
        ];

        for path in candidates {
            let p = Path::new(path);
            if p.exists() {
                return Some(p.to_path_buf());
            }
        }
        
        // Fallback: try kpsewhich
        if let Ok(output) = std::process::Command::new("kpsewhich")
            .args(["-var-value", "TEXMFDIST"])
            .output()
        {
            if output.status.success() {
                let texmf = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let latex_path = PathBuf::from(&texmf).join("tex/latex");
                if latex_path.exists() {
                    return Some(latex_path);
                }
            }
        }
        
        None
    }

    pub fn scan(&self) -> PackageIndex {
        let mut index = PackageIndex::new();

        if let Some(root) = &self.tex_root {
            log::info!("Scanning packages in: {:?}", root);
            for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "sty" {
                            if let Some(stem) = entry.path().file_stem() {
                                let pkg_name = stem.to_string_lossy().to_string();
                                // Parse the file (read then parse)
                                if let Ok(content) = fs::read_to_string(entry.path()) {
                                     let metadata = self.parse_content(&content);
                                     index.insert(pkg_name, metadata);
                                }
                            }
                        }
                    }
                }
            }
        } else {
             log::warn!("TeX root not found. Skipping scan.");
        }

        index
    }

    fn parse_content(&self, content: &str) -> PackageMetadata {
        let mut metadata = PackageMetadata::default();

        // Very basic regex parsing
        // Captures \newcommand{\foo} or \newcommand*{\foo}
        let re_cmd = Regex::new(r"\\(?:re)?newcommand\*?\{?\\([a-zA-Z@]+)\}?").unwrap();
        // Captures \newenvironment{foo}
        let re_env = Regex::new(r"\\newenvironment\{([a-zA-Z*]+)\}").unwrap();

        for cap in re_cmd.captures_iter(content) {
            if let Some(cmd) = cap.get(1) {
                metadata.commands.push(cmd.as_str().to_string());
            }
        }

        for cap in re_env.captures_iter(content) {
             if let Some(env) = cap.get(1) {
                metadata.environments.push(env.as_str().to_string());
            }
        }

        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_content() {
        let scanner = PackageScanner::new();
        let content = r#"
            \newcommand{\foo}{bar}
            \renewcommand*{\baz}[1]{qux}
            \newenvironment{myenv}{start}{end}
            \newenvironment{starenv*}{start}{end}
        "#;
        
        let metadata = scanner.parse_content(content);
        
        assert!(metadata.commands.contains(&"foo".to_string()));
        assert!(metadata.commands.contains(&"baz".to_string()));
        assert!(metadata.environments.contains(&"myenv".to_string()));
        assert!(metadata.environments.contains(&"starenv*".to_string()));
    }
}
