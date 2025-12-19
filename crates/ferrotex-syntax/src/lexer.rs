use crate::SyntaxKind;

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
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
                        '\\' | '{' | '}' | '[' | ']' | '%' => break,
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
}
