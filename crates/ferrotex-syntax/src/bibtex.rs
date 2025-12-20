use rowan::{TextRange, TextSize};
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::CharIndices;

/// Represents a single BibTeX entry (e.g., `@article{...}`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibEntry {
    /// The type of the entry (e.g., "article", "book").
    pub entry_type: String,
    /// The citation key (e.g., "knuth1984").
    pub key: String,
    /// The fields of the entry (e.g., "author" -> "Knuth", "title" -> "The TeXbook").
    pub fields: HashMap<String, String>,
    /// The full range of the entry in the source file.
    pub range: TextRange,
}

/// Represents a parsed BibTeX file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibFile {
    /// The list of entries found in the file.
    pub entries: Vec<BibEntry>,
}

/// A very simple, best-effort BibTeX parser.
pub fn parse_bibtex(input: &str) -> BibFile {
    let mut entries = Vec::new();
    let mut chars = input.char_indices().peekable();

    loop {
        let Some((start_idx, c)) = chars.next() else {
            break;
        };
        if c == '@'
            && let Some(entry) = parse_entry(&mut chars, start_idx, input.len())
        {
            entries.push(entry);
        }
    }

    BibFile { entries }
}

fn parse_entry(
    chars: &mut Peekable<CharIndices>,
    start_idx: usize,
    input_len: usize,
) -> Option<BibEntry> {
    // 1. Read entry type (e.g., article)
    let entry_type = read_until(chars, |c| c == '{' || c.is_whitespace())?;
    skip_whitespace(chars);

    if let Some((_, '{')) = chars.next() {
        // Ok
    } else {
        return None;
    }
    skip_whitespace(chars);

    // 2. Read key
    let key = read_until(chars, |c| c == ',' || c.is_whitespace())?;
    skip_whitespace(chars);

    // Expect comma
    if let Some(&(_, ',')) = chars.peek() {
        chars.next();
    }

    // 3. Read fields (simplified: skip until closing brace of entry)
    let fields = HashMap::new();
    let mut brace_depth = 1;
    let end_idx;

    loop {
        let Some((idx, c)) = chars.next() else {
            // End of file inside entry? Use input_len as end
            end_idx = input_len;
            break;
        };
        match c {
            '{' => brace_depth += 1,
            '}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    end_idx = idx + 1; // Include the closing brace
                    break;
                }
            }
            _ => {}
        }
    }

    Some(BibEntry {
        entry_type: entry_type.to_lowercase(),
        key,
        fields,
        range: TextRange::new(
            TextSize::from(start_idx as u32),
            TextSize::from(end_idx as u32),
        ),
    })
}

fn read_until<F>(chars: &mut Peekable<CharIndices>, predicate: F) -> Option<String>
where
    F: Fn(char) -> bool,
{
    let mut s = String::new();
    while let Some(&(_, c)) = chars.peek() {
        if predicate(c) {
            break;
        }
        s.push(c);
        chars.next();
    }
    if s.is_empty() { None } else { Some(s) }
}

fn skip_whitespace(chars: &mut Peekable<CharIndices>) {
    while let Some(&(_, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_bib() {
        let input = r#"
@article{key1,
    author = "Author One",
    title = {Title One}
}

@book{key2,
    author = "Author Two"
}
"#;
        let bib = parse_bibtex(input);
        assert_eq!(bib.entries.len(), 2);
        assert_eq!(bib.entries[0].key, "key1");
        assert_eq!(bib.entries[0].entry_type, "article");
        assert_eq!(bib.entries[1].key, "key2");
        assert_eq!(bib.entries[1].entry_type, "book");
    }

    #[test]
    fn test_messy_bib() {
        let input = r#"
@misc{ key3 , field = {val} }
@COMMENT{ ignored }
"#;
        let bib = parse_bibtex(input);
        assert_eq!(bib.entries.len(), 2);
        assert_eq!(bib.entries[0].key, "key3");
    }
}
