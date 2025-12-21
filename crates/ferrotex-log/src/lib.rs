//! # FerroTeX Log Parser
//!
//! This crate provides a parser for LaTeX log files. It is designed to be robust and
//! capable of streaming/incremental parsing to support real-time feedback in editors.
//!
//! The core struct is `LogParser`, which produces a stream of `LogEvent`s.

pub mod ir;
pub mod parser;

pub use parser::LogParser;

/// Schema version for the log event IR.
///
/// This version follows semantic versioning:
/// - MAJOR: Breaking changes to event structure
/// - MINOR: New optional fields or event types
/// - PATCH: Bug fixes to parsing behavior
///
/// Starting with 1.0.0, backward compatibility is guaranteed within major versions.
pub const SCHEMA_VERSION: &str = "1.0.0";
