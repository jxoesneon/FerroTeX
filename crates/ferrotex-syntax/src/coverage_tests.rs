use crate::lexer::Lexer;
use crate::parser::parse;
use crate::SyntaxKind;
// use rowan::TextRange; // unused

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn test_lexer_brackets_and_math() {
        let input = r"\[ E = mc^2 \]";
        let mut lexer = Lexer::new(input);
        // \[
        let (k1, t1) = lexer.next_token();
        assert_eq!(k1, SyntaxKind::Command);
        assert_eq!(t1, "\\[");
        
        lexer.next_token(); // space
        
        let (k2, t2) = lexer.next_token();
        assert_eq!(k2, SyntaxKind::Text);
        assert_eq!(t2, "E");
    }

    #[test]
    fn test_lexer_eof() {
        let input = "";
        let mut lexer = Lexer::new(input);
        let (k, t) = lexer.next_token();
        assert_eq!(k, SyntaxKind::Eof);
        assert_eq!(t, "");
    }

    #[test]
    fn test_lexer_whitespace_and_newlines() {
        let input = "  \n  ";
        let mut lexer = Lexer::new(input);
        let (k, t) = lexer.next_token();
        assert_eq!(k, SyntaxKind::Whitespace);
        assert_eq!(t, "  \n  ");
    }

    #[test]
    fn test_lexer_single_char_commands() {
        let input = r"\$ \% \_";
        let mut lexer = Lexer::new(input);
        let (k, t) = lexer.next_token();
        assert_eq!(k, SyntaxKind::Command);
        assert_eq!(t, r"\$");
        lexer.next_token(); // space
        let (k2, t2) = lexer.next_token();
        assert_eq!(k2, SyntaxKind::Command);
        assert_eq!(t2, r"\%");
    }

    #[test]
    fn test_parser_unclosed_environment() {
        let input = r"\begin{document} Hello";
        let res = parse(input);
        assert!(!res.errors.is_empty());
        assert_eq!(res.errors[0].message, "Unclosed environment, expected \\end");
    }

    #[test]
    fn test_parser_missing_brace_after_begin() {
        let input = r"\begin document}";
        let res = parse(input);
        assert!(!res.errors.is_empty());
        assert!(res.errors[0].message.contains("Expected '{'"));
    }
    
    #[test]
    fn test_parser_missing_brace_after_end() {
        let input = r"\begin{a}\end a}";
        let res = parse(input);
        assert!(!res.errors.is_empty());
        assert!(res.errors[0].message.contains("Expected '{'"));
    }

    #[test]
    fn test_parser_unmatched_rbrace_in_env() {
        let input = r"\begin{a} } \end{a}";
        let res = parse(input);
        assert!(!res.errors.is_empty());
        assert_eq!(res.errors[0].message, "Unmatched '}' inside environment");
    }
    
    #[test]
    fn test_parser_unmatched_rbrace_toplevel() {
        let input = r"}";
        let res = parse(input);
        assert!(!res.errors.is_empty());
        assert_eq!(res.errors[0].message, "Unmatched '}'");
    }

    #[test]
    fn test_parser_citation_missing_brace() {
        let input = r"\cite key}";
        let res = parse(input);
        assert!(!res.errors.is_empty());
    }

    #[test]
    fn test_parser_citation_bad_optional() {
        let input = r"\cite [ arg } {key}";
        let res = parse(input);
        assert!(!res.errors.is_empty()); // Expected ]
    }

    #[test]
    fn test_parser_bibliography_missing_brace() {
        let input = r"\bibliography refs}";
        let res = parse(input);
        assert!(!res.errors.is_empty());
    }
    
    #[test]
    fn test_parser_addbibresource_bad_optional() {
        let input = r"\addbibresource [backend=biber {refs.bib}";
        // It might consume until } or EOF looking for ]
        let res = parse(input);
        assert!(!res.errors.is_empty());
    }

    #[test]
    fn test_parser_label_missing_brace() {
        let input = r"\label val}";
        let res = parse(input);
        assert!(!res.errors.is_empty());
    }

    #[test]
    fn test_parser_ref_missing_brace() {
        let input = r"\ref val}";
        let res = parse(input);
        assert!(!res.errors.is_empty());
    }

    #[test]
    fn test_parser_include_missing_brace() {
        let input = r"\input file}";
        let res = parse(input);
        assert!(!res.errors.is_empty());
    }
    
    #[test]
    fn test_parser_missing_brace_after_section() {
        let input = r"\section Title}";
        let res = parse(input);
        assert!(!res.errors.is_empty());
    }

    #[test]
    fn test_nested_env_correct() {
        let input = r"\begin{outer} \begin{inner} \end{inner} \end{outer}";
        let res = parse(input);
        assert!(res.errors.is_empty());
    }
    
    #[test]
    fn test_deeply_nested_structure() {
        let input = r"\begin{a}\begin{b}\begin{c}\end{c}\end{b}\end{a}";
         let res = parse(input);
        assert!(res.errors.is_empty());
    }

    #[test]
    fn test_verbatim_like_environments() {
        // Our parser is naive and parses content of verbatim like normal content
        // unless we add specific support. This test just ensures it doesn't crash.
        let input = r"\begin{verbatim} \end{verbatim} $ % & { } ";
        let _res = parse(input);
        // It should try to parse $ % etc as tokens or comments.
        // It might error on unmatched braces if verbatim contains them?
        // Actually our environment parser loops until \end
    }

    #[test]
    fn test_command_staggered() {
        let input = r"\a\b\c";
        let res = parse(input);
        assert!(res.errors.is_empty());
    }
    
    #[test]
    fn test_unexpected_tokens_recovery() {
        let input = r"\begin{a} \unknown [ ] { } \end{a}";
        let res = parse(input);
        assert!(res.errors.is_empty());
    }

    #[test]
    fn test_lexer_comment() {
        let input = "% A comment\nNext";
        let mut lexer = Lexer::new(input);
        let (k, t) = lexer.next_token();
        assert_eq!(k, SyntaxKind::Comment);
        assert_eq!(t, "% A comment"); // Newline is usually consumed or stops it?
        // Lexer impl: while next != \n && != \r
        // So \n is NOT consumed in the comment token.
        
        let (k2, _t2) = lexer.next_token();
        assert_eq!(k2, SyntaxKind::Whitespace); // The newline
    }

    #[test]
    fn test_lexer_error_token() {
        // Find a char that falls into "_" but not Text loop if stuck?
        // Actually "_" in match is Text run.
        // It consumes until special char.
        // Let's try control char or emojis?
        let input = "💩";
        let mut lexer = Lexer::new(input);
        let (k, t) = lexer.next_token();
        assert_eq!(k, SyntaxKind::Text);
        assert_eq!(t, "💩");
    }

    // NEW PREVIOUSLY MISSED TESTS
    #[test]
    fn test_lexer_generic_tokens() {
        // Test symbols that default to Token
        let input = "@ # $";
        let mut lexer = Lexer::new(input);
        
        let (k1, t1) = lexer.next_token();
        assert_eq!(k1, SyntaxKind::Text); // @ is Text in our lexer fallback
        assert_eq!(t1, "@");
        
        lexer.next_token(); // whitespace
        
        let (k2, _t2) = lexer.next_token();
        assert_eq!(k2, SyntaxKind::Text); // # is Text
        
        lexer.next_token(); // whitespace
        
        // $ is Dollar
        let (k3, _t3) = lexer.next_token();
        assert_eq!(k3, SyntaxKind::Dollar);
    }
    
    #[test]
    fn test_lexer_math_dollar() {
         let input = "$$";
         let mut lexer = Lexer::new(input);
         let (k1, _) = lexer.next_token();
         assert_eq!(k1, SyntaxKind::Dollar);
         let (k2, _) = lexer.next_token();
         assert_eq!(k2, SyntaxKind::Dollar);
    }

    #[test]
    fn test_parser_unclosed_command() {
        // Command followed by EOF
        let input = "\\mycommand";
        let parse = parse(input);
        let root = parse.syntax();
        // Check that we have a Command element or token
        // In our parser impl, top level commands are usually wrapped? 
        // parse_element calls parse_command_or_environment.
        // If unknown command, it BUMPS.
        // So it produces a Command TOKEN at root?
        // Let's just check the text preserved.
        assert_eq!(root.to_string(), "\\mycommand");
    }
    
    #[test]
    fn test_parser_excessive_closing_braces() {
        let input = "text } more";
        let parse = parse(input);
        assert!(!parse.errors.is_empty());
        assert_eq!(parse.errors[0].message, "Unmatched '}'");
    }
    
    #[test]
    fn test_parser_command_empty_args() {
        // Command with {}
        let input = "\\cmd{}";
        let _ = parse(input);
    }

    #[test]
    fn test_parser_command_whitespace_in_args() {
        let input = "\\cmd{  arg  }";
        let _ = parse(input);
    }
}
