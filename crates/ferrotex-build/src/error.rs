//! # FerroTeX Error Handling
//!
//! This module provides a unified error handling system for all FerroTeX crates.
//! It replaces the scattered error handling approaches (anyhow, custom errors, etc.)
//! with a single, consistent error type that supports rich context and source locations.
//!
//! ## When to Use Each Variant
//!
//! | Variant | Use When | Example |
//! |---------|----------|---------|
//! | `ParseError` | LaTeX syntax parsing fails | Missing closing brace, invalid command |
//! | `AnalysisError` | Semantic analysis fails | Undefined reference, type mismatch |
//! | `IoError` | File system operations fail | Cannot read .tex file, permission denied |
//! | `ConfigurationError` | Invalid configuration | Missing environment variable, bad config file |
//! | `GenericError` | Everything else | Internal invariant violation |
//!
//! ## Type Aliases
//!
//! - `FerroTeXResult<T>` - Use this instead of `Result<T, FerroTeXError>`
//!
//! ## Conversions
//!
//! The error type automatically converts from common error types:
//! - `std::io::Error` → `FerroTeXError::IoError`
//! - `anyhow::Error` → `FerroTeXError::GenericError`
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use ferrotex_build::error::{FerroTeXError, FerroTeXResult, SourceLocation};
//!
//! fn parse_latex(input: &str) -> FerroTeXResult<ast::Document> {
//!     // On parse failure, include source location
//!     if input.is_empty() {
//!         return Err(FerroTeXError::parse_error(
//!             "Empty document",
//!             SourceLocation::new(1, 1),
//!         ));
//!     }
//!     Ok(ast::Document::new())
//! }
//!
//! fn read_file(path: &str) -> FerroTeXResult<String> {
//!     // std::io::Error converts automatically
//!     std::fs::read_to_string(path)?
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{self, Display};
use std::io;

/// Represents a location in the source code.
///
/// This is used by parse and analysis errors to pinpoint where
/// an issue occurred. Lines and columns are 1-indexed for user-friendliness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    /// 1-indexed line number
    pub line: usize,
    /// 1-indexed column number
    pub column: usize,
}

impl SourceLocation {
    /// Create a new source location.
    ///
    /// # Arguments
    ///
    /// * `line` - 1-indexed line number
    /// * `column` - 1-indexed column number
    ///
    /// # Example
    ///
    /// ```
    /// use ferrotex_build::error::SourceLocation;
    ///
    /// let loc = SourceLocation::new(5, 12);
    /// assert_eq!(loc.line, 5);
    /// assert_eq!(loc.column, 12);
    /// ```
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Create a location for the start of a document (line 1, column 1).
    pub fn start() -> Self {
        Self { line: 1, column: 1 }
    }
}

impl Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A source span representing a range in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceSpan {
    /// Start location (inclusive)
    pub start: SourceLocation,
    /// End location (exclusive)
    pub end: SourceLocation,
}

impl SourceSpan {
    /// Create a new source span from start and end locations.
    pub fn new(start: SourceLocation, end: SourceLocation) -> Self {
        Self { start, end }
    }

    /// Create a single-point span at the given location.
    pub fn point(loc: SourceLocation) -> Self {
        Self { start: loc, end: loc }
    }
}

impl Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            write!(f, "at {}", self.start)
        } else {
            write!(f, "from {} to {}", self.start, self.end)
        }
    }
}

/// Context information for analysis errors.
///
/// Analysis errors often require additional context to help users understand
/// why something failed. This struct holds that contextual information.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnalysisContext {
    /// The name of the analysis pass that failed
    pub pass: String,
    /// Additional context about what was being analyzed
    pub context: String,
    /// Optional hint for fixing the issue
    pub hint: Option<String>,
}

impl AnalysisContext {
    /// Create a new analysis context.
    pub fn new(pass: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            pass: pass.into(),
            context: context.into(),
            hint: None,
        }
    }

    /// Add a hint to the context (builder pattern).
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// Configuration error details.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigurationErrorDetails {
    /// The configuration key that caused the error
    pub key: String,
    /// The expected format or type
    pub expected: String,
    /// The actual value received (if any)
    pub actual: Option<String>,
}

impl ConfigurationErrorDetails {
    /// Create new configuration error details.
    pub fn new(key: impl Into<String>, expected: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            expected: expected.into(),
            actual: None,
        }
    }

    /// Add the actual value (builder pattern).
    pub fn with_actual(mut self, actual: impl Into<String>) -> Self {
        self.actual = Some(actual.into());
        self
    }
}

