use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Result of a SyncTeX Forward Search (Source -> PDF).
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ForwardSearchResult {
    pub page: u32,
    pub x: f64,
    pub y: f64,
}

/// Result of a SyncTeX Inverse Search (PDF -> Source).
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct InverseSearchResult {
    pub file: String,
    pub line: u32,
}

/// Runs `synctex view` to find the PDF location corresponding to a source location.
/// Note: synctex coordinates are in points (72 dpi).
pub fn forward_search(
    tex_path: &Path,
    pdf_path: &Path,
    line: u32,
    col: u32,
) -> Option<ForwardSearchResult> {
    let args = get_forward_search_args(tex_path, pdf_path, line, col);

    let output = Command::new("synctex").args(&args).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_forward_search_output(&stdout)
}

fn get_forward_search_args(tex_path: &Path, pdf_path: &Path, line: u32, col: u32) -> Vec<String> {
    let input_spec = format!("{}:{}:{}", line + 1, col + 1, tex_path.to_string_lossy());
    vec![
        "view".to_string(),
        "-i".to_string(),
        input_spec,
        "-o".to_string(),
        pdf_path.to_string_lossy().to_string(),
    ]
}

/// Parses the stdout from `synctex view` to extract forward definition.
pub fn parse_forward_search_output(stdout: &str) -> Option<ForwardSearchResult> {
    // output format:
    // This is SyncTeX...
    // Output:PDF:...
    // Page:1
    // x:123.456
    // y:789.012
    // ...

    let mut page = 0;
    let mut x = 0.0;
    let mut y = 0.0;

    for line in stdout.lines() {
        if let Some(p) = line.strip_prefix("Page:") {
            page = p.trim().parse().unwrap_or(0);
        } else if let Some(val) = line.strip_prefix("x:") {
            x = val.trim().parse().unwrap_or(0.0);
        } else if let Some(val) = line.strip_prefix("y:") {
            y = val.trim().parse().unwrap_or(0.0);
        }
    }

    if page > 0 {
        Some(ForwardSearchResult { page, x, y })
    } else {
        None
    }
}

/// Runs `synctex edit` to find the source location corresponding to a PDF location.
pub fn inverse_search(pdf_path: &Path, page: u32, x: f64, y: f64) -> Option<InverseSearchResult> {
    let args = get_inverse_search_args(pdf_path, page, x, y);
    let output = Command::new("synctex")
        .arg("edit")
        .arg(&args[1]) // -o
        .arg(&args[2]) // spec
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_inverse_search_output(&stdout)
}

fn get_inverse_search_args(pdf_path: &Path, page: u32, x: f64, y: f64) -> Vec<String> {
    // synctex edit -o "page:x:y:pdf_path"
    let input_spec = format!("{}:{}:{}:{}", page, x, y, pdf_path.to_string_lossy());
    vec!["edit".to_string(), "-o".to_string(), input_spec]
}

/// Parses the stdout from `synctex edit` to extract inverse search result.
pub fn parse_inverse_search_output(stdout: &str) -> Option<InverseSearchResult> {
    // format:
    // Line:10
    // Column:5
    // Input:/path/to/file.tex

    let mut file = String::new();
    let mut line_num = 0;

    for line in stdout.lines() {
        if let Some(l) = line.strip_prefix("Line:") {
            line_num = l.trim().parse().unwrap_or(0);
        } else if let Some(f) = line.strip_prefix("Input:") {
            file = f.trim().to_string();
        }
    }

    if !file.is_empty() && line_num > 0 {
        Some(InverseSearchResult {
            file,
            line: line_num - 1,
        }) // Convert back to 0-indexed
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_forward_search_args() {
        let tex = Path::new("main.tex");
        let pdf = Path::new("main.pdf");
        let args = get_forward_search_args(tex, pdf, 10, 5);
        assert_eq!(args[0], "view");
        assert_eq!(args[1], "-i");
        assert_eq!(args[2], "11:6:main.tex"); // 1-based args
        assert_eq!(args[3], "-o");
        assert_eq!(args[4], "main.pdf");
    }

    #[test]
    fn test_get_inverse_search_args() {
        let pdf = Path::new("doc.pdf");
        let args = get_inverse_search_args(pdf, 1, 100.0, 200.0);
        assert_eq!(args[0], "edit");
        assert_eq!(args[1], "-o");
        assert_eq!(args[2], "1:100:200:doc.pdf");
    }

    #[test]
    fn test_parse_forward_search() {
        let output = r#"This is SyncTeX 1.0
Output:PDF:main.pdf
Page:1
x:100.5
y:200.75
"#;
        let result = parse_forward_search_output(output).expect("Should parse valid output");
        assert_eq!(
            result,
            ForwardSearchResult {
                page: 1,
                x: 100.5,
                y: 200.75,
            }
        );
    }

    #[test]
    fn test_parse_forward_search_invalid() {
        let output = "Invalid output";
        assert!(parse_forward_search_output(output).is_none());
    }

    #[test]
    fn test_parse_inverse_search() {
        let output = r#"SyncTeX 1.0
Line:10
Column:5
Input:/path/to/main.tex
"#;
        let result = parse_inverse_search_output(output).expect("Should parse valid output");
        assert_eq!(
            result,
            InverseSearchResult {
                file: "/path/to/main.tex".to_string(),
                line: 9, // 1-based (10) -> 0-based (9)
            }
        );
    }

    #[test]
    fn test_parse_inverse_search_invalid() {
        let output = "Line:0\nInput:";
        assert!(parse_inverse_search_output(output).is_none());
    }

    #[test]
    fn test_forward_search_execution() {
        // This test exercises the function signature and argument preparation.
        // It is expected to return None if synctex is not installed or fails.
        let tex = Path::new("main.tex");
        let pdf = Path::new("main.pdf");
        let _ = forward_search(tex, pdf, 10, 5);
    }

    #[test]
    fn test_inverse_search_execution() {
        // This test exercises the function signature and argument preparation.
        // It is expected to return None if synctex is not installed or fails.
        let pdf = Path::new("doc.pdf");
        let _ = inverse_search(pdf, 1, 100.0, 200.0);
    }
}
