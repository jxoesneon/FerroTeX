//! LaTeX mathematical expression validation utilities.
//!
//! ## Overview
//!
//! This module provides tools for validating LaTeX mathematical expressions, focusing
//! on delimiter matching and command argument validation. The validators help catch
//! common LaTeX math errors early in the editing process, enabling better IDE
//! diagnostics.
//!
//! ## Key Components
//!
//! - [`DelimiterKind`] - Enumeration of supported delimiter types
//! - [`Delimiter`] - Represents a delimiter occurrence in source text
//! - [`MathError`] - Validation errors with diagnostic information
//! - [`DelimiterValidator`] - Stack-based delimiter matching validator
//!
//! ## Delimiter Matching Algorithm
//!
//! The [`DelimiterValidator`] uses a **stack-based matching algorithm** similar to
//! balanced parenthesis validation:
//!
//! 1. Push opening delimiters (`(`, `[`, `{`, `\langle`, etc.) onto a stack
//! 2. When a closing delimiter is encountered, pop the stack and verify it matches
//! 3. Report errors for:
//!    - Mismatched pairs (e.g., `(` paired with `]`)
//!    - Unmatched closing delimiters (stack empty when `)` is found)
//!    - Unmatched opening delimiters (stack non-empty at end)
//!
//! ### LaTeX-Specific Considerations
//!
//! LaTeX math supports both bare delimiters `()[]{}` and sized delimiters via
//! `\left(` and `\right)` commands. The `is_left_command` field in [`Delimiter`]
//! tracks this distinction for future enhancement (e.g., enforcing that `\left(`
//! is paired with `\right)`, not just bare `)`).
//!
//! ## Command Argument Validation
//!
//! The [`get_expected_args`] function provides expected argument counts for common
//! math commands like `\frac{}{}, `\sqrt{}`, `\binom{}{}`. This enables diagnostics
//! like "command `\frac` expects 2 arguments but got 1".
//!
//! ## Examples
//!
//! ### Validating Delimiter Balance
//!
//! ```
//! use ferrotex_core::math_validator::{Delimiter, DelimiterKind, DelimiterValidator};
//!
//! let delimiters = vec![
//!     Delimiter {
//!         kind: DelimiterKind::LeftParen,
//!         position: 0,
//!         is_left_command: false,
//!     },
//!     Delimiter {
//!         kind: DelimiterKind::RightParen,
//!         position: 10,
//!         is_left_command: false,
//!     },
//! ];
//!
//! let mut validator = DelimiterValidator::new();
//! validator.validate(&delimiters);
//!
//! if validator.has_errors() {
//!     for error in validator.errors() {
//!         eprintln!("{}", error.to_diagnostic_message());
//!     }
//! }
//! ```
//!
//! ### Checking Expected Argument Counts
//!
//! ```
//! use ferrotex_core::math_validator::get_expected_args;
//!
//! assert_eq!(get_expected_args("frac"), Some(2));
//! assert_eq!(get_expected_args("sqrt"), Some(1));
//! assert_eq!(get_expected_args("unknown"), None);
//! ```

use std::collections::HashMap;

/// Represents a mathematical delimiter type in LaTeX.
///
/// This enum covers the common delimiter types used in LaTeX math mode,
/// including both ASCII characters and LaTeX commands.
#[derive(Debug, Clone, PartialEq)]
pub enum DelimiterKind {
    /// Opening parenthesis `(`
    LeftParen,
    /// Closing parenthesis `)`
    RightParen,
    /// Opening bracket `[`
    LeftBracket,
    /// Closing bracket `]`
    RightBracket,
    /// Opening brace `{`
    LeftBrace,
    /// Closing brace `}`
    RightBrace,
    /// Opening angle bracket `\langle`
    LeftAngle,
    /// Closing angle bracket `\rangle`
    RightAngle,
    /// Opening floor `\lfloor`
    LeftFloor,
    /// Closing floor `\rfloor`
    RightFloor,
    /// Opening ceiling `\lceil`
    LeftCeil,
    /// Closing ceiling `\rceil`
    RightCeil,
}

/// A delimiter occurrence in LaTeX source code.
///
/// This struct captures the location and type of a delimiter found during parsing.
#[derive(Debug, Clone)]
pub struct Delimiter {
    /// The type of delimiter (opening or closing, and which kind).
    pub kind: DelimiterKind,
    /// Byte offset position in the source text.
    pub position: usize,
    /// Whether this delimiter was created with `\left` or `\right` commands.
    ///
    /// LaTeX supports sized delimiters via `\left(` and `\right)`. This field
    /// tracks whether the delimiter is part of such a pair, which may be used
    /// for enhanced validation in the future (e.g., enforcing that `\left(`
    /// must be paired specifically with `\right)`).
    pub is_left_command: bool,
}