/// The unified error type for all FerroTeX operations.
///
/// This enum encompasses all error types that can occur in the FerroTeX
/// ecosystem. Each variant carries specific context relevant to that
/// error type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FerroTeXError {
    /// A syntax or parsing error with an associated source location.
    ///
    /// Use this when LaTeX source code cannot be parsed correctly.
    /// The location should point to where the parser failed.
    ParseError {
        message: String,
        location: SourceLocation,
        /// Optional snippet of source code around the error
        snippet: Option<String>,
    },

    /// A semantic analysis error with context about what was being analyzed.
    ///
    /// Use this when parsed code is semantically invalid (e.g., undefined
    /// references, type mismatches).
    AnalysisError {
        message: String,
        context: AnalysisContext,
        /// Optional source location
        location: Option<SourceLocation>,
    },

    /// An I/O error from file system operations.
    ///
    /// Use this when file reading, writing, or other I/O operations fail.
    /// Automatically converts from `std::io::Error`.
    IoError {
        message: String,
        path: Option<String>,
    },

    /// A configuration error.
    ///
    /// Use this when configuration files are invalid, environment variables
    /// are missing, or other configuration issues occur.
    ConfigurationError {
        message: String,
        details: ConfigurationErrorDetails,
    },

    /// A generic error for cases not covered by other variants.
    ///
    /// Use this sparingly - prefer specific error variants when possible.
    /// This is primarily for internal invariant violations and unexpected errors.
    GenericError {
        message: String,
        /// Optional source location if relevant
        location: Option<SourceLocation>,
    },
}

impl FerroTeXError {
    /// Create a parse error at the given location.
    ///
    /// # Example
    ///
    /// ```
    /// use ferrotex_build::error::{FerroTeXError, SourceLocation};
    ///
    /// let err = FerroTeXError::parse_error("Unexpected token", SourceLocation::new(5, 3));
    /// ```
    pub fn parse_error(message: impl Into<String>, location: SourceLocation) -> Self {
        Self::ParseError {
            message: message.into(),
            location,
            snippet: None,
        }
    }

    /// Create a parse error with a source snippet.
    pub fn parse_error_with_snippet(
        message: impl Into<String>,
        location: SourceLocation,
        snippet: impl Into<String>,
    ) -> Self {
        Self::ParseError {
            message: message.into(),
            location,
            snippet: Some(snippet.into()),
        }
    }

    /// Create an analysis error.
    ///
    /// # Example
    ///
    /// ```
    /// use ferrotex_build::error::{FerroTeXError, AnalysisContext};
    ///
    /// let ctx = AnalysisContext::new("type_check", "checking equation on line 5");
    /// let err = FerroTeXError::analysis_error("Type mismatch", ctx);
    /// ```
    pub fn analysis_error(message: impl Into<String>, context: AnalysisContext) -> Self {
        Self::AnalysisError {
            message: message.into(),
            context,
            location: None,
        }
    }

    /// Create an analysis error with a location.
    pub fn analysis_error_with_location(
        message: impl Into<String>,
        context: AnalysisContext,
        location: SourceLocation,
    ) -> Self {
        Self::AnalysisError {
            message: message.into(),
            context,
            location: Some(location),
        }
    }

