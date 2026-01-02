use crate::{SyntaxKind, SyntaxNode, lexer::Lexer};
use rowan::{GreenNode, GreenNodeBuilder, TextRange, TextSize};
use std::iter::Peekable;

/// Represents an error encountered during parsing.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SyntaxError {
    /// The error message.
    pub message: String,
    /// The range in the source text where the error occurred.
    pub range: TextRange,
}

/// The FerroTeX parser.
///
/// It takes a string input and produces a GreenNode (untyped syntax tree) and a list of errors.
/// It uses a recursive descent approach.
/// The core parser implementation.
///
/// This struct manages the parsing state, including the token stream
/// and the construction of the syntax tree via `rowan`.
///
/// It is typically used via the high-level [`parse`] function, but can be
/// accessed directly for advanced use cases (e.g., partial parsing).
pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    builder: GreenNodeBuilder<'static>,
    errors: Vec<SyntaxError>,
    current_offset: TextSize,
}

impl<'a> Parser<'a> {
    /// Creates a new `Parser` for the given input string.
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input).peekable(),
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
            current_offset: TextSize::from(0),
        }
    }

    /// Parses the input and returns the result.
    pub fn parse(mut self) -> ParseResult {
        self.builder.start_node(SyntaxKind::Root.into());
        while self.peek() != SyntaxKind::Eof {
            self.parse_element();
        }
        self.builder.finish_node();
        ParseResult {
            green_node: self.builder.finish(),
            errors: self.errors,
        }
    }

    fn peek(&mut self) -> SyntaxKind {
        self.lexer
            .peek()
            .map(|(k, _)| *k)
            .unwrap_or(SyntaxKind::Eof)
    }

    fn peek_text(&mut self) -> &str {
        self.lexer.peek().map(|(_, t)| *t).unwrap_or("")
    }

    fn bump(&mut self) {
        if let Some((kind, text)) = self.lexer.next() {
            self.builder.token(kind.into(), text);
            let len = TextSize::of(text);
            self.current_offset += len;
        }
    }

    fn error(&mut self, message: String) {
        let start = self.current_offset;
        let text = self.peek_text();
        let len = TextSize::of(text);
        let range = TextRange::at(start, len);
        self.errors.push(SyntaxError { message, range });
    }

    fn parse_element(&mut self) {
        match self.peek() {
            SyntaxKind::Command => self.parse_command_or_environment(),
            SyntaxKind::LBrace => self.parse_group(),
            SyntaxKind::RBrace => {
                self.error("Unmatched '}'".into());
                self.builder.start_node(SyntaxKind::Error.into());
                self.bump();
                self.builder.finish_node();
            }
            SyntaxKind::Eof => {}
            _ => self.bump(),
        }
    }

    fn parse_group(&mut self) {
        self.builder.start_node(SyntaxKind::Group.into());
        self.bump(); // Consume '{'

        while self.peek() != SyntaxKind::Eof && self.peek() != SyntaxKind::RBrace {
            self.parse_element();
        }

        if self.peek() == SyntaxKind::RBrace {
            self.bump(); // Consume '}'
        } else {
            self.error("Expected '}'".into());
        }
        self.builder.finish_node();
    }

    fn parse_command_or_environment(&mut self) {
        let cmd_type = if let Some((SyntaxKind::Command, text)) = self.lexer.peek() {
            match *text {
                "\\begin" => 1,
                "\\section" => 2,
                "\\input" | "\\include" => 3,
                "\\label" => 4,
                "\\ref" => 5,
                "\\cite" => 6,
                "\\bibliography" | "\\addbibresource" => 7,
                _ => 0,
            }
        } else {
            0
        };

        match cmd_type {
            1 => self.parse_environment(),
            2 => self.parse_section(),
            3 => self.parse_include(),
            4 => self.parse_label(),
            5 => self.parse_ref(),
            6 => self.parse_citation(),
            7 => self.parse_bibliography(),
            _ => self.bump(),
        }
    }

    fn parse_citation(&mut self) {
        self.builder.start_node(SyntaxKind::Citation.into());
        self.bump(); // Consume \cite

        // Optional argument [ ... ]
        if self.peek() == SyntaxKind::LBracket {
            self.bump(); // consume [
            while self.peek() != SyntaxKind::Eof && self.peek() != SyntaxKind::RBracket {
                self.parse_element();
            }
            if self.peek() == SyntaxKind::RBracket {
                self.bump(); // consume ]
            } else {
                self.error("Expected ']'".into());
            }
        }

        // Expect {keys}
        if self.peek() == SyntaxKind::LBrace {
            self.parse_group();
        } else {
            self.error("Expected '{' after \\cite".into());
        }

        self.builder.finish_node();
    }

    fn parse_bibliography(&mut self) {
        self.builder.start_node(SyntaxKind::Bibliography.into());
        let is_addbibresource = if let Some((SyntaxKind::Command, text)) = self.lexer.peek() {
            *text == "\\addbibresource"
        } else {
            false
        };
        self.bump(); // Consume command

        // Optional argument [ ... ] (biblatex: \addbibresource[...]{...})
        if is_addbibresource && self.peek() == SyntaxKind::LBracket {
            self.bump(); // consume [
            while self.peek() != SyntaxKind::Eof && self.peek() != SyntaxKind::RBracket {
                self.parse_element();
            }
            if self.peek() == SyntaxKind::RBracket {
                self.bump(); // consume ]
            } else {
                self.error("Expected ']'".into());
            }
        }

        // Expect {path}
        if self.peek() == SyntaxKind::LBrace {
            self.parse_group();
        } else {
            self.error("Expected '{' after bibliography command".into());
        }

        self.builder.finish_node();
    }

    fn parse_label(&mut self) {
        self.builder.start_node(SyntaxKind::LabelDefinition.into());
        self.bump(); // Consume \label

        // Expect {name}
        if self.peek() == SyntaxKind::LBrace {
            self.parse_group();
        } else {
            self.error("Expected '{' after \\label".into());
        }

        self.builder.finish_node();
    }

    fn parse_ref(&mut self) {
        self.builder.start_node(SyntaxKind::LabelReference.into());
        self.bump(); // Consume \ref

        // Expect {name}
        if self.peek() == SyntaxKind::LBrace {
            self.parse_group();
        } else {
            self.error("Expected '{' after \\ref".into());
        }

        self.builder.finish_node();
    }

    fn parse_include(&mut self) {
        self.builder.start_node(SyntaxKind::Include.into());
        self.bump(); // Consume command

        // Expect {path}
        if self.peek() == SyntaxKind::LBrace {
            self.parse_group();
        } else {
            self.error("Expected '{' after include command".into());
        }

        self.builder.finish_node();
    }

    fn parse_section(&mut self) {
        self.builder.start_node(SyntaxKind::Section.into());
        self.bump(); // Consume \section

        // Optional: handle * for \section*?
        // For now, simple \section{...}

        // Expect {Title}
        if self.peek() == SyntaxKind::LBrace {
            self.parse_group();
        } else {
            // Missing title is not a fatal syntax error in terms of structure recovery,
            // but we can flag it.
            self.error("Expected '{' after \\section".into());
        }

        self.builder.finish_node();
    }

    fn parse_environment(&mut self) {
        self.builder.start_node(SyntaxKind::Environment.into());
        self.bump(); // Consume \begin

        // Expect {name}
        if self.peek() == SyntaxKind::LBrace {
            self.parse_group(); // The argument of begin
        } else {
            self.error("Expected '{' after \\begin".into());
        }

        // Parse content until \end
        loop {
            match self.peek() {
                SyntaxKind::Eof => {
                    self.error("Unclosed environment, expected \\end".into());
                    break;
                }
                SyntaxKind::Command => {
                    if let Some((_, text)) = self.lexer.peek() {
                        if *text == "\\end" {
                            self.bump(); // Consume \end
                            if self.peek() == SyntaxKind::LBrace {
                                self.parse_group(); // The argument of end
                            } else {
                                self.error("Expected '{' after \\end".into());
                            }
                            break;
                        } else if *text == "\\begin" {
                            // Nested environment
                            self.parse_environment();
                        } else {
                            self.bump();
                        }
                    } else {
                        self.bump();
                    }
                }
                SyntaxKind::RBrace => {
                    self.error("Unmatched '}' inside environment".into());
                    self.builder.start_node(SyntaxKind::Error.into());
                    // Removed the extra self.builder.finish_node(); here
                }
                _ => self.bump(), // Consume other tokens
            }
        }
        self.builder.finish_node();
    }
}