/// Returns the expected number of arguments for common LaTeX math commands.
///
/// This function provides argument count information for frequently-used math
/// commands, enabling validation diagnostics like "command `\frac` expects 2
/// arguments but got 1".
///
/// # Arguments
///
/// * `command` - The command name without the leading backslash (e.g., "frac", "sqrt")
///
/// # Returns
///
/// - `Some(n)` if the command is recognized, where `n` is the expected argument count
/// - `None` if the command is not in the known set
///
/// # Examples
///
/// ```
/// use ferrotex_core::math_validator::get_expected_args;
///
/// assert_eq!(get_expected_args("frac"), Some(2));  // \frac{num}{denom}
/// assert_eq!(get_expected_args("sqrt"), Some(1));  // \sqrt{expr}
/// assert_eq!(get_expected_args("customcmd"), None); // Unknown command
/// ```
///
/// # Note
///
/// Some commands like `\sqrt` accept optional arguments (e.g., `\sqrt[n]{x}`), but
/// this function returns only the *required* argument count.
pub fn get_expected_args(command: &str) -> Option<usize> {
    let mut map = HashMap::new();

    // Fractions and binomials
    map.insert("frac", 2);
    map.insert("dfrac", 2);
    map.insert("tfrac", 2);
    map.insert("cfrac", 2);
    map.insert("binom", 2);
    map.insert("dbinom", 2);
    map.insert("tbinom", 2);

    // Roots
    map.insert("sqrt", 1); // Note: \sqrt can have optional argument

    // Text in math
    map.insert("text", 1);
    map.insert("mathrm", 1);
    map.insert("mathbf", 1);
    map.insert("mathit", 1);
    map.insert("mathsf", 1);
    map.insert("mathtt", 1);
    map.insert("mathcal", 1);
    map.insert("mathbb", 1);
    map.insert("mathfrak", 1);

    // Operators
    map.insert("overline", 1);
    map.insert("underline", 1);
    map.insert("hat", 1);
    map.insert("tilde", 1);
    map.insert("bar", 1);
    map.insert("vec", 1);
    map.insert("dot", 1);
    map.insert("ddot", 1);

    map.get(command).copied()
}

/// Check if delimiters match correctly
pub fn delimiters_match(left: &DelimiterKind, right: &DelimiterKind) -> bool {
    matches!(
        (left, right),
        (DelimiterKind::LeftParen, DelimiterKind::RightParen)
            | (DelimiterKind::LeftBracket, DelimiterKind::RightBracket)
            | (DelimiterKind::LeftBrace, DelimiterKind::RightBrace)
            | (DelimiterKind::LeftAngle, DelimiterKind::RightAngle)
            | (DelimiterKind::LeftFloor, DelimiterKind::RightFloor)
            | (DelimiterKind::LeftCeil, DelimiterKind::RightCeil)
    )
}

/// Math validation error
#[derive(Debug, Clone)]
pub enum MathError {
    /// Found a closing delimiter that doesn't match the opening one (e.g., `( ]`).
    MismatchedDelimiter {
        /// Position of the opening delimiter.
        left_pos: usize,
        /// Position of the closing delimiter.
        right_pos: usize,
        /// Type of the opening delimiter.
        left_kind: DelimiterKind,
        /// Type of the closing delimiter.
        right_kind: DelimiterKind,
    },
    /// Found an opening delimiter at the end with no matching closer.
    UnmatchedOpening {
        /// Position of the opening delimiter.
        pos: usize,
        /// Type of the opening delimiter.
        kind: DelimiterKind,
    },
    /// Found a closing delimiter without a preceding opening one.
    UnmatchedClosing {
        /// Position of the closing delimiter.
        pos: usize,
        /// Type of the closing delimiter.
        kind: DelimiterKind,
    },
    /// A command was called with the wrong number of arguments.
    IncorrectArgumentCount {
        /// Name of the command (e.g., "frac").
        command: String,
        /// Position of the command.
        position: usize,
        /// Number of arguments expected.
        expected: usize,
        /// Number of arguments actually provided.
        actual: usize,
    },
}

impl MathError {
    /// Converts the error into a human-readable diagnostic message.
    pub fn to_diagnostic_message(&self) -> String {
        match self {
            MathError::MismatchedDelimiter {
                left_kind,
                right_kind,
                ..
            } => {
                format!(
                    "Mismatched delimiters: {:?} paired with {:?}",
                    left_kind, right_kind
                )
            }
            MathError::UnmatchedOpening { kind, .. } => {
                format!("Unmatched opening delimiter: {:?}", kind)
            }
            MathError::UnmatchedClosing { kind, .. } => {
                format!("Unmatched closing delimiter: {:?}", kind)
            }
            MathError::IncorrectArgumentCount {
                command,
                expected,
                actual,
                ..
            } => {
                format!(
                    "Command '\\{}' expects {} argument(s) but got {}",
                    command, expected, actual
                )
            }
        }
    }
}

/// Delimiter validation logic.
pub mod delimiter_validator;

#[cfg(test)]
mod tests;

pub use delimiter_validator::DelimiterValidator;
