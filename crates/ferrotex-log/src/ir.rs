use serde::{Deserialize, Serialize};

/// Represents a span of text in the log file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Start byte offset (inclusive).
    pub start: usize,
    /// End byte offset (exclusive).
    pub end: usize,
}

impl Span {
    /// Creates a new `Span`.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// A confidence score for a parsed event, ranging from 0.0 to 1.0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Confidence(pub f64);

impl Default for Confidence {
    fn default() -> Self {
        Self(1.0)
    }
}

/// A parsed event from the LaTeX log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEvent {
    /// The location of this event in the log file.
    pub span: Span,
    /// Confidence level of the parsing (for heuristic parsers).
    pub confidence: Confidence,
    /// The actual event data.
    #[serde(flatten)]
    pub payload: EventPayload,
}

/// The specific type of log event and its associated data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum EventPayload {
    /// Entered a file (e.g., `(./main.tex`).
    FileEnter {
        /// The path of the file entered.
        path: String,
    },
    /// Exited the current file (e.g., `)`).
    FileExit,
    /// An error message starting with `!`.
    ErrorStart {
        /// The error message content.
        message: String,
    },
    /// A reference to a line number in the source file (e.g., `l.10`).
    ErrorLineRef {
        /// The line number reported.
        line: u32,
        /// Context text following the line number.
        source_excerpt: Option<String>,
    },
    /// A context line provided by LaTeX after an error.
    ErrorContextLine {
        /// The content of the context line.
        text: String,
    },
    /// A warning message (e.g., `LaTeX Warning: ...`).
    Warning {
        /// The warning message content.
        message: String,
    },
    /// General informational message.
    Info {
        /// The message content.
        message: String,
    },
    /// An artifact produced by the build (e.g., PDF, aux file).
    OutputArtifact {
        /// Path to the artifact.
        path: Option<String>,
        /// Format of the artifact (e.g., "pdf").
        format: Option<String>,
        /// Role of the artifact.
        role: Option<String>,
    },
    /// Summary of the build status.
    BuildSummary {
        /// Whether the build was successful.
        success: bool,
    },
}

/// A standardized diagnostic derived from log events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Severity level of the diagnostic.
    pub severity: Severity,
    /// The diagnostic message.
    pub message: String,
    /// The source file associated with the diagnostic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// The range within the source file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<LspRange>,
    /// Confidence in the diagnostic accuracy.
    pub confidence: Confidence,
    /// Information about where this diagnostic came from in the log.
    pub provenance: Provenance,
}

/// Severity of a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Error condition.
    Error,
    /// Warning condition.
    Warning,
    /// Informational message.
    Information,
    /// Hint or suggestion.
    Hint,
}

/// A range in a text document (0-indexed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspRange {
    /// Start position.
    pub start: LspPosition,
    /// End position.
    pub end: LspPosition,
}

/// A position in a text document (0-indexed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspPosition {
    /// Line number.
    pub line: u32,
    /// Character offset on the line.
    pub character: u32,
}

/// Provenance information for a diagnostic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// The span in the log file that generated this diagnostic.
    pub log_span: Span,
    /// The file stack at the time of the event.
    pub file_stack: Vec<String>,
}
