use ferrotex_syntax::{SyntaxKind, SyntaxNode};
use rowan::NodeOrToken;
use tower_lsp::lsp_types::{Position, Range, TextEdit};

/// Formats the entire document represented by `root`.
///
/// This is a conservative formatter. It primarily focuses on:
/// 1. Correcting indentation for environment blocks.
/// 2. Trimming trailing whitespace (optional, effectively side-effect of correct indentation if line is re-emitted).
///
/// It does NOT aggressively reflow text or change line breaks.
pub fn format_document(root: &SyntaxNode, line_index: &line_index::LineIndex) -> Vec<TextEdit> {
    let mut edits = Vec::new();
    // let indent_level = 0;

    // We walk the tree in preorder.
    // However, for indentation, line-based processing is often easier given that LaTeX is free-form.
    // Mixed approach:
    // 1. Traverse to calculate the "target indent" for each line.
    // 2. Diff against "actual indent".

    // Let's model this by lines for simplicity, using the CST to inform the indent level.
    // This is valid because indentation changes are driven by `\begin` and `\end` which are CST nodes.

    // Valid indentation triggers:
    // - Enter \begin{...} -> +1 for following lines
    // - Enter \end{...} -> -1 for current line (and following)

    // Map line_number -> indentation_delta
    // But we need to be careful with multiple commands on one line.

    // Let's try a token stream approach.
    let mut indent_depth: i32 = 0;
    // let last_line = 0;

    // We will collect "Line X should have depth Y" events.
    // Because `\begin` increases depth for the *contents*, it affects lines *after* the `\begin` line.
    // `\end` decreases depth for *itself*.

    // Store (line, depth)
    let mut line_depths = std::collections::BTreeMap::new();

    for event in root.preorder_with_tokens() {
        match event {
            rowan::WalkEvent::Enter(element) => {
                match element {
                    NodeOrToken::Node(n) => {
                        if n.kind() == SyntaxKind::Environment {
                            // The Environment node wraps \begin, content, and \end.
                            // We don't increment here, we wait for the specific parts.
                            // Actually, ferrotex-syntax structure for Environment might be:
                            // Environment
                            //   Command (\begin) ...
                            //   Content...
                            //   Command (\end) ...
                        }
                    }
                    NodeOrToken::Token(t) => {
                        if t.kind() == SyntaxKind::Command {
                            let text = t.text();
                            if text == "\\begin" {
                                indent_depth += 1;
                            } else if text == "\\end" {
                                indent_depth = indent_depth.saturating_sub(1);
                                // The \end line itself should be at the lower depth
                                let start_line = line_index.line_col(t.text_range().start()).line;
                                line_depths.insert(start_line, indent_depth);
                            }
                        }

                        // Capture the depth for the current line of this token
                        let start_line = line_index.line_col(t.text_range().start()).line;
                        line_depths.entry(start_line).or_insert(indent_depth);
                    }
                }
            }
            rowan::WalkEvent::Leave(_element) => {
                // In preorder traversal, Leave follows Enter.
                // We handled state changes in Enter.
            }
        }
    }

    // Now generate edits for lines that have wrong indentation.
    // This assumes we want to indent non-empty lines.
    // We need to read the actual file lines to compare.
    // But we don't have the string directly here? accessing `root.text()` is costly if huge.
    // We can assume we have access to it.

    // Wait, the standard way is to return TextEdits.
    // We can iterate over all lines in the file.
    let text = root.to_string(); // This constructs the full string, acceptable for v0.10.
    let lines: Vec<&str> = text.lines().collect();

    // Recalculate accurate depths based on the token walk above?
    // The token walk above was a bit rough.
    // Let's refine:
    // We need to know which lines start with `\end`.
    // And which lines are inside an environment.

    // Reset
    // indent_depth = 0;
    // let mut computed_depths = Vec::with_capacity(lines.len());

    // We need to map lines to tokens to see what commands are on them.
    // Or just look for the text? No, use CST for robustness (don't indent comments like commands).

    // New strategy:
    // 1. Scan tokens. Record Line -> Delta (+1, -1).
    // 2. Prefix sum to get target indent for each line.

    let mut line_deltas = vec![0isize; lines.len() + 1];

    for token in root
        .descendants_with_tokens()
        .filter_map(|e| e.into_token())
    {
        if token.kind() == SyntaxKind::Command {
            let t = token.text();
            let line = line_index.line_col(token.text_range().start()).line as usize;
            if line >= lines.len() {
                continue;
            }

            if t == "\\begin" {
                // \begin increases indent for the NEXT line
                if line + 1 < line_deltas.len() {
                    line_deltas[line + 1] += 1;
                }
            } else if t == "\\end" {
                // \end decreases indent for THIS line (and thus following)
                if line < line_deltas.len() {
                    line_deltas[line] -= 1;
                }
                // But wait, if we decrease at line i, we must ensure line i+1 doesn't decrease AGAIN
                // unless we are using absolute depths.
                // Prefix sum strategy works on "Change in depth at this line".

                // Let's verify:
                // Line 0: \begin{foo}  (Current: 0. Next: +1)
                // Line 1:   bar        (Current: 1)
                // Line 2: \end{foo}    (Current: 0)

                // So \begin at L0 means L1 gets +1.
                // \end at L2 means L2 gets -1 relative to L1.

                // My logic above:
                // \begin at L -> delta[L+1] += 1
                // \end at L   -> delta[L] -= 1
                // AND we need to correct for the fact that \begin closes.
                // Actually \begin affects all subsequent lines.
                // \end affects current and subsequent lines.

                // So line_deltas stores "change to applying depth" at this index?
                // No, simpler:
                // depth[i] = depth[i-1] + delta[i]

                // \begin at L:
                // It signifies a level increase.
                // If we have just \begin, level increases.
                // If we have \begin ... \end on ONE line, level is 0 change for next line.
            }
        }
    }

    // Correct strategy:
    // Walk tokens. Track `current_indent`.
    // When we hit new line, record `current_indent`.
    // Special case: if a line *starts* with `\end` (ignoring whitespace), it should use `current_indent - 1`.

    // Let's do a concrete walk.
    // let current_indent = 0;
    let mut target_indents = vec![0; lines.len()];

    // We need to iterate lines and find the "indentation critical tokens" on them.
    // But lines are not nodes.

    // We can iterate the tokens and group by line.
    // Or just use the string text for checking `\end` at start of line?
    // Using simple string matching for `^\s*\\end` is risky if it's in a comment.
    // But `ferrotex-syntax` handles comments.

    // Robust approach:
    // 1. Identify lines that contain `\begin` or `\end` as significant tokens.
    // 2. Determine net effect on indent.

    let mut line_effects = vec![(0isize, 0isize); lines.len()]; // (pre_adjustment, post_adjustment)
    // pre_adjustment: applied to THIS line (e.g. \end)
    // post_adjustment: applied to NEXT line (e.g. \begin)

    for token in root
        .descendants_with_tokens()
        .filter_map(|e| e.into_token())
    {
        if token.kind() == SyntaxKind::Command {
            let txt = token.text();
            let line = line_index.line_col(token.text_range().start()).line as usize;
            if line >= lines.len() {
                continue;
            }

            if txt == "\\begin" {
                line_effects[line].1 += 1; // Increment for next line
            } else if txt == "\\end" {
                line_effects[line].0 -= 1; // Decrement for this line
                // If we decrement for this line, we IMPLICITLY decrement for next line too purely by state carry-over?
                // No, we need to model the state machine.
            }
        }
    }

    // Now compute state
    let mut depth = 0isize;
    for (i, (pre, post)) in line_effects.iter().enumerate() {
        // "pre" affects the current line's visuals (like \end usually outdents itself)
        // But strictly, formatting state is:
        // Start of line depth = End of previous line depth.
        // Then we apply "pre" modifiers?

        // Actually:
        // Depth at start of Line i = Depth at end of Line i-1.
        // But if Line i contains `\end` at the start, we want to visually render it with -1.
        // AND calculate end-of-line depth with -1.

        let visual_depth = depth + pre;
        target_indents[i] = visual_depth.max(0) as usize;

        // Calculate depth for next line
        // Net change for this line is (count(\begin) - count(\end))?
        // My line_effects logic:
        // \begin: post += 1
        // \end: pre -= 1

        // Total delta = pre + post?
        // \begin: pre=0, post=1. Net +1. Correct.
        // \end: pre=-1, post=0. Net -1. Correct.
        // \begin \end: pre=-1, post=1. Net 0. Correct.

        depth += pre + post;
    }

    // Generate Edits
    for (i, line_content) in lines.iter().enumerate() {
        let trimmed = line_content.trim_start();
        if trimmed.is_empty() {
            // Don't indent empty lines
            continue;
        }

        let target_indent_count = target_indents[i] * 4; // 4 spaces
        let current_indent_str = &line_content[..(line_content.len() - trimmed.len())];
        let current_indent_count = current_indent_str.len(); // Assuming spaces. If tabs, this is fuzzy.

        // If strict spaces:
        let target_str = " ".repeat(target_indent_count);
        if current_indent_str != target_str {
            // Replace indentation
            edits.push(TextEdit {
                range: Range {
                    start: Position {
                        line: i as u32,
                        character: 0,
                    },
                    end: Position {
                        line: i as u32,
                        character: current_indent_count as u32,
                    },
                },
                new_text: target_str,
            });
        }
    }

    edits
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrotex_syntax::parse;
    use line_index::LineIndex;

    fn check_format(input: &str, expected: &str) {
        let parse = parse(input);
        let root = parse.syntax();
        let line_index = LineIndex::new(input);

        let edits = format_document(&root, &line_index);

        // Apply edits to simulate result
        // Simplified applier since our edits are line-based replacements
        // let mut result = input.to_string();
        // Sort edits reverse by range so indices don't shift?
        // Our edits replace the indentation at start of lines.
        // If we process from bottom to top line, it's safe.
        // Edits are ordered by line ID (0..N).
        // So reverse iteration works.

        let mut lines: Vec<String> = input.lines().map(|s| s.to_string()).collect();
        for edit in edits {
            let line_idx = edit.range.start.line as usize;
            if line_idx < lines.len() {
                let line_content = &lines[line_idx];
                let trimmed = line_content.trim_start();
                let indent = edit.new_text;
                lines[line_idx] = format!("{}{}", indent, trimmed);
            }
        }

        let mut actual = lines.join("\n");
        // Re-add trailing newline if input had it, split removes it
        if input.ends_with('\n') {
            actual.push('\n');
        }

        assert_eq!(actual, expected, "Formatting mismatch");
    }

    #[test]
    fn test_format_indentation() {
        let input = r#"\documentclass{article}
\begin{document}
Hello World
\begin{itemize}
\item One
\item Two
\end{itemize}
\end{document}"#;

        let expected = r#"\documentclass{article}
\begin{document}
    Hello World
    \begin{itemize}
        \item One
        \item Two
    \end{itemize}
\end{document}"#;


        check_format(input, expected);
    }

    #[test]
    fn test_format_idempotency() {
        let input = r#"\documentclass{article}
\begin{document}
    \begin{itemize}
        \item One
    \end{itemize}
\end{document}"#;
        // First format
        let parse = parse(input);
        let root = parse.syntax();
        let line_index = LineIndex::new(input);
        let edits = format_document(&root, &line_index);
        
        // Apply edits (which should be none if already formatted, or minimal)
        // If the input is already well-formatted, formatting it again should yield zero edits?
        // Our formatter always returns edits if indentation mismatches.
        // If it returns edits, applying them should result in the same string.
        assert!(edits.is_empty(), "Well-formatted document should produce no edits");
    }

    #[test]
    fn test_format_preserves_blank_lines() {
        let input = r#"\begin{document}
    Hello

    World
\end{document}"#;
        let expected = r#"\begin{document}
    Hello

    World
\end{document}"#;
        // Blank lines inside should be preserved (though potentially indented if not empty)
        // Our logic: trimmed.is_empty() -> continue. So blank lines are untouched.
        
        check_format(input, expected);
    }
}
