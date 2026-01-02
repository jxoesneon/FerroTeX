use ferrotex_math_semantics::analysis::infer_shape;
use ferrotex_math_semantics::delimiters::check_delimiters;
use ferrotex_math_semantics::Shape;
use ferrotex_syntax::{SyntaxKind, SyntaxNode};
use line_index::LineIndex;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

pub fn check_math(root: &SyntaxNode, line_index: &LineIndex) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // 1. Check delimiter balance
    for error in check_delimiters(root) {
        let offset = rowan::TextSize::from(error.offset as u32);
        let pos = line_index.line_col(offset);
        let lsp_range = Range {
            start: Position {
                line: pos.line,
                character: pos.col,
            },
            end: Position {
                line: pos.line,
                character: pos.col + 1,
            },
        };
        diagnostics.push(Diagnostic {
            range: lsp_range,
            severity: Some(DiagnosticSeverity::WARNING),
            code: Some(tower_lsp::lsp_types::NumberOrString::String(
                "delimiter-mismatch".to_string(),
            )),
            code_description: None,
            source: Some("ferrotex-math".to_string()),
            message: error.message,
            related_information: None,
            tags: None,
            data: None,
        });
    }

    // 2. Check matrix shapes
    for node in root.descendants() {
        if node.kind() == SyntaxKind::Environment {
            // Check if it is a matrix environment
            let mut is_matrix = false;
            // Naive check: scan children for group containing "matrix"
            for child in node.children() {
                if child.kind() == SyntaxKind::Group {
                    let text = child.text().to_string();
                    if text.contains("matrix") {
                        is_matrix = true;
                        break;
                    }
                }
            }

            if is_matrix {
                let shape = infer_shape(&node);
                if let Shape::Invalid(msg) = shape {
                    let range = node.text_range();
                    let start = line_index.line_col(range.start());
                    let end = line_index.line_col(range.end());

                    let lsp_range = Range {
                        start: Position {
                            line: start.line,
                            character: start.col,
                        },
                        end: Position {
                            line: end.line,
                            character: end.col,
                        },
                    };

                    diagnostics.push(Diagnostic {
                        range: lsp_range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(tower_lsp::lsp_types::NumberOrString::String(
                            "math-semantics".to_string(),
                        )),
                        code_description: None,
                        source: Some("ferrotex-math".to_string()),
                        message: msg,
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrotex_syntax::parse;
    use line_index::LineIndex;

    #[test]
    fn test_check_math_no_matrix() {
        let input = r"\begin{document}Hello\end{document}";
        let parsed = parse(input);
        let root = SyntaxNode::new_root(parsed.green_node());
        let line_index = LineIndex::new(input);

        let diags = check_math(&root, &line_index);
        assert!(diags.is_empty(), "No matrix = no diagnostics");
    }

    #[test]
    fn test_check_math_valid_matrix() {
        // A well-formed matrix should not produce errors
        let input = r"\begin{pmatrix}1 & 2 \\ 3 & 4\end{pmatrix}";
        let parsed = parse(input);
        let root = SyntaxNode::new_root(parsed.green_node());
        let line_index = LineIndex::new(input);

        let diags = check_math(&root, &line_index);
        assert!(diags.is_empty(), "Valid matrix should have no diagnostics");
    }

    #[test]
    fn test_check_math_invalid_matrix_shape() {
        // Uneven rows: 2 columns in first row, 1 column in second
        let input = r"\begin{pmatrix}1 & 2 \\ 3\end{pmatrix}";
        let parsed = parse(input);
        let root = SyntaxNode::new_root(parsed.green_node());
        let line_index = LineIndex::new(input);

        let diags = check_math(&root, &line_index);
        assert!(
            !diags.is_empty(),
            "Invalid matrix shape should produce diagnostics"
        );
        assert!(diags.iter().any(|d| d.message.contains("Jagged matrix")));
    }

    #[test]
    fn test_check_math_empty_matrix() {
        let input = r"\begin{pmatrix}\end{pmatrix}";
        let parsed = parse(input);
        let root = SyntaxNode::new_root(parsed.green_node());
        let line_index = LineIndex::new(input);

        let diags = check_math(&root, &line_index);
        // Empty matrix might be valid or not depending on parser, but shouldn't panic
        // It technically has 1 row, 0 columns? Or 0 rows?
        // Let's just ensure it scans.
        let _ = diags;
    }
}
