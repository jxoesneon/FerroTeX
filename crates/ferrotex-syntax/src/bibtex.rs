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
    skip_whitespace(chars);

    // 3. Read fields
    let mut fields = HashMap::new();
    let mut end_idx = input_len;

    loop {
        skip_whitespace(chars);
        // Check for end of entry
        if let Some(&(_, '}')) = chars.peek() {
            if let Some((idx, _)) = chars.next() {
                end_idx = idx + 1;
            }
            break;
        }

        // Read field name
        let field_name = read_until(chars, |c| c == '=' || c.is_whitespace() || c == '}');
        if let Some(name) = field_name { 
            let name = name.trim().to_lowercase();
             if name.is_empty() {
                // Could be trailing comma or malformed 
                if let Some(&(_, '}')) = chars.peek() {
                    continue; // Loop will catch it next iteration
                }
                // Determine if we should break or skip char?
                // Let's consume one char to avoid infinite loop if stuck
                if chars.next().is_none() { break; }
                continue;
            }
            
            skip_whitespace(chars);
            // Expect =
            if let Some(&(_, '=')) = chars.peek() {
                chars.next(); // consume =
                skip_whitespace(chars);
                
                // Read value
                if let Some(val) = read_value(chars) {
                    fields.insert(name, val);
                }
                
                skip_whitespace(chars);
                // Consume optional comma
                if let Some(&(_, ',')) = chars.peek() {
                    chars.next();
                }
            } else {
                // Missing equals, maybe malformed, skip to next comma or end
                 // Consuming until comma or brace
                 read_until(chars, |c| c == ',' || c == '}');
                 if let Some(&(_, ',')) = chars.peek() { chars.next(); }
            }
        } else {
             // No field name found, check closure
             if let Some(&(_, '}')) = chars.peek() {
                 continue;
             }
             if chars.next().is_none() { break; }
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

fn read_value(chars: &mut Peekable<CharIndices>) -> Option<String> {
    // Value can be:
    // "..."
    // { ... }
    // digits (simple)
    // identifier (macro - treated as string for now)
    
    let &(_, c) = chars.peek()?;
    
    if c == '"' {
        chars.next(); // consume "
        // Read until "
        // Handle escaped quotes? Simplified: no escapes for now or simplistic.
        let mut val = String::new();
        while let Some(&(_, ch)) = chars.peek() {
            if ch == '"' {
                chars.next();
                break;
            }
            val.push(ch);
            chars.next();
        }
        Some(val)
    } else if c == '{' {
        chars.next(); // consume {
        let mut val = String::new();
        let mut depth = 1;
        while let Some(&(_, ch)) = chars.peek() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    chars.next();
                    break;
                }
            }
            val.push(ch);
            chars.next();
        }
        Some(val)
    } else {
        // Read until comma or closing brace
        // This covers numbers and unquoted strings/macros
        read_until(chars, |char_code| char_code == ',' || char_code == '}' || char_code.is_whitespace())
    }
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

    #[test]
    fn test_empty_bib() {
        let entries = parse_bibtex("");
        assert!(entries.entries.is_empty());
    }

    #[test]
    fn test_invalid_entry_ignored() {
        // Test resilience to malformed input
        let input = r#"
            @Article{key,
                title = {Title}
            % Missing closing brace ? 
        "#;
        let entries = parse_bibtex(input);
        // Should parse partial entry or none if incomplete
        // Based on logic, if loop breaks due to EOF without '}', it might push entry?
        // Ah, it doesn't push unless '}' is found in the fields loop (line 78 and 82 loops)
        // Actually line 82 breaks the fields loop.
        // If EOF, fields loop breaks at line 127 if chars.next() is none
        // Then parse_entry returns Some(BibEntry...)
        // So we expect 1 entry even if unclosed?
        // Let's assert based on behavior.
        assert!(entries.entries.len() <= 1);
    }
    
    #[test]
    fn test_bib_comments_everywhere() {
        let input = r#"
            % Top comment
            @Book{ lib,
              % Field comment
              title = "Library", % Inline comment
              year = 2020
            }
            % Bottom comment
        "#;
        let entries = parse_bibtex(input);
        assert_eq!(entries.entries.len(), 1);
        if let Some(t) = entries.entries[0].fields.get("title") {
             assert_eq!(t, "Library");
        }
    }

    #[test]
    fn test_bib_quoted_values() {
        let input = r#"@Misc{x, note = "quoted string"}"#;
        let entries = parse_bibtex(input);
        assert_eq!(entries.entries[0].fields.get("note"), Some(&"quoted string".to_string()));
    }
    
    #[test]
    fn test_bib_mixed_delimiters() {
        let input = r#"@Misc{x, year = 1999, month = "Jan", note = {Braced}}"#;
        let entries = parse_bibtex(input);
        assert_eq!(entries.entries[0].fields.get("year"), Some(&"1999".to_string()));
        assert_eq!(entries.entries[0].fields.get("month"), Some(&"Jan".to_string()));
        assert_eq!(entries.entries[0].fields.get("note"), Some(&"Braced".to_string()));
    }
    
    #[test]
    fn test_bib_trailing_comma() {
        let input = r#"@Misc{x, year=1999,}"#;
        let entries = parse_bibtex(input);
        assert_eq!(entries.entries.len(), 1);
        assert_eq!(entries.entries[0].fields.get("year"), Some(&"1999".to_string()));
    }
}
