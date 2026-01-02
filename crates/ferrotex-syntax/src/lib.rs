//! # FerroTeX Syntax
//!
//! Lexer, parser, and lossless syntax tree implementation for LaTeX source code.
//!
//! ## Overview
//!
//! This crate provides a **fault-tolerant** parser for LaTeX documents that produces
//! a **lossless Concrete Syntax Tree (CST)** using the [`rowan`](https://github.com/rust-analyzer/rowan)
//! library. The parser is designed to handle incomplete, malformed, or evolving documents
//! gracefully, making it suitable for IDE use cases where the source is being actively edited.
//!
//! ## Architecture
//!
//! The parsing pipeline consists of three main components:
//!
//! ```text
//! ┌──────────┐      ┌────────┐      ┌─────────────┐
//! │  Source  │ ───► │ Lexer  │ ───► │   Parser    │
//! │  (str)   │      │        │      │             │
//! └──────────┘      └────────┘      └─────────────┘
//!                       │                   │
//!                       ▼                   ▼
//!                 SyntaxKind           GreenNode
//!                   tokens              (CST)
//! ```
//!
//! ### Component Responsibilities
//!
//! - **[`lexer`]** - Tokenizes LaTeX source into [`SyntaxKind`] tokens
//! - **[`parser`]** - Builds a CST using recursive descent parsing
//! - **[`bibtex`]** - Specialized parsing for BibTeX bibliography files
//!
//! ## Design Principles
//!
//! ### 1. Lossless Representation
//!
//! The syntax tree preserves **all** source information including:
//! - Whitespace
//! - Comments
//! - Malformed or unrecognized tokens
//!
//! This enables precise source mapping, formatting preservation, and reliable
//! round-tripping (`parse(source).to_string() == source`).
//!
//! ### 2. Error Tolerance
//!
//! The parser continues after encountering errors, producing a best-effort tree plus
//! a list of [`parser::SyntaxError`]s. This ensures IDE features work even in incomplete
//! documents.
//!
//! ### 3. Incremental Updates
//!
//! The `rowan` CST is designed for incremental re-parsing (not yet fully implemented),
//! allowing efficient updates when small portions of the document change.
//!
//! ## Key Types
//!
//! - [`SyntaxKind`] - Enumeration of all token and node types
//! - [`SyntaxNode`] - A node in the concrete syntax tree
//! - [`SyntaxToken`] - A terminal token (leaf) in the CST
//! - [`parser::Parser`] - The main parsing entry point
//! - [`parser::ParseResult`] - Contains the CST and any errors
//!
//! ## Examples
//!
//! ### Basic Parsing
//!
//! ```
//! use ferrotex_syntax::parse;
//!
//! let source = r"\section{Introduction}
//! This is a \textbf{LaTeX} document.
//! ";
//!
//! let result = parse(source);
//! let root = result.syntax();
//!
//! // Check for parse errors
//! if result.errors.is_empty() {
//!     println!("Parse successful!");
//! } else {
//!     for error in &result.errors {
//!         eprintln!("Error: {} at {:?}", error.message, error.range);
//!     }
//! }
//! ```
//!
//! ### Traversing the Syntax Tree
//!
//! ```
//! use ferrotex_syntax::{parse, SyntaxKind};
//!
//! let result = parse(r"\section{Intro} \label{sec:intro}");
//! let root = result.syntax();
//!
//! // Find all section nodes
//! for child in root.children() {
//!     if child.kind() == SyntaxKind::Section {
//!         println!("Found section at: {:?}", child.text_range());
//!     }
//! }
//! ```
//!
//! ### Using the Lexer Directly
//!
//! ```
//! use ferrotex_syntax::lexer::Lexer;
//!
//! let source = r"\section{Hello}";
//! let tokens: Vec<_> = Lexer::new(source).collect();
//!
//! for (kind, text) in tokens {
//!     println!("{:?}: {:?}", kind, text);
//! }
//! ```
//!
//! ## Rowan Integration
//!
//! This crate uses the [`rowan`](https://github.com/rust-analyzer/rowan) library,
//! originally developed for rust-analyzer. Rowan provides:
//!
//! - **Immutable, persistent trees** with structural sharing
//! - **Red-green tree architecture** separating syntax (green) from parent pointers (red)
//! - **Zero-copy text representation** via interned strings
//!
//! See [`FerroTexLanguage`] for the language implementation that connects [`SyntaxKind`]
//! to rowan's generic tree types.

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
