pub mod lexer;
pub mod parser;

pub use parser::parse;
use rowan::Language;
pub use rowan::{TextRange, TextSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    // Tokens
    LBrace = 0,
    RBrace,
    LBracket,
    RBracket,
    Command, // \section, \item
    Whitespace,
    Comment, // % ...
    Text,    // Regular text
    Error,   // Lexer error

    // Composite Nodes
    Root,
    Group,           // { ... }
    Environment,     // \begin{...} ... \end{...}
    Section,         // \section{...} (heuristic)
    Include,         // \input{...}, \include{...}
    LabelDefinition, // \label{...}
    LabelReference,  // \ref{...}

    // Technical
    Eof,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FerroTexLanguage {}

impl Language for FerroTexLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::Eof as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<FerroTexLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<FerroTexLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<FerroTexLanguage>;
