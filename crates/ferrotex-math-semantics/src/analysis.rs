use crate::{Dimension, Shape};
use ferrotex_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};
// use rowan::ast::AstNode;

/// Analyzes a SyntaxNode (typically an Environment) to infer its mathematical shape.
pub fn infer_shape(node: &SyntaxNode) -> Shape {
    let mut rows = Vec::new();
    let mut current_row_cols = 0;

    // We must use descendants_with_tokens to see leaf nodes like Text and Command.
    for element in node.descendants_with_tokens() {
        if let SyntaxElement::Token(token) = element {
            let kind = token.kind();
            let text = token.text();

            if kind == SyntaxKind::Text {
                // Count ampersands in text blocks
                for char in text.chars() {
                    if char == '&' {
                        current_row_cols += 1;
                    }
                }
            } else if kind == SyntaxKind::Command && text == "\\\\" {
                rows.push(current_row_cols + 1);
                current_row_cols = 0;
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

        // Debug tree to be sure
        // for child in envs[0].descendants_with_tokens() {
        //    eprintln!("{:?}", child);
        // }

        let shape = infer_shape(&envs[0]);
        // rows=2, cols=2
        // input: 1 & 0 \\ 0 & 1
        // row 1: 1 (0) & (+) 0 (0) -> 1 ampersand -> 2 cols
        // row 2: 0 (0) & (+) 1 (0) -> 1 ampersand -> 2 cols
        // Correct.
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
}
