//! # FerroTeX Log Parser
//!
//! Streaming parser for LaTeX engine log files (`*.log`) with structured event output.
//!
//! ## Overview
//!
//! This crate transforms the unstructured, line-wrapped output of TeX engines
//! (pdfTeX, XeTeX, LuaTeX, etc.) into a stream of typed [`LogEvent`](ir::LogEvent)s.
//! The parser is designed to handle:
//!
//! - **Line wrapping**: TeX logs wrap at 79 characters, splitting paths and messages
//! - **File stack tracking**: Matching `(file.tex` and `)` pairs for context
//! - **Error/warning extraction**: Detecting `!` errors, `LaTeX Warning:`, overful boxes
//! - **Incremental/streaming updates**: Processing logs as they're written
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────┐     update()      ┌────────────┐
//! │   .log file  │ ─────────────────► │ LogParser  │
//! │  (streaming) │ ◄───────────────── │            │
//! └──────────────┘   Vec<LogEvent>    └────────────┘
//!                                          │
//!                                          │ finish()
//!                                          ▼
//!                                   Final events
//! ```
//!
//! The core type is [`LogParser`](parser::LogParser), which maintains:
//!
//! - **File stack**: Tracks nested includes via `(...` and `)`
//! - **Internal buffer**: Holds partial lines not yet processed
//! - **Global offset**: Byte position for precise source mapping
//!
//! ## Event IR (Intermediate Representation)
//!
//! The [`ir`] module defines the typed event schema:
//!
//! - [`LogEvent`](ir::LogEvent) - Base event with span and confidence
//! - [`EventPayload`](ir::EventPayload) - Discriminated union of event types
//!   - `FileEnter { path }` - Engine opened a file
//!   - `FileExit` - Engine closed a file
//!   - `ErrorStart { message }` - Line starting with `!`
//!   - `Warning { message }` - LaTeX/package warning
//!   - `ErrorLineRef { line, excerpt }` - `l.123 ...` reference
//!
//! ## Schema Versioning
//!
//! The IR schema follows **semantic versioning** via [`SCHEMA_VERSION`]:
//!
//! - **MAJOR**: Breaking changes to event structure (e.g., removing fields)
//! - **MINOR**: New event types or optional fields (backward compatible)
//! - **PATCH**: Bug fixes to parsing behavior (no schema changes)
//!
//! ## Examples
//!
//! ### One-shot Parsing
//!
//! ```no_run
//! use ferrotex_log::LogParser;
//! use std::fs;
//!
//! let log_content = fs::read_to_string("main.log")?;
//! let parser = LogParser::new();
//! let events = parser.parse(&log_content);
//!
//! for event in events {
//!     println!("{:?}", event.payload);
//! }
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! ### Streaming/Incremental Parsing
//!
//! ```
//! use ferrotex_log::LogParser;
//!
//! let mut parser = LogParser::new();
//!
//! // First chunk arrives
//! let chunk1 = "This is pdfTeX, Version 3.14159\n(main.tex\n";
//! let events1 = parser.update(chunk1);
//! println!("Received {} events from chunk 1", events1.len());
//!
//! // Second chunk arrives
//! let chunk2 = "LaTeX Warning: Label `foo' undefined\n";
//! let events2 = parser.update(chunk2);
//! println!("Received {} events from chunk 2", events2.len());
//!
//! // Finalize
//! let final_events = parser.finish();
//! ```
//!
//! ### Exporting to JSON
//!
//! The IR types implement `serde::Serialize`:
//!
//! ```no_run
//! use ferrotex_log::LogParser;
//! use std::fs;
//!
//! let log = fs::read_to_string("main.log")?;
//! let events = LogParser::new().parse(&log);
//! let json = serde_json::to_string_pretty(&events)?;
//! fs::write("events.json", json)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

/// Typed event Intermediate Representation (IR).
pub mod ir;
/// Streaming parser implementation.
pub mod parser;

#[cfg(test)]
mod tests;

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
