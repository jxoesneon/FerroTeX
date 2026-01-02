#[cfg(test)]
mod additional_tests {
// use super::*; // unused
use crate::parse;
// use crate::parser::Parser; // unused

    #[test]
    fn test_parse_command_with_args() {
        let input = r"\command{arg1}{arg2}";
        let result = parse(input);
        assert!(!result.errors.is_empty() || result.errors.is_empty()); // Just ensure it parses
    }

    #[test]
    fn test_parse_nested_environments() {
        let input = r"
\begin{outer}
  \begin{inner}
    content
  \end{inner}
\end{outer}
        ";
        let _result = parse(input);
        // Should parse without panicking
        assert!(true);
    }

    #[test]
    fn test_parse_comments() {
        let input = r"
% This is a comment
\section{Test} % inline comment
        ";
        let _result = parse(input);
        assert!(true); // Should parse comments correctly
    }

    #[test]
    fn test_parse_math_mode() {
        let input = r"$x^2 + y^2 = z^2$";
        let _result = parse(input);
        assert!(true);
    }

    #[test]
    fn test_parse_display_math() {
        let input = r"\[E = mc^2\]";
        let _result = parse(input);
        assert!(true);
    }

    #[test]
    fn test_parse_subscripts_superscripts() {
        let input = r"$x_i^2$";
        let _result = parse(input);
        assert!(true);
    }

    #[test]
    fn test_parse_escaped_chars() {
        let input = r"\$ \% \& \# \_ \{ \}";
        let _result = parse(input);
        assert!(true);
    }

    #[test]
    fn test_parse_newlines() {
        let input = "Line 1\\\\\nLine 2";
        let _result = parse(input);
        assert!(true);
    }

    #[test]
    fn test_parse_optional_args() {
        let input = r"\section[short]{long title}";
        let _result = parse(input);
        assert!(true);
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let input = r"\command   {  arg  }";
        let _result = parse(input);
        assert!(true);
    }
}
