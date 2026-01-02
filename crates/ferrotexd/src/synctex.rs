use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of a SyncTeX Forward Search (Source -> PDF).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForwardSearchResult {
    pub page: u32,
    pub x: f64,
    pub y: f64,
}

/// Result of a SyncTeX Inverse Search (PDF -> Source).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InverseSearchResult {
    pub file: String,
    pub line: u32,
}

pub struct SyncTexIndex {
    pub version: String,
    pub files: HashMap<u32, PathBuf>,
    pub boxes: Vec<SyncTexBox>,
}

#[derive(Debug, Clone)]
pub struct SyncTexBox {
    pub tag: u32,
    pub line: u32,
    pub page: u32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl SyncTexIndex {
    pub fn load(pdf_path: &Path) -> Option<Self> {
        let mut synctex_path = pdf_path.to_path_buf();
        synctex_path.set_extension("synctex.gz");
        
        if !synctex_path.exists() {
            synctex_path = pdf_path.with_extension("synctex");
            if !synctex_path.exists() {
                return None;
            }
        }

        let file = File::open(&synctex_path).ok()?;
        let mut reader: Box<dyn BufRead> = if synctex_path.extension().map_or(false, |e| e == "gz") {
            Box::new(BufReader::new(GzDecoder::new(file)))
        } else {
            Box::new(BufReader::new(file))
        };

        let mut files = HashMap::new();
        let mut boxes = Vec::new();
        let mut current_page = 0;
        let mut version = String::new();

        let mut line_buf = String::new();
        while reader.read_line(&mut line_buf).unwrap_or(0) > 0 {
            let line = line_buf.trim();
            if line.is_empty() {
                line_buf.clear();
                continue;
            }

            match line.chars().next() {
                Some('S') if line.starts_with("SyncTeX Version:") => {
                    version = line["SyncTeX Version:".len()..].trim().to_string();
                }
                Some('I') if line.starts_with("Input:") => {
                    let parts: Vec<&str> = line["Input:".len()..].splitn(2, ':').collect();
                    if parts.len() == 2 {
                        if let Ok(tag) = parts[0].parse::<u32>() {
                            files.insert(tag, PathBuf::from(parts[1]));
                        }
                    }
                }
                Some('{') => {
                    current_page = line[1..].parse().unwrap_or(0);
                }
                Some('[') | Some('(') | Some('v') | Some('h') | Some('x') | Some('g') => {
                    // Record format: char<tag>,<line>,<col>:<x>,<y>,<w>,<h>,<v>
                    // We only care about tag, line, x, y, w, h
                    let content = &line[1..];
                    let parts: Vec<&str> = content.split(':').collect();
                    if parts.len() >= 2 {
                        let left: Vec<&str> = parts[0].split(',').collect();
                        let right: Vec<&str> = parts[1].split(',').collect();
                        
                        if left.len() >= 2 && right.len() >= 4 {
                            let tag = left[0].parse().unwrap_or(0);
                            let line_num = left[1].parse().unwrap_or(0);
                            let x = right[0].parse().unwrap_or(0.0);
                            let y = right[1].parse().unwrap_or(0.0);
                            let w = right[2].parse().unwrap_or(0.0);
                            let h = right[3].parse().unwrap_or(0.0);
                            
                            boxes.push(SyncTexBox {
                                tag,
                                line: line_num,
                                page: current_page,
                                x,
                                y,
                                width: w,
                                height: h,
                            });
                        }
                    }
                }
                _ => {}
            }
            line_buf.clear();
        }

        Some(SyncTexIndex { version, files, boxes })
    }

    pub fn forward_search(&self, tex_path: &Path, line: u32) -> Option<ForwardSearchResult> {
        let tag = self.files.iter()
            .find(|(_, p)| p.ends_with(tex_path) || tex_path.ends_with(p))
            .map(|(t, _)| *t)?;

        // Find the first box that matches the tag and line
        // Typically we want the one closest to the line
        self.boxes.iter()
            .find(|b| b.tag == tag && b.line >= line + 1)
            .map(|b| ForwardSearchResult {
                page: b.page,
                // Use standard 72bpm units (PDF.js default)
                x: b.x / 65536.0,
                y: b.y / 65536.0,
            })
    }

    pub fn inverse_search(&self, page: u32, x: f64, y: f64) -> Option<InverseSearchResult> {
        // x and y are in pts (usually) from the viewer.
        // SyncTeX boxes are in 65536 units.
        let target_x = x * 65536.0;
        let target_y = y * 65536.0;
        
        // Find the box that contains (x, y) and is on the right page
        // We look for the smallest box or the most specific node
        let mut best_match: Option<&SyncTexBox> = None;
        let mut min_area = f64::MAX;

        for b in &self.boxes {
            if b.page == page {
                if target_x >= b.x && target_x <= b.x + b.width 
                   && target_y >= b.y - b.height && target_y <= b.y  // y is typically top-down or bottom-up
                {
                    let area = b.width * b.height;
                    if area < min_area {
                        min_area = area;
                        best_match = Some(b);
                    }
                }
            }
        }

        let b = best_match?;
        let file_path = self.files.get(&b.tag)?;
        
        Some(InverseSearchResult {
            file: file_path.to_string_lossy().to_string(),
            line: b.line.saturating_sub(1),
        })
    }
}

pub fn forward_search(tex_path: &Path, pdf_path: &Path, line: u32, _col: u32) -> Option<ForwardSearchResult> {
    let index = SyncTexIndex::load(pdf_path)?;
    index.forward_search(tex_path, line)
}

pub fn inverse_search(pdf_path: &Path, page: u32, x: f64, y: f64) -> Option<InverseSearchResult> {
    let index = SyncTexIndex::load(pdf_path)?;
    index.inverse_search(page, x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_synctex_search_logic() {
        let mut files = HashMap::new();
        let tex_path = PathBuf::from("main.tex");
        files.insert(1, tex_path.clone());

        let boxes = vec![
            SyncTexBox {
                tag: 1,
                line: 10,
                page: 1,
                x: 100.0 * 65536.0,
                y: 200.0 * 65536.0,
                width: 50.0 * 65536.0,
                height: 10.0 * 65536.0,
            },
        ];

        let index = SyncTexIndex {
            version: "1".to_string(),
            files,
            boxes,
        };

        // Test Forward Search
        let forward = index.forward_search(&PathBuf::from("main.tex"), 9).unwrap();
        assert_eq!(forward.page, 1);
        assert_eq!(forward.x, 100.0);
        assert_eq!(forward.y, 200.0);

        // Test Inverse Search
        let inverse = index.inverse_search(1, 125.0, 195.0).unwrap();
        assert_eq!(inverse.line, 9);
        assert!(inverse.file.contains("main.tex"));

        // Test Boundary Inverse Search (out of box)
        assert!(index.inverse_search(1, 200.0, 300.0).is_none());
    }

    #[test]
    fn test_load_mock_file() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let pdf_path = dir.path().join("test.pdf");
        let synctex_path = dir.path().join("test.synctex");
        
        let mut file = File::create(&synctex_path)?;
        writeln!(file, "SyncTeX Version:1")?;
        writeln!(file, "Input:1:main.tex")?;
        writeln!(file, "{{1")?;
        writeln!(file, "[1,10,1:100,200,50,10,5")?;
        writeln!(file, "}}1")?;
        drop(file); // Ensure file is closed for Windows
        
        // Load (extension-less path)
        let index = SyncTexIndex::load(&pdf_path).expect("Should load .synctex");
        assert_eq!(index.version, "1");
        assert_eq!(index.files.get(&1).unwrap().to_str().unwrap(), "main.tex");
        assert_eq!(index.boxes.len(), 1);
        assert_eq!(index.boxes[0].line, 10);
        
        Ok(())
    }
}