/// The result of a parse operation.
///
/// Contains the root of the syntax tree and any errors encountered during parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    /// The root node of the CST (Concrete Syntax Tree).
    green_node: GreenNode,
    /// List of syntax errors found during parsing.
    ///
    /// If this list is empty, the document is syntactically valid (though likely
    /// still has semantic errors).
    pub errors: Vec<SyntaxError>,
}

impl ParseResult {
    /// Returns the root [`SyntaxNode`] of the parsed tree.
    ///
    /// This node can be traversed to inspect the structure of the document.
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node.clone())
    }

    /// Returns the raw [`GreenNode`] (internal rowan representation).
    ///
    /// Useful for advanced manipulation or caching.
    pub fn green_node(&self) -> GreenNode {
        self.green_node.clone()
    }
}

/// Parses a LaTeX source string into a syntax tree.
///
/// This is the main entry point for the syntax crate. It performs a complete
/// parse of the input and returns a [`ParseResult`].
///
/// # Arguments
///
/// * `text` - The LaTeX source code to parse
///
/// # Returns
///
/// A [`ParseResult`] containing the syntax tree and any validation errors.
///
/// # Examples
///
/// ```
/// use ferrotex_syntax::parser::parse;
///
/// let tree = parse("x + y");
/// println!("{}", tree.syntax());
/// ```
pub fn parse(text: &str) -> ParseResult {
    Parser::new(text).parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SyntaxKind;

    #[test]
    fn test_parse_group() {
        let input = r"{ \cmd }";
        let parse = parse(input);
        let node = parse.syntax();
        assert_eq!(node.kind(), SyntaxKind::Root);
        let children: Vec<_> = node.children().collect();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].kind(), SyntaxKind::Group);
    }

    #[test]
    fn test_parse_environment() {
        let input = r"\begin{itemize} \item A \end{itemize}";
        let parse = parse(input);
        let node = parse.syntax();
        // Root -> Environment
        let env = node.children().next().unwrap();
        assert_eq!(env.kind(), SyntaxKind::Environment);
    }

    #[test]
    fn test_nested() {
        let input = r"\begin{a} { \begin{b} \end{b} } \end{a}";
        let parse = parse(input);
        assert!(parse.errors.is_empty());
    }

    #[test]
    fn test_errors() {
        let input = r"{ \cmd";
        let parse = parse(input);
        assert!(!parse.errors.is_empty());
        assert_eq!(parse.errors[0].message, "Expected '}'");
        // range should be at EOF
        // offset of "{" is 0, len 1. " " is 1, len 1. "\cmd" is 2, len 4.
        // Total len 6.
        // Expected '}' at EOF.
        assert_eq!(parse.errors[0].range.start(), TextSize::from(6));
    }

    #[test]
    fn test_section() {
        let input = r"\section{Introduction}";
        let parse = parse(input);
        let node = parse.syntax();
        // Root -> Section
        let section = node.children().next().unwrap();
        assert_eq!(section.kind(), SyntaxKind::Section);

        // Check children of section (should be \section token and Group)
        // Note: Rowan children() only yields Nodes, not Tokens.
        let group = section.children().next().unwrap();
        assert_eq!(group.kind(), SyntaxKind::Group);
    }

    #[test]
    fn test_include() {
        let input = r"\input{chapters/intro}";
        let result = parse(input);
        let node = result.syntax();
        let include = node.children().next().unwrap();
        assert_eq!(include.kind(), SyntaxKind::Include);

        let input2 = r"\include{chapters/concl}";
        let result2 = parse(input2);
        let node2 = result2.syntax();
        let include2 = node2.children().next().unwrap();
        assert_eq!(include2.kind(), SyntaxKind::Include);
    }

    #[test]
    fn test_labels_refs() {
        let input = r"\label{fig1} \ref{fig1}";
        let _ = parse(input);
    }

    #[test]
    fn test_parser_citation_recovery() {
        let input = r"\cite{ \end{document}"; // Malformed
        let res = parse(input);
        assert!(!res.errors.is_empty());
    }

    #[test]
    fn test_parser_bib_recovery() {
        let input = r"\bibliography{ \end{document}"; // Malformed
        let res = parse(input);
        assert!(!res.errors.is_empty());
    }

    #[test]
    fn test_parser_math_unclosed() {
        let input = r"$ x + y"; // Unclosed dollar
        let res = parse(input);
        // Our current parser might just treat it as a dollar token? 
        // Need to check if it emits an error.
    }

    #[test]
    fn test_citation() {
        let input = r"\cite{key1,key2} \cite[p. 23]{key3}";
        let parse = parse(input);
        let node = parse.syntax();
        let children: Vec<_> = node.children().collect();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].kind(), SyntaxKind::Citation);
        assert_eq!(children[1].kind(), SyntaxKind::Citation);
    }

    #[test]
    fn test_bibliography() {
        let input = r"\bibliography{refs} \addbibresource[backend=biber]{refs.bib}";
        let parse = parse(input);
        let node = parse.syntax();
        let children: Vec<_> = node.children().collect();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].kind(), SyntaxKind::Bibliography);
        assert_eq!(children[1].kind(), SyntaxKind::Bibliography);
    }
}