    /// Create an I/O error.
    ///
    /// # Example
    ///
    /// ```
    /// use ferrotex_build::error::FerroTeXError;
    ///
    /// let err = FerroTeXError::io_error("Failed to read file").with_path("/path/to/file.tex");
    /// ```
    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError {
            message: message.into(),
            path: None,
        }
    }

    /// Add a path to an I/O error (builder pattern).
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        if let Self::IoError { path: ref mut p, .. } = self {
            *p = Some(path.into());
        }
        self
    }

    /// Create a configuration error.
    ///
    /// # Example
    ///
    /// ```
    /// use ferrotex_build::error::{FerroTeXError, ConfigurationErrorDetails};
    ///
    /// let details = ConfigurationErrorDetails::new("output_format", "pdf or dvi");
    /// let err = FerroTeXError::configuration_error("Invalid output format", details);
    /// ```
    pub fn configuration_error(
        message: impl Into<String>,
        details: ConfigurationErrorDetails,
    ) -> Self {
        Self::ConfigurationError {
            message: message.into(),
            details,
        }
    }

    /// Create a generic error.
    ///
    /// # Example
    ///
    /// ```
    /// use ferrotex_build::error::FerroTeXError;
    ///
    /// let err = FerroTeXError::generic_error("Internal invariant violated");
    /// ```
    pub fn generic_error(message: impl Into<String>) -> Self {
        Self::GenericError {
            message: message.into(),
            location: None,
        }
    }

    /// Add a location to a generic error (builder pattern).
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        if let Self::GenericError { location: ref mut loc, .. } = self {
            *loc = Some(location);
        }
        self
    }

    /// Get the error message.
    pub fn message(&self) -> &str {
        match self {
            Self::ParseError { message, .. } => message,
            Self::AnalysisError { message, .. } => message,
            Self::IoError { message, .. } => message,
            Self::ConfigurationError { message, .. } => message,
            Self::GenericError { message, .. } => message,
        }
    }

    /// Get the source location if available.
    pub fn location(&self) -> Option<SourceLocation> {
        match self {
            Self::ParseError { location, .. } => Some(*location),
            Self::AnalysisError { location, .. } => *location,
            Self::GenericError { location, .. } => *location,
            _ => None,
        }
    }

    /// Check if this is a parse error.
    pub fn is_parse_error(&self) -> bool {
        matches!(self, Self::ParseError { .. })
    }

    /// Check if this is an analysis error.
    pub fn is_analysis_error(&self) -> bool {
        matches!(self, Self::AnalysisError { .. })
    }

    /// Check if this is an I/O error.
    pub fn is_io_error(&self) -> bool {
        matches!(self, Self::IoError { .. })
    }

    /// Check if this is a configuration error.
    pub fn is_configuration_error(&self) -> bool {
        matches!(self, Self::ConfigurationError { .. })
    }

    /// Check if this is a generic error.
    pub fn is_generic_error(&self) -> bool {
        matches!(self, Self::GenericError { .. })
    }
}

impl Display for FerroTeXError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError {
                message,
                location,
                snippet: None,
            } => {
                write!(f, "Parse error at {}: {}", location, message)
            }
            Self::ParseError {
                message,
                location,
                snippet: Some(snippet),
            } => {
                write!(
                    f,
                    "Parse error at {}: {}\n  near: {}",
                    location, message, snippet
                )
            }
            Self::AnalysisError {
                message,
                context,
                location: None,
            } => {
                write!(
                    f,
                    "Analysis error in {}: {}\n  context: {}",
                    context.pass, message, context.context
                )
            }
            Self::AnalysisError {
                message,
                context,
                location: Some(loc),
            } => {
                write!(
                    f,
                    "Analysis error at {} in {}: {}\n  context: {}",
                    loc, context.pass, message, context.context
                )
            }
            Self::IoError {
                message,
                path: None,
            } => {
                write!(f, "I/O error: {}", message)
            }
            Self::IoError {
                message,
                path: Some(path),
            } => {
                write!(f, "I/O error for '{}': {}", path, message)
            }
            Self::ConfigurationError { message, details } => {
                write!(
                    f,
                    "Configuration error for '{}': {} (expected: {})",
                    details.key, message, details.expected
                )?;
                if let Some(actual) = &details.actual {
                    write!(f, ", got: '{}'", actual)?;
                }
                Ok(())
            }
            Self::GenericError {
                message,
                location: None,
            } => {
                write!(f, "Error: {}", message)
            }
            Self::GenericError {
                message,
                location: Some(loc),
            } => {
                write!(f, "Error at {}: {}", loc, message)
            }
        }
    }
}

impl Error for FerroTeXError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // FerroTeXError is the source of truth, so we don't have a source
        None
    }
}

/// Convert from `std::io::Error` to `FerroTeXError`.
///
/// This enables using the `?` operator on I/O operations:
///
/// ```rust,ignore
/// let content = std::fs::read_to_string(path)?; // Automatically converts to FerroTeXError
/// ```
impl From<io::Error> for FerroTeXError {
    fn from(err: io::Error) -> Self {
        Self::IoError {
            message: err.to_string(),
            path: None,
        }
    }
}

/// Convert from `anyhow::Error` to `FerroTeXError`.
///
/// This enables gradual migration from anyhow to the unified error type:
///
/// ```rust,ignore
/// fn old_function() -> anyhow::Result<()> { ... }
///
/// fn new_function() -> FerroTeXResult<()> {
///     old_function()?; // Converts anyhow::Error to FerroTeXError::GenericError
///     Ok(())
/// }
/// ```
impl From<anyhow::Error> for FerroTeXError {
    fn from(err: anyhow::Error) -> Self {
        Self::GenericError {
            message: err.to_string(),
            location: None,
        }
    }
}

