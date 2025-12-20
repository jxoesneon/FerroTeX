//! # FerroTeX Log Parser
//!
//! This crate provides a parser for LaTeX log files. It is designed to be robust and
//! capable of streaming/incremental parsing to support real-time feedback in editors.
//!
//! The core struct is `LogParser`, which produces a stream of `LogEvent`s.

pub mod ir;
pub mod parser;

pub use parser::LogParser;
