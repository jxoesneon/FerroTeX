use std::collections::HashMap;

/// Represents a single BibTeX entry (e.g., `@article{...}`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibEntry {
    /// The type of the entry (e.g., "article", "book").
    pub entry_type: String,
    /// The citation key (e.g., "knuth1984").
    pub key: String,
    /// The fields of the entry (e.g., "author" -> "Knuth", "title" -> "The TeXbook").
    pub fields: HashMap<String, String>,
}

/// Represents a parsed BibTeX file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibFile {
    /// The list of entries found in the file.
    pub entries: Vec<BibEntry>,
}

/// A very simple, best-effort BibTeX parser.
///
/// It doesn't use the full Lexer/Parser infrastructure of LaTeX because
/// BibTeX syntax is quite different and simpler in structure (though complex in details).
/// For v0.8.0, we prioritize extracting keys for completion.
///
/// # Arguments
///
/// * `input` - The BibTeX source code as a string.
///
/// # Returns
///
/// A `BibFile` struct containing all parsed entries.
pub fn parse_bibtex(input: &str) -> BibFile {
    let mut entries = Vec::new();
    let mut chars = input.chars().peekable();

    loop {
        let Some(c) = chars.next() else {
            break;
        };
        if c == '@'
            && let Some(entry) = parse_entry(&mut chars)
        {
            entries.push(entry);
        }
    }

    BibFile { entries }
}

fn parse_entry(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<BibEntry> {
    // 1. Read entry type (e.g., article)
    let entry_type = read_until(chars, |c| c == '{' || c.is_whitespace())?;
    skip_whitespace(chars);

    if chars.next()? != '{' {
        return None;
    }
    skip_whitespace(chars);

    // 2. Read key
    let key = read_until(chars, |c| c == ',' || c.is_whitespace())?;
    skip_whitespace(chars);

    // Expect comma
    if chars.peek() == Some(&',') {
        chars.next();
    } else {
        // Might be just a key and nothing else? rare but possible
    }

    // 3. Read fields (simplified: skip until closing brace of entry)
    // For v0.8.0 we mostly care about keys, but reading fields is good for future.
    // For now, let's just skip to the matching closing brace to advance properly.
    // A proper parser would parse "field = value".

    let fields = HashMap::new();
    let mut brace_depth = 1;

    loop {
        let Some(c) = chars.next() else {
            break;
        };
        match c {
            '{' => brace_depth += 1,
            '}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
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
    })
}

fn read_until<F>(chars: &mut std::iter::Peekable<std::str::Chars>, predicate: F) -> Option<String>
where
    F: Fn(char) -> bool,
{
    let mut s = String::new();
    while let Some(&c) = chars.peek() {
        if predicate(c) {
            break;
        }
        s.push(c);
        chars.next();
    }
    if s.is_empty() { None } else { Some(s) }
}

fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
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
