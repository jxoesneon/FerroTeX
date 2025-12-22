//! # FerroTeX Syntax
//!
//! This crate provides the syntax tree definition, lexer, and parser for the FerroTeX project.
//! It is based on `rowan` for lossless syntax trees.

pub mod bibtex;
pub mod lexer;
pub mod parser;

#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod additional_tests;

pub use parser::parse;
use rowan::Language;
pub use rowan::{TextRange, TextSize};

/// Syntax kinds for FerroTeX.
///
/// This enum defines all possible tokens and composite nodes in the syntax tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    // Tokens
    /// Left brace `{`
    LBrace = 0,
    /// Right brace `}`
    RBrace,
    /// Left bracket `[`
    LBracket,
    /// Right bracket `]`
    RBracket,
    /// A LaTeX command starting with `\` (e.g., `\section`, `\item`, `\%`)
    Command, // \section, \item
    /// Dollar sign `$` for inline math
    Dollar, // $
    /// Whitespace characters
    Whitespace,
    /// Comment starting with `%`
    Comment, // % ...
    /// Regular text content
    Text, // Regular text
    /// Lexer error token
    Error, // Lexer error

    // Composite Nodes
    /// The root node of the syntax tree
    Root,
    /// A group enclosed in braces `{ ... }`
    Group, // { ... }
    /// An environment block `\begin{...} ... \end{...}`
    Environment, // \begin{...} ... \end{...}
    /// A section command (e.g., `\section{...}`)
    Section, // \section{...} (heuristic)
    /// An include command (e.g., `\input{...}`, `\include{...}`)
    Include, // \input{...}, \include{...}
    /// A label definition `\label{...}`
    LabelDefinition, // \label{...}
    /// A label reference `\ref{...}`
    LabelReference, // \ref{...}
    /// A citation `\cite{...}`
    Citation, // \cite{...}
    /// A bibliography command `\bibliography{...}` or `\addbibresource{...}`
    Bibliography, // \bibliography{...}, \addbibresource{...}

    // Technical
    /// End of file
    Eof,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

/// The FerroTeX language definition for `rowan`.
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

/// A syntax node in the FerroTeX language.
pub type SyntaxNode = rowan::SyntaxNode<FerroTexLanguage>;
/// A syntax token in the FerroTeX language.
pub type SyntaxToken = rowan::SyntaxToken<FerroTexLanguage>;
/// A syntax element (node or token) in the FerroTeX language.
pub type SyntaxElement = rowan::SyntaxElement<FerroTexLanguage>;
