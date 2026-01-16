//! # Math Semantic Analysis
//!
//! Provides algorithms for inferring the structural properties of LaTeX math
//! environments, such as matrix dimensions and consistency.

use crate::{Dimension, Shape};
use ferrotex_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

/// Analyzes a SyntaxNode (typically an Environment) to infer its mathematical shape.
///
/// This function scans the descendants of the node for LaTeX alignment characters (`&`)
/// and row delimiters (`\\`) to determine the dimensions of a matrix-like environment.
///
/// # Features
///
/// - **Multicolumn Awareness**: Correctly accounts for `\multicolumn{n}{...}{...}` commands,
///   adding `n-1` to the column count of the current row.
/// - **Nested Group Protection**: Ignores alignment markers `&` inside protected groups `{...}`,
///   preventing false positives from complex cell content.
///
/// # Examples
///
/// ```
/// use ferrotex_syntax::parse;
/// use ferrotex_math_semantics::analysis::infer_shape;
///
/// let input = r"\begin{matrix} 1 & 2 \\ 3 & \multicolumn{2}{c}{4} \end{matrix}";
/// let root = parse(input).syntax();
/// let env = root.descendants().find(|n| n.kind() == ferrotex_syntax::SyntaxKind::Environment).unwrap();
/// let shape = infer_shape(&env);
/// ```
pub fn infer_shape(node: &SyntaxNode) -> Shape {
    let mut rows = Vec::new();
    let mut current_row_cols = 0;

    let mut it = node.children_with_tokens().peekable();
    while let Some(element) = it.next() {
        match element {
            SyntaxElement::Token(token) => {
                let kind = token.kind();
                let text = token.text();

                if kind == SyntaxKind::Text {
                    for char in text.chars() {
                        if char == '&' {
                            current_row_cols += 1;
                        }
                    }
                } else if kind == SyntaxKind::Command {
                    if text == "\\\\" {
                        rows.push(current_row_cols + 1);
                        current_row_cols = 0;
                    } else if text == "\\multicolumn" {
                        // 1. Find the column count group
                        while let Some(peeked) = it.peek() {
                            if peeked.kind() == SyntaxKind::Whitespace
                                || peeked.kind() == SyntaxKind::Comment
                            {
                                it.next();
                                continue;
                            }
                            if let SyntaxElement::Node(group) = peeked
                                && group.kind() == SyntaxKind::Group
                            {
                                let g_text = group.text().to_string();
                                let count_str =
                                    g_text.trim_matches(|c| c == '{' || c == '}').trim();
                                if let Ok(n) = count_str.parse::<usize>()
                                    && n > 1
                                {
                                    current_row_cols += n - 1;
                                }
                                it.next(); // Consume count group
                            }
                            break;
                        }
                        // 2. Skip the next two arguments: {align} and {content}
                        let mut skipped = 0;
                        while skipped < 2 {
                            if let Some(peeked) = it.peek() {
                                if peeked.kind() == SyntaxKind::Whitespace
                                    || peeked.kind() == SyntaxKind::Comment
                                {
                                    it.next();
                                    continue;
                                }
                                if peeked.kind() == SyntaxKind::Group {
                                    it.next();
                                    skipped += 1;
                                    continue;
                                }
                            }
                            break;
                        }
                    }
                }
            }
            SyntaxElement::Node(_) => {
                // Precision: ampersands inside Groups { & } are protected and don't count as delimiters.
                // We skip them here.
            }
        }
    }

    // Push the last row (add 1 because column count = ampersands + 1)
    rows.push(current_row_cols + 1);

    if rows.is_empty() {
        return Shape::Unknown;
    }

    // Check consistency
    let first_row_cols = rows[0];
    for (i, cols) in rows.iter().enumerate().skip(1) {
        if *cols != first_row_cols {
            return Shape::Invalid(format!(
                "Jagged matrix: row 1 has {} columns, but row {} has {}",
                first_row_cols,
                i + 1,
                cols
            ));
        }
    }

    Shape::Matrix {
        rows: Dimension::Finite(rows.len()),
        cols: Dimension::Finite(first_row_cols),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrotex_syntax::parse;

    #[test]
    fn test_matrix_shape() {
        // Valid 2x2.
        let input = r"\begin{pmatrix} 1 & 0 \\ 0 & 1 \end{pmatrix}";
        let parse = parse(input);
        let root = parse.syntax();
        let envs: Vec<_> = root
            .descendants()
            .filter(|kind| kind.kind() == SyntaxKind::Environment)
            .collect();
        assert!(!envs.is_empty(), "No environment found");

        let shape = infer_shape(&envs[0]);
        assert_eq!(
            shape,
            Shape::Matrix {
                rows: Dimension::Finite(2),
                cols: Dimension::Finite(2)
            }
        );
    }

    #[test]
    fn test_jagged_matrix() {
        let input = r"\begin{pmatrix} 1 & 0 \\ 1 & 2 & 3 \end{pmatrix}";
        let parse = parse(input);
        let root = parse.syntax();
        let envs: Vec<_> = root
            .descendants()
            .filter(|kind| kind.kind() == SyntaxKind::Environment)
            .collect();
        let shape = infer_shape(&envs[0]);
        match shape {
            Shape::Invalid(msg) => assert!(msg.contains("Jagged matrix")),
            _ => panic!("Expected jagged matrix error, got {:?}", shape),
        }
    }

    #[test]
    fn test_matrix_with_multicolumn() {
        // \multicolumn{2} counts as 2 columns.
        // Row 1: A & \multicolumn{2}{c}{B} & C   => 4 columns total.
        let input = r"\begin{pmatrix} A & \multicolumn{2}{c}{B} & C \\ D & E & F & G \end{pmatrix}";
        let parse = ferrotex_syntax::parse(input);
        let root = parse.syntax();
        let envs: Vec<_> = root
            .descendants()
            .filter(|kind| kind.kind() == ferrotex_syntax::SyntaxKind::Environment)
            .collect();
        let shape = infer_shape(&envs[0]);
        assert_eq!(
            shape,
            Shape::Matrix {
                rows: Dimension::Finite(2),
                cols: Dimension::Finite(4)
            }
        );
    }
}
