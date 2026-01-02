//! # FerroTeX Core
//!
//! Core utilities and services for the FerroTeX LaTeX language platform.
//!
//! ## Overview
//!
//! This crate provides essential infrastructure components that support the FerroTeX
//! ecosystem, including package management abstractions and LaTeX math validation utilities.
//! These components are designed to be reusable across different FerroTeX tools
//! (language server, CLI, and other consumers).
//!
//! ## Modules
//!
//! - [`package_manager`] - Abstraction layer for TeX package managers (tlmgr, MiKTeX)
//! - [`math_validator`] - Validation tools for LaTeX mathematical expressions
//!
//! ## Design Philosophy
//!
//! The core crate follows these principles:
//!
//! - **TeX Distribution Agnostic**: Supports multiple TeX distributions through trait abstractions
//! - **Testability**: All external interactions (commands, filesystem) are mockable via traits
//! - **Zero External State**: Pure functions and explicit dependencies for predictability
//! - **Incremental Validation**: Support streaming/incremental analysis where applicable
//!
//! ## Examples
//!
//! ### Using the Package Manager
//!
//! ```no_run
//! use ferrotex_core::package_manager::PackageManager;
//!
//! // Auto-detect available package manager (tlmgr or MiKTeX)
//! let pm = PackageManager::new();
//!
//! if pm.is_available() {
//!     // Install a package
//!     match pm.install("amsmath") {
//!         Ok(status) => println!("Install status: {:?}", status),
//!         Err(e) => eprintln!("Installation failed: {}", e),
//!     }
//! }
//! ```
//!
//! ### Validating Math Delimiters
//!
//! ```
//! use ferrotex_core::math_validator::{DelimiterValidator, Delimiter, DelimiterKind};
//!
//! let mut validator = DelimiterValidator::new();
//! // Construct a manual token stream for demonstration
//! let delimiters = vec![
//!     Delimiter { kind: DelimiterKind::LeftParen, position: 0, is_left_command: true },
//!     Delimiter { kind: DelimiterKind::RightParen, position: 10, is_left_command: true },
//! ];
//!
//! validator.validate(&delimiters);
//! if !validator.has_errors() {
//!     println!("Math expression is valid!");
//! }
//! ```
//!
//! ## Feature Flags
//!
//! Currently, this crate does not define any feature flags. All functionality is
//! available by default.

pub mod math_validator;
pub mod package_manager;
