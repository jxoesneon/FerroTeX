use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

/// Result of a SyncTeX Forward Search (Source -> PDF).
#[derive(Debug, Serialize, Deserialize)]
pub struct ForwardSearchResult {
    pub page: u32,
    pub x: f64,
    pub y: f64,
}

/// Result of a SyncTeX Inverse Search (PDF -> Source).
#[derive(Debug, Serialize, Deserialize)]
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
    // synctex view -i "line:col:tex_path" -o "pdf_path"
    // output format:
    // This is SyncTeX...
    // Output:PDF:...
    // Page:1
    // x:123.456
    // y:789.012
    // ...
    
    let input_spec = format!("{}:{}:{}", line + 1, col + 1, tex_path.to_string_lossy());
    
    let output = Command::new("synctex")
        .arg("view")
        .arg("-i")
        .arg(&input_spec)
        .arg("-o")
        .arg(pdf_path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse output
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
pub fn inverse_search(
    pdf_path: &Path,
    page: u32,
    x: f64,
    y: f64,
) -> Option<InverseSearchResult> {
    // synctex edit -o "page:x:y:pdf_path"
    // format:
    // Line:10
    // Column:5
    // Input:/path/to/file.tex
    
    let input_spec = format!("{}:{}:{}:{}", page, x, y, pdf_path.to_string_lossy());

    let output = Command::new("synctex")
        .arg("edit")
        .arg("-o")
        .arg(&input_spec)
        .output()
        .ok()?;
        
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
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
        Some(InverseSearchResult { file, line: line_num - 1 }) // Convert back to 0-indexed
    } else {
        None
    }
}
