use ferrotex_syntax::{SyntaxElement, SyntaxKind, SyntaxNode, TextRange};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroDef {
    pub name: String,
    pub args: usize,
    pub has_optional: bool, // If true, first arg is optional []
    pub definition_range: TextRange,
}

/// Scans a syntax tree for user-defined macros: \newcommand, \renewcommand, \providecommand.
pub fn scan_macros(root: &SyntaxNode) -> Vec<MacroDef> {
    let mut macros = Vec::new();

    for element in root.descendants_with_tokens() {
        if element.kind() == SyntaxKind::Command {
            let text = element.to_string();
            // We only care about command definitions
            if matches!(
                text.as_str(),
                "\\newcommand" | "\\renewcommand" | "\\providecommand" | "\\DeclareRobustCommand"
            ) {
                if let Some(def) = parse_macro_definition(&element) {
                    macros.push(def);
                }
            }
        }
    }

    macros
}

fn parse_macro_definition(node: &SyntaxElement) -> Option<MacroDef> {
    // Structure: \newcommand{\name}[argc][opt]{def}
    // We expect:
    // 1. Group {\name} OR just \name token (if not grouped)
    // 2. Optional: Group [argc]
    // 3. Optional: Group [opt]
    // 4. Group {def}

    let mut current = node.clone();

    // Helper to skip whitespace/comments
    let mut next_element = || loop {
        if let Some(next) = current.next_sibling_or_token() {
            current = next.clone();
            let kind = current.kind();
            if kind != SyntaxKind::Whitespace && kind != SyntaxKind::Comment {
                return Some(current.clone());
            }
        } else {
            return None;
        }
    };

    // 1. Extract Name
    let name_elem = next_element()?;

    let name = if name_elem.kind() == SyntaxKind::Group {
        // Extract \name from {\name}
        let text = name_elem.to_string(); // e.g., {\foo}
                                          // Remove braces
        let inner = text.trim_matches(|c| c == '{' || c == '}').trim();
        if inner.starts_with('\\') {
            inner.to_string()
        } else {
            return None; // Invalid
        }
    } else if name_elem.kind() == SyntaxKind::Command {
        // Just \name
        name_elem.to_string()
    } else {
        return None;
    };

    // 2. Scan for arguments [N] and [default]
    let mut args = 0;
    let mut has_optional = false;

    // Check next element for [argc]
    if let Some(n) = next_element() {
        let mut next = n;

        // Check for [argc]
        // Handles [N] as Group OR [ + N + ] tokens
        if next.to_string().starts_with('[') {
            let is_token_bracket = next.to_string() == "["; // LBracket token

            let mut num_str = String::new();

            if is_token_bracket {
                // Consume until ]
                loop {
                    if let Some(n) = next_element() {
                        let t = n.to_string();
                        if t == "]" {
                            break;
                        }
                        num_str.push_str(&t);
                    } else {
                        return None;
                    }
                }
            } else {
                // It's a Group like [2]
                num_str = next.to_string();
            }

            let clean_num = num_str.trim_matches(|c| c == '[' || c == ']').trim();
            if let Ok(n) = clean_num.parse::<usize>() {
                args = n;
            }

            // Move to next to check for [default] (Optional arg)
            if let Some(n) = next_element() {
                next = n;
                // Check for [opt]
                if next.to_string().starts_with('[') {
                    has_optional = true;

                    let is_token_bracket = next.to_string() == "[";
                    if is_token_bracket {
                        // Consume until ]
                        loop {
                            if let Some(n) = next_element() {
                                if n.to_string() == "]" {
                                    break;
                                }
                            } else {
                                return None;
                            }
                        }
                    }

                    // Move to body
                    if let Some(b) = next_element() {
                        next = b;
                    } else {
                        return None;
                    }
                }
            } else {
                return None;
            }
        }

        // Now `next` should be the body { ... }
        // We record the range of the definition body for "Peek Definition"

        Some(MacroDef {
            name,
            args,
            has_optional,
            definition_range: next.text_range(),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrotex_syntax::parse;

    #[test]
    fn test_scan_macros_basic() {
        let text = r"\newcommand{\foo}{bar}";
        let parse = parse(text);
        let macros = scan_macros(&parse.syntax());
        assert_eq!(macros.len(), 1);
        assert_eq!(macros[0].name, "\\foo");
        assert_eq!(macros[0].args, 0);
        assert!(!macros[0].has_optional);
    }

    #[test]
    fn test_scan_macros_args() {
        let text = r"\newcommand{\baz}[2]{Arg #1 and #2}";
        let parse = parse(text);
        let macros = scan_macros(&parse.syntax());
        assert_eq!(macros.len(), 1);
        assert_eq!(macros[0].name, "\\baz");
        assert_eq!(macros[0].args, 2);
        assert!(!macros[0].has_optional);
    }

    #[test]
    fn test_scan_macros_optional() {
        let text = r"\newcommand{\opt}[2][default]{Opt #1, Mand #2}";
        let parse = parse(text);
        let macros = scan_macros(&parse.syntax());
        assert_eq!(macros.len(), 1);
        assert_eq!(macros[0].name, "\\opt");
        assert_eq!(macros[0].args, 2);
        assert!(macros[0].has_optional);
    }

    #[test]
    fn test_scan_macros_providecommand() {
        let text = r"\providecommand{\prov}{content}";
        let parse = parse(text);
        let macros = scan_macros(&parse.syntax());
        assert_eq!(macros.len(), 1);
        assert_eq!(macros[0].name, "\\prov");
    }

    #[test]
    fn test_scan_macros_renewcommand() {
        let text = r"\renewcommand{\renew}{content}";
        let parse = parse(text);
        let macros = scan_macros(&parse.syntax());
        assert_eq!(macros.len(), 1);
        assert_eq!(macros[0].name, "\\renew");
    }

    #[test]
    fn test_scan_macros_robust() {
        let text = r"\DeclareRobustCommand{\robust}{content}";
        let parse = parse(text);
        let macros = scan_macros(&parse.syntax());
        assert_eq!(macros.len(), 1);
        assert_eq!(macros[0].name, "\\robust");
    }

    #[test]
    fn test_scan_macros_invalid() {
        // Missing name
        let text = r"\newcommand{body}";
        let parse_res = parse(text);
        let macros = scan_macros(&parse_res.syntax());
        assert!(macros.is_empty());

        // Invalid name format
        let text2 = r"\newcommand{notacommand}{body}";
        let parse_res2 = parse(text2);
        let macros2 = scan_macros(&parse_res2.syntax());
        assert!(macros2.is_empty());
    }

    #[test]
    fn test_scan_macros_no_braces() {
        // \newcommand\foo{bar} is valid in LaTeX
        let text = r"\newcommand\foo{bar}";
        let parse = parse(text);
        let macros = scan_macros(&parse.syntax());
        assert_eq!(macros.len(), 1);
        assert_eq!(macros[0].name, "\\foo");
    }

    #[test]
    fn test_scan_macros_whitespace() {
        let text = r"\newcommand { \foo } [ 1 ] {bar}";
        let parse = parse(text);
        let macros = scan_macros(&parse.syntax());
        assert_eq!(macros.len(), 1);
        assert_eq!(macros[0].name, "\\foo");
        assert_eq!(macros[0].args, 1);
    }
}
