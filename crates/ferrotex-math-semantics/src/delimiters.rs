//! Delimiter validation for LaTeX math environments.
//!
//! Detects mismatched or unbalanced delimiters like:
//! - `\left` / `\right`
//! - Parentheses, brackets, braces

use ferrotex_syntax::SyntaxNode;

/// Represents a delimiter mismatch error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelimiterError {
    /// Human-readable description of the error.
    pub message: String,
    /// Byte offset in the source where the error was detected.
    pub offset: usize,
}

/// Checks a syntax tree for delimiter mismatches.
pub fn check_delimiters(root: &SyntaxNode) -> Vec<DelimiterError> {
    let mut errors = Vec::new();
    let text = root.text().to_string();
    
    // Track \left / \right pairs using text scanning
    let mut left_count = 0usize;
    let mut right_count = 0usize;
    
    // Find all \left and \right in text
    for (idx, _) in text.match_indices("\\left") {
        left_count += 1;
        // Check if there are more \rights than \lefts at this point
        let rights_before = text[..idx].matches("\\right").count();
        if rights_before > text[..idx].matches("\\left").count() {
            // Already handled
        }
    }
    
    for (idx, _) in text.match_indices("\\right") {
        right_count += 1;
        let lefts_before = text[..idx].matches("\\left").count();
        let rights_before = text[..idx].matches("\\right").count();
        if rights_before >= lefts_before {
            errors.push(DelimiterError {
                message: "Unmatched \\right without corresponding \\left".to_string(),
                offset: idx,
            });
        }
    }
    
    if left_count > right_count {
        errors.push(DelimiterError {
            message: format!("{} unmatched \\left delimiter(s)", left_count - right_count),
            offset: 0, // Report at start of document
        });
    }
    
    // Check basic bracket balance in math content
    let mut paren_stack: Vec<(char, usize)> = Vec::new();
    
    for (idx, ch) in text.char_indices() {
        match ch {
            '(' | '[' | '{' => paren_stack.push((ch, idx)),
            ')' => {
                if let Some((open, _)) = paren_stack.pop() {
                    if open != '(' {
                        errors.push(DelimiterError {
                            message: format!("Mismatched delimiter: expected closing for '{}', found ')'", open),
                            offset: idx,
                        });
                    }
                }
            }
            ']' => {
                if let Some((open, _)) = paren_stack.pop() {
                    if open != '[' {
                        errors.push(DelimiterError {
                            message: format!("Mismatched delimiter: expected closing for '{}', found ']'", open),
                            offset: idx,
                        });
                    }
                }
            }
            '}' => {
                if let Some((open, _)) = paren_stack.pop() {
                    if open != '{' {
                        errors.push(DelimiterError {
                            message: format!("Mismatched delimiter: expected closing for '{}', found '}}'", open),
                            offset: idx,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    
    // Report unclosed delimiters
    for (open, offset) in paren_stack {
        errors.push(DelimiterError {
            message: format!("Unclosed delimiter '{}'", open),
            offset,
        });
    }
    
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrotex_syntax::parse;

    #[test]
    fn test_balanced_delimiters() {
        let input = r"\left( x + y \right)";
        let parsed = parse(input);
        let root = SyntaxNode::new_root(parsed.green_node());
        let errors = check_delimiters(&root);
        assert!(errors.is_empty(), "Balanced delimiters should have no errors");
    }

    #[test]
    fn test_unmatched_left() {
        let input = r"\left( x + y";
        let parsed = parse(input);
        let root = SyntaxNode::new_root(parsed.green_node());
        let errors = check_delimiters(&root);
        assert!(!errors.is_empty(), "Unmatched \\left should produce error");
    }

    #[test]
    fn test_unmatched_right() {
        let input = r"x + y \right)";
        let parsed = parse(input);
        let root = SyntaxNode::new_root(parsed.green_node());
        let errors = check_delimiters(&root);
        assert!(!errors.is_empty(), "Unmatched \\right should produce error");
    }

    #[test]
    fn test_mismatched_delimiters() {
        let inputs = vec![
            "( ]",
            "[ }",
            "{ )",
        ];
        for input in inputs {
            let parsed = parse(input);
            let root = SyntaxNode::new_root(parsed.green_node());
            let errors = check_delimiters(&root);
            assert!(errors.iter().any(|e| e.message.contains("Mismatched delimiter")), "Should detect mismatched delimiter in '{}'", input);
        }
    }

    #[test]
    fn test_unclosed_paren() {
        let input = "(";
        let parsed = parse(input);
        let root = SyntaxNode::new_root(parsed.green_node());
        let errors = check_delimiters(&root);
        assert!(errors.iter().any(|e| e.message.contains("Unclosed delimiter")));
    }
}
