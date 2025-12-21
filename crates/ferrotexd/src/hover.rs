use ferrotex_syntax::{SyntaxKind, SyntaxNode, TextRange, TextSize};
use rowan::TokenAtOffset;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind};

/// Computes hover information for the given cursor position.
/// 
/// Supports:
/// - Environment Math (e.g. `\begin{equation}`) -> Rendered Math Block
/// - Inline Math (e.g. `$x$`) -> Rendered Math Block
pub fn find_hover(root: &SyntaxNode, offset: TextSize) -> Option<Hover> {
    let token = match root.token_at_offset(offset) {
        TokenAtOffset::None => return None,
        TokenAtOffset::Single(t) => t,
        TokenAtOffset::Between(l, r) => {
            // Prefer the token that is not whitespace if possible, or right one
            if l.kind() != SyntaxKind::Whitespace {
                l
            } else {
                r
            }
        }
    };

    // Strategy 1: Check for Environment "equation" or "align"
    // Use the AST parent structure for robust environment checking
    let mut current = token.parent()?;
    while current.kind() != SyntaxKind::Root {
        if current.kind() == SyntaxKind::Environment {
            // Check environment name
            // Environment -> Command(\begin) -> Group({equation})
            // This requires traversing children of Environment to find the name.
            // Simplified: get full text of environment and regex/string check?
            // "Better": look at children.
            
            // For now, let's just use the range of the environment if it looks like math.
            // But getting the NAME of the environment from CST is strictly better.
            let text = current.to_string();
            // Simple heuristic to extract name: after \begin{ and before }
            let env_name = if let Some(start) = text.find("\\begin{") {
                if let Some(end) = text[start..].find('}') {
                    &text[start + 7..start + end]
                } else {
                    "Environment"
                }
            } else {
                "Environment"
            };

            if text.contains("\\begin{equation}") || text.contains("\\begin{align}") || text.contains("\\begin{gather}") {
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("**{}**\n$$\n{}\n$$", env_name, text.trim()),
                    }),
                    range: None, 
                });
            }
            // Stop bubbling if we hit an environment, don't show outer environment?
            // Usually inner is most relevant.
            break;
        }
        current = current.parent()?;
    }

    // Strategy 2: Inline Math $...$
    // Since our parser doesn't have "Math" nodes yet (only tokens), we scan siblings.
    // This is heuristics-based on token stream until parser supports inline math.
    
    // Scan backwards for $
    let mut left_dollar = None;
    let mut right_dollar = None;

    // We start from `token` and walk siblings? No, `rowan` tokens don't have siblings chain easily without parent.
    // Parent is likely Root or Group.
    
    // Let's iterate all tokens in the parent file? No, too slow.
    // `token.siblings_with_tokens(Direction)`? Not available on Token, only Node.
    // But `token` is owned by `token.parent()`.
    
    // We can iterate children of parent.
    if let Some(parent) = token.parent() {
        let children: Vec<_> = parent.children_with_tokens().collect();
        // Find index of our token
        if let Some(idx) = children.iter().position(|it| it.as_token() == Some(&token)) {
             // Scan left
             for i in (0..idx).rev() {
                 #[allow(clippy::collapsible_if)]
                 if let Some(t) = children[i].as_token() {
                     if t.kind() == SyntaxKind::Dollar {
                         left_dollar = Some(t.text_range());
                         break;
                     }
                     // Stop at newlines boundaries for safety (inline math usually single line, but not always)
                     // if t.text().contains('\n') { break; } // LaTeX allows newlines in math
                 }
                 // If we hit another composite node? Inline math is usually flat tokens if parser puts them in Root.
             }

             // Scan right
             #[allow(clippy::needless_range_loop)]
             for i in idx..children.len() {
                 #[allow(clippy::collapsible_if)]
                 if let Some(t) = children[i].as_token() {
                     if t.kind() == SyntaxKind::Dollar {
                        right_dollar = Some(t.text_range());
                        break;
                     }
                 }
             }

             if let (Some(l), Some(r)) = (left_dollar, right_dollar) {
                 // Valid inline math span
                 let range = TextRange::new(l.start(), r.end());
                 let math_text = &root.to_string()[range]; // Slicing root text is safe-ish if we have reference
                 // Actually `root.text().slice(range)`
                 
                 // Clean up $ signs for display?
                 // MD expects `$$ x $$` for display math or `$ x $` for inline.
                 // Let's strip the outer $ for the markdown block if we wrap in $$
                 let inner = &math_text[1..math_text.len()-1];
                 
                 return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("$${}$$\n(Inline Math)", inner),
                    }),
                    range: None, 
                });
             }
        }
    }

    // Strategy 3: Display Math \[ ... \]
    // Similar scanning but looking for Command(\[) and Command(\])
    // The lexer might split \[ into Command(\[).
    // Let's check token.parent siblings again?
    // Same scan logic, just different delimiters.

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrotex_syntax::parse;

    #[test]
    fn test_hover_environment() {
        let input = r#"
        \begin{equation}
            E = mc^2
        \end{equation}
        "#;
        let p = parse(input);
        // Offset inside "mc^2".
        // \begin{equation} is approx 20 chars long?
        // Let's find offset of "mc^2"
        let offset = TextSize::from(input.find("mc^2").unwrap() as u32);
        let hover = find_hover(&p.syntax(), offset).expect("No hover found");
        
        match hover.contents {
            HoverContents::Markup(m) => {
                assert_eq!(m.kind, MarkupKind::Markdown);
                assert!(m.value.contains("$$"));
                assert!(m.value.contains("E = mc^2"));
            },
            _ => panic!("Wrong hover content type"),
        }
    }

    #[test]
    fn test_hover_inline_math() {
        // Need to make sure $ is tokenized as Dollar for this test to pass
        // Assuming user accepted ferrotex-syntax changes
        let input = r#"Text $ a^2 + b^2 = c^2 $ Text"#;
        let p = parse(input);
        let offset = TextSize::from(input.find("b^2").unwrap() as u32);
        let hover = find_hover(&p.syntax(), offset).expect("No hover found for inline math");
        
        match hover.contents {
            HoverContents::Markup(m) => {
                assert!(m.value.contains("$$"));
                assert!(m.value.contains("a^2 + b^2 = c^2"));
                assert!(m.value.contains("Inline Math"));
            },
            _ => panic!("Wrong hover content type"),
        }
    }
}
