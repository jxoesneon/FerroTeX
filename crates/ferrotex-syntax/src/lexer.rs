use crate::SyntaxKind;

/// A lexer for LaTeX source code.
///
/// ## Overview
///
/// The lexer performs **character-level scanning** of LaTeX source, producing
/// a stream of ([`SyntaxKind`], `&str`) tuples. It handles:
///
/// - **Commands**: `\section`, `\item`, `\%` (escape sequences)
/// - **Delimiters**: `{`, `}`, `[`, `]`
/// - **Math mode**: `$` (inline math delimiter)
/// - **Comments**: `%` through end of line
/// - **Whitespace**: Consecutive whitespace collapsed into single tokens
/// - **Text**: Everything else, consumed greedily until a special character
///
/// ## UTF-8 Handling
///
/// The lexer is **fully UTF-8 aware**, correctly handling multi-byte characters
/// in commands, text, and comments. Position tracking uses byte offsets internally
/// but respects character boundaries.
///
/// ## Performance Characteristics
///
/// - **Single-pass**: O(n) time complexity where n is source length
/// - **Zero-copy**: Returns `&str` slices into the original source
/// - **Lazy**: Implemented as an iterator, tokens generated on demand
///
/// ## Examples
///
/// ### Basic Tokenization
///
/// ```
/// use ferrotex_syntax::lexer::Lexer;
/// use ferrotex_syntax::SyntaxKind;
///
/// let source = r"\section{Hello} % comment";
/// let tokens: Vec<_> = Lexer::new(source).collect();
///
/// assert_eq!(tokens[0].0, SyntaxKind::Command); // \section
/// assert_eq!(tokens[1].0, SyntaxKind::LBrace);  // {
/// assert_eq!(tokens[2].0, SyntaxKind::Text);    // Hello
/// ```
///
/// ### Handling Multi-byte UTF-8
///
/// ```
/// use ferrotex_syntax::lexer::Lexer;
///
/// let source = r"Émilie Noether's theorem";
/// let mut lexer = Lexer::new(source);
///
/// let (kind, text) = lexer.next().unwrap();
/// assert_eq!(text, "Émilie"); // Correctly handles é
/// ```
pub struct Lexer<'a> {
    /// The input source text being lexed.
    input: &'a str,
    /// Current byte position in the input.
    position: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new `Lexer` for the given input string.
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    /// Returns the next token (kind, text).
    /// If EOF, returns (SyntaxKind::Eof, "").
    pub fn next_token(&mut self) -> (SyntaxKind, &'a str) {
        if self.position >= self.input.len() {
            return (SyntaxKind::Eof, "");
        }

        let start = self.position;
        let rest = &self.input[start..];
        let mut chars = rest.chars();
        let c = chars.next().unwrap();

        let kind = match c {
            '\\' => {
                // Command
                self.position += c.len_utf8();
                if let Some(next) = chars.next() {
                    if next.is_alphabetic() {
                        // Multi-letter command: \section
                        self.position += next.len_utf8();
                        while let Some(n) = self.input[self.position..].chars().next() {
                            if n.is_alphabetic() {
                                self.position += n.len_utf8();
                            } else {
                                break;
                            }
                        }
                    } else {
                        // Single-symbol command: \$ or \_
                        self.position += next.len_utf8();
                    }
                }
                SyntaxKind::Command
            }
            '{' => {
                self.position += c.len_utf8();
                SyntaxKind::LBrace
            }
            '}' => {
                self.position += c.len_utf8();
                SyntaxKind::RBrace
            }
            '[' => {
                self.position += c.len_utf8();
                SyntaxKind::LBracket
            }
            ']' => {
                self.position += c.len_utf8();
                SyntaxKind::RBracket
            }
            '$' => {
                self.position += c.len_utf8();
                SyntaxKind::Dollar
            }
            '%' => {
                // Comment
                self.position += c.len_utf8();
                while let Some(n) = self.input[self.position..].chars().next() {
                    if n == '\n' || n == '\r' {
                        break;
                    }
                    self.position += n.len_utf8();
                }
                SyntaxKind::Comment
            }
            c if c.is_whitespace() => {
                self.position += c.len_utf8();
                while let Some(n) = self.input[self.position..].chars().next() {
                    if n.is_whitespace() {
                        self.position += n.len_utf8();
                    } else {
                        break;
                    }
                }
                SyntaxKind::Whitespace
            }
            _ => {
                // Text run
                self.position += c.len_utf8();
                while let Some(n) = self.input[self.position..].chars().next() {
                    match n {
                        '\\' | '{' | '}' | '[' | ']' | '%' | '$' => break,
                        c if c.is_whitespace() => break,
                        _ => self.position += n.len_utf8(),
                    }
                }
                SyntaxKind::Text
            }
        };

        (kind, &self.input[start..self.position])
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (SyntaxKind, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let (kind, text) = self.next_token();
        if kind == SyntaxKind::Eof {
            None
        } else {
            Some((kind, text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<(SyntaxKind, &str)> {
        let lexer = Lexer::new(input);
        lexer.collect()
    }

    #[test]
    fn test_basic_tokens() {
        let input = r"\section{Hello} % comment";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                (SyntaxKind::Command, "\\section"),
                (SyntaxKind::LBrace, "{"),
                (SyntaxKind::Text, "Hello"),
                (SyntaxKind::RBrace, "}"),
                (SyntaxKind::Whitespace, " "),
                (SyntaxKind::Comment, "% comment"),
            ]
        );
    }

    #[test]
    fn test_escaped_symbols() {
        let input = r"Wait 50\%";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                (SyntaxKind::Text, "Wait"),
                (SyntaxKind::Whitespace, " "),
                (SyntaxKind::Text, "50"),
                (SyntaxKind::Command, "\\%"),
            ]
        );
    }

    #[test]
    fn test_lexer_empty_input() {
        let input = "";
        let tokens = tokenize(input);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_lexer_only_whitespace() {
        let input = "   \n\t ";
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![(SyntaxKind::Whitespace, "   \n\t ")]);
    }

    #[test]
    fn test_lexer_unexpected_chars() {
        // Technically nothing is unexpected in our lexer yet (it falls back to text),
        // but this verifies that behavior.
        let input = "@#*&^";
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![(SyntaxKind::Text, "@#*&^")]);
    }

    #[test]
    fn test_lexer_mixed_math_and_text() {
        let input = "a$b$c";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                (SyntaxKind::Text, "a"),
                (SyntaxKind::Dollar, "$"),
                (SyntaxKind::Text, "b"),
                (SyntaxKind::Dollar, "$"),
                (SyntaxKind::Text, "c"),
            ]
        );
    }

    #[test]
    fn test_lexer_brackets() {
        let input = r"[arg]";
        let tokens = tokenize(input);
        assert_eq!(
            tokens,
            vec![
                (SyntaxKind::LBracket, "["),
                (SyntaxKind::Text, "arg"),
                (SyntaxKind::RBracket, "]"),
            ]
        );
    }

    #[test]
    fn test_lexer_multi_byte_text() {
        let input = "Étude";
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![(SyntaxKind::Text, "Étude")]);
    }

    #[test]
    fn test_lexer_comment_with_carriage_return() {
        let input = "% comment\rnext";
        let tokens = tokenize(input);
        assert_eq!(tokens[0], (SyntaxKind::Comment, "% comment"));
    }
}