/// A convenient type alias for `Result<T, FerroTeXError>`.
///
/// Use this type alias throughout FerroTeX crates instead of defining
/// custom result types.
///
/// # Example
///
/// ```rust
/// use ferrotex_build::error::{FerroTeXResult, FerroTeXError};
///
/// fn may_fail(input: &str) -> FerroTeXResult<i32> {
///     if input.is_empty() {
///         return Err(FerroTeXError::generic_error("empty input"));
///     }
///     Ok(input.len() as i32)
/// }
/// ```
pub type FerroTeXResult<T> = Result<T, FerroTeXError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_creation() {
        let err = FerroTeXError::parse_error("Unexpected token", SourceLocation::new(5, 3));
        assert!(err.is_parse_error());
        assert_eq!(err.message(), "Unexpected token");
        assert_eq!(err.location(), Some(SourceLocation::new(5, 3)));
    }

    #[test]
    fn test_parse_error_display() {
        let err = FerroTeXError::parse_error("Unexpected token", SourceLocation::new(5, 3));
        let msg = format!("{}", err);
        assert!(msg.contains("Parse error"));
        assert!(msg.contains("5:3"));
        assert!(msg.contains("Unexpected token"));
    }

    #[test]
    fn test_parse_error_with_snippet() {
        let err = FerroTeXError::parse_error_with_snippet(
            "Unexpected token",
            SourceLocation::new(5, 3),
            "\\begin{foo}",
        );
        let msg = format!("{}", err);
        assert!(msg.contains("near:"));
        assert!(msg.contains("\\begin{foo}"));
    }

    #[test]
    fn test_analysis_error() {
        let ctx = AnalysisContext::new("type_check", "equation on line 5");
        let err = FerroTeXError::analysis_error("Type mismatch", ctx);
        assert!(err.is_analysis_error());
        assert_eq!(err.message(), "Type mismatch");
        assert_eq!(err.location(), None);
    }

    #[test]
    fn test_analysis_error_with_location() {
        let ctx = AnalysisContext::new("type_check", "equation on line 5");
        let err = FerroTeXError::analysis_error_with_location(
            "Type mismatch",
            ctx,
            SourceLocation::new(5, 10),
        );
        assert_eq!(err.location(), Some(SourceLocation::new(5, 10)));
    }

    #[test]
    fn test_io_error() {
        let err = FerroTeXError::io_error("Permission denied").with_path("/etc/passwd");
        assert!(err.is_io_error());
        let msg = format!("{}", err);
        assert!(msg.contains("/etc/passwd"));
        assert!(msg.contains("Permission denied"));
    }

    #[test]
    fn test_configuration_error() {
        let details = ConfigurationErrorDetails::new("output_format", "pdf or dvi")
            .with_actual("txt");
        let err = FerroTeXError::configuration_error("Invalid format", details);
        assert!(err.is_configuration_error());
        let msg = format!("{}", err);
        assert!(msg.contains("output_format"));
        assert!(msg.contains("txt"));
    }

    #[test]
    fn test_generic_error() {
        let err = FerroTeXError::generic_error("Something went wrong")
            .with_location(SourceLocation::new(1, 1));
        assert!(err.is_generic_error());
        assert_eq!(err.location(), Some(SourceLocation::new(1, 1)));
    }

    #[test]
    fn test_io_error_from_std_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: FerroTeXError = io_err.into();
        assert!(err.is_io_error());
        assert!(err.message().contains("file not found"));
    }

    #[test]
    fn test_anyhow_error_conversion() {
        let anyhow_err = anyhow::anyhow!("Something went wrong");
        let err: FerroTeXError = anyhow_err.into();
        assert!(err.is_generic_error());
        assert_eq!(err.message(), "Something went wrong");
    }

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation::new(10, 25);
        assert_eq!(format!("{}", loc), "10:25");
    }

    #[test]
    fn test_source_span_display() {
        let span = SourceSpan::new(SourceLocation::new(1, 1), SourceLocation::new(5, 10));
        assert_eq!(format!("{}", span), "from 1:1 to 5:10");
    }

    #[test]
    fn test_source_span_point() {
        let span = SourceSpan::point(SourceLocation::new(3, 5));
        assert_eq!(span.start, span.end);
        assert_eq!(format!("{}", span), "at 3:5");
    }

    #[test]
    fn test_ferrotex_result_type() {
        fn returns_ok() -> FerroTeXResult<i32> {
            Ok(42)
        }

        fn returns_err() -> FerroTeXResult<i32> {
            Err(FerroTeXError::generic_error("test error"))
        }

        assert_eq!(returns_ok().unwrap(), 42);
        assert!(returns_err().is_err());
    }

    #[test]
    fn test_error_source_trait() {
        let err = FerroTeXError::generic_error("test");
        // FerroTeXError is the source of truth, so source() returns None
        assert!(err.source().is_none());
    }

    #[test]
    fn test_analysis_context_with_hint() {
        let ctx = AnalysisContext::new("pass", "context").with_hint("Try this instead");
        assert_eq!(ctx.hint, Some("Try this instead".to_string()));
    }

    #[test]
    fn test_configuration_error_details_with_actual() {
        let details = ConfigurationErrorDetails::new("key", "expected").with_actual("actual");
        assert_eq!(details.actual, Some("actual".to_string()));
    }

    #[test]
    fn test_source_location_start() {
        let loc = SourceLocation::start();
        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, 1);
    }

    #[test]
    fn test_error_message_variants() {
        let parse_err = FerroTeXError::parse_error("parse", SourceLocation::start());
        let analysis_err = FerroTeXError::analysis_error("analysis", AnalysisContext::new("p", "c"));
        let io_err = FerroTeXError::io_error("io");
        let config_err = FerroTeXError::configuration_error("config", ConfigurationErrorDetails::new("k", "e"));
        let generic_err = FerroTeXError::generic_error("generic");

        assert_eq!(parse_err.message(), "parse");
        assert_eq!(analysis_err.message(), "analysis");
        assert_eq!(io_err.message(), "io");
        assert_eq!(config_err.message(), "config");
        assert_eq!(generic_err.message(), "generic");
    }

    #[test]
    fn test_error_location_none() {
        let io_err = FerroTeXError::io_error("io");
        let config_err = FerroTeXError::configuration_error("config", ConfigurationErrorDetails::new("k", "e"));
        assert_eq!(io_err.location(), None);
        assert_eq!(config_err.location(), None);
    }

    #[test]
    fn test_analysis_error_display_no_location() {
        let ctx = AnalysisContext::new("pass_name", "some_context");
        let err = FerroTeXError::analysis_error("error_msg", ctx);
        let msg = format!("{}", err);
        assert!(msg.contains("Analysis error in pass_name"));
        assert!(msg.contains("error_msg"));
        assert!(msg.contains("context: some_context"));
    }

    #[test]
    fn test_io_error_display_no_path() {
        let err = FerroTeXError::io_error("no_path_error");
        let msg = format!("{}", err);
        assert_eq!(msg, "I/O error: no_path_error");
    }

    #[test]
    fn test_generic_error_display_no_location() {
        let err = FerroTeXError::generic_error("no_loc_generic");
        let msg = format!("{}", err);
        assert_eq!(msg, "Error: no_loc_generic");
    }

    #[test]
    fn test_error_predicates() {
        let parse_err = FerroTeXError::parse_error("p", SourceLocation::start());
        assert!(parse_err.is_parse_error());
        assert!(!parse_err.is_analysis_error());

        let analysis_err = FerroTeXError::analysis_error("a", AnalysisContext::new("p", "c"));
        assert!(analysis_err.is_analysis_error());
        assert!(!analysis_err.is_io_error());

        let io_err = FerroTeXError::io_error("i");
        assert!(io_err.is_io_error());
        assert!(!io_err.is_configuration_error());

        let config_err = FerroTeXError::configuration_error("c", ConfigurationErrorDetails::new("k", "e"));
        assert!(config_err.is_configuration_error());
        assert!(!config_err.is_generic_error());

        let generic_err = FerroTeXError::generic_error("g");
        assert!(generic_err.is_generic_error());
        assert!(!generic_err.is_parse_error());
    }

    #[test]
    fn test_builder_methods_on_wrong_variants() {
        let parse_err = FerroTeXError::parse_error("p", SourceLocation::start());
        
        // with_path should be a no-op on non-IoError
        let still_parse_err = parse_err.clone().with_path("foo");
        assert_eq!(parse_err, still_parse_err);

        // with_location should be a no-op on non-GenericError
        let still_parse_err_2 = parse_err.clone().with_location(SourceLocation::new(10, 10));
        assert_eq!(parse_err, still_parse_err_2);
    }
}
