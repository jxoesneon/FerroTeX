# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.24.0] - 2026-04-09

> "Diagnostic Completeness" Update

Closes the gap between the static analysis capabilities already present in the workspace index and what was actually surfaced through LSP diagnostics and code actions.

### Added

- **Deprecated-command diagnostics**: `{\bf ...}`, `{\it ...}`, `{\rm ...}`, `$$...$$` (display math), and obsolete packages (`times`, `a4wide`, `epsfig`, `psfig`) now produce `warning` diagnostics via `textDocument/publishDiagnostics` on every `didOpen`/`didChange`. Previously `Workspace::validate_deprecated()` existed but was never called from `validate_document()`.
- **Code actions for deprecated commands**: each deprecated diagnostic is paired with a `quickfix` code action:
  - Font commands (`\bf`, `\it`, `\rm`, `\sf`, `\tt`, `\sc`, `\sl`) → `\textbf{...}` etc.
  - Display math `$$...$$` → `\[...\]`
- **`\RequirePackage` support**: the package scanner now recognises `\RequirePackage` in addition to `\usepackage`, ensuring packages declared in `.cls`/`.sty` files are included in completion and package-awareness features.
- **`\addbibresource` indexing**: BibLaTeX documents using `\addbibresource{refs.bib}` now have their `.bib` files indexed for citation key completion and cross-file validation, identical to `\bibliography{refs}`.

### Changed

- `code_action_provider` capability advertised in `InitializeResult`.
- Label and duplicate-label diagnostics now carry `source: "ferrotex"` for better client-side filtering.

### Fixed

- Three regression tests added covering the new diagnostic pipeline, deprecated helper functions, and `\addbibresource` indexing. All 419 workspace tests pass.

## [0.23.0] - 2026-04-04

> "Institutional Excellence" Update

This major minor release marks FerroTeX's transition to an institutional-grade platform, featuring a unified error architecture, hardened security models, and a robust local CI infrastructure.

### Added

- **Unified Error System**:
  - Introduced `FerroTeXError` in `ferrotex-build` as the workspace-wide diagnostic standard.
  - Migrated `syntax`, `analysis`, `cli`, and `core` to the new structured error system.
- **Local CI Infrastructure**:
  - New `scripts/ci-run.sh` script for running the full CI pipeline locally via Docker.
  - Static `Dockerfile.ci-test` ensures 100% environment parity between local and remote builds.
- **Security & Hardening**:
  - Implemented **CodeQL Advanced Security** scanning for Rust, JavaScript, and GitHub Actions.
  - Established a formal **Security Model** with threat analysis and VFS isolation specifications.
- **Institutional Documentation**:
  - Comprehensive **API Reference** (`docs/api.md`) for all core crates.
  - Detailed **Troubleshooting Guide** for enterprise/academic support.
  - Expanded **Glossary** and technical specifications for all core pillars.

### Changed

- **Test Coverage**: Achieved a foundational milestone of **>90% code coverage** across the entire workspace.
- **Repository Health**: Consolidated 5 stale branches and deleted 20+ redundant remote branches for a leaner development experience.

### Fixed

- **Security Remediation**: Massively updated dependencies to resolve dozens of known vulnerabilities in both Rust and Node.js ecosystems.
- **Link Integrity**: Patched the link checker to correctly ignore build artifacts and system directories.

## [0.22.0] - 2026-01-16

> "The Debug & Reliability Update"

This minor release introduces the fully validated Tectonic Debug Driver alongside massive semantic engine reliability improvements and registry-based dependency management.

### Added

- **Debug Adapter Protocol (DAP)**:
  - Finalized Tectonic Adapter implementation with robust launch sequences.
  - Implemented `launch`, `attach`, `evaluate`, and `variables` requests for the Tectonic engine.
  - Added support for live engine event interpretation during debugging.

### Fixed

- **Parsing Resilience**:
  - Resolved potential panic in `scan_file` when encountering malformed or detached braced command arguments.
  - Hardened scanner against partial or broken package declarations.
- **Bibliography Indexing**:
  - Fixed edge case where `get_all_citation_keys` ignored bibliographies not explicitly referenced by a TeX file (orphan bib support).

### Changed

- **Code Quality**:
  - Enforced strict `cargo clippy` compliance across the entire workspace.
  - Verified compilation of optimized release artifacts with `lto = true`.
- **Tectonic Integration**:
  - Shifted from path-based patches to registry dependencies for all `jxoesneon-tectonic-*` crates.
  - Standardized dependency constraints to `>= 0.16.0` to leverage the published [`jxoesneon-tectonic`](https://crates.io/crates/jxoesneon-tectonic) ecosystem.
  - Unified the engine feature flag to `jxoesneon-tectonic-engine` across the workspace for consistent configuration.
- **Testing**:
  - Achieved >95% code coverage across the Language Server Semantic Engine.
  - Reached 100% coverage in critical components including `completer.rs`.

## [0.21.0] - 2026-01-03

> "Global Expansion" Release

This release brings full cross-platform support with automated binary bundling for Linux, Windows, and macOS.

### Added

- **Multi-Platform Distribution**:
  - Implemented automatic platform-specific bundling; the extension now detects the OS and installs the correct pre-compiled binary.
  - Added official support for Linux (x64), Windows (x64), and macOS (Intel & Apple Silicon).
  - Configured CI/CD to produce four distinct VSIX packages for Open VSX and GitHub Releases.
- **Linux Validation**:
  - Added `Dockerfile.local` and CI steps to verify compilation and binary execution on Ubuntu.
- **Production Optimizations**:
  - Enabled `lto = true` and `strip = true` for release builds, significantly reducing binary size and improving startup time.

## [0.20.1] - 2026-01-03

> "Stellar Security" Patch

### Fixed

- **Security Remediation**:
  - Remediated five critical vulnerabilities in `gix-*` dependencies via workspace-level `[patch.crates-io]` overrides.
  - Upgraded `reqwest` to `v0.13.1` in `ferrotexd` to modernize the networking stack.
- **Build Stability**:
  - Implemented workaround for directory creation failures in `tectonic_bridge_harfbuzz` build scripts.
  - Resolved `Cargo.toml` duplicate patch table errors.

## [0.20.0] - 2026-01-02

> "Engine Synchrony" Release

Transforms FerroTeX into a high-fidelity "Scientific Compiler" platform, introducing cryptographic reproducibility and live state inspection.

### Added

- **Content-Addressable Reproducibility**:
  - Automatic `ferrotex.lock` generation for hermetic builds.
  - `ferrotex verify` command for cryptographic baseline checks.
- **DAP State Inspection**:
  - Live shadowing of TeX internal registers (`\count`, `\dimen`) and macros.
  - Real-time engine state visualization in the VS Code variables pane.
- **Safety-Critical Analysis**:
  - Preventative detection of infinite recursion and stack overflows in macros.
  - Cycle detection for circular control sequences.
- **Universal Build System**:
  - DAG-based build engine with topological sorting and cycle detection.
- **Semantic Math Verification**:
  - Real-time jagged matrix detection and mathematical delimiter balancing.

### Fixed

- Cleaned up all compiler warnings across the entire workspace (10 crates).
- Resolved unused imports and variables in `ferrotex-dap` and `ferrotex-syntax`.
- Synchronized all workspace crate versions to `0.20.0`.

### Changed

- Refactored `TectonicShim` to use trace-based variable extraction instead of unstable FFI.
- Enhanced LSP progress reporting for package indexing.

## [0.19.2] - 2025-12-29

### Fixed

- Marketplace images now use absolute GitHub URLs to resolve 404 errors on Open VSX.

## [0.19.1] - 2025-12-29

### Changed

- **Asset Refresh**:
  - Updated marketplace hero banner and feature screenshots.
  - Improved `icon.png` visibility.

## [0.19.0] - 2025-12-29

### Added

- **Windows Support**:
  - Official support for Windows 10/11 (x64) with verified E2E test suite.
  - Fixed binary discovery logic to support `.exe` extension.
  - Resolved compilation issues on MSVC and updated build pipeline for Windows system dependencies.

## [0.18.0] - 2025-12-22

### Added

- **Image Paste Wizard**: Seamlessly paste images from clipboard into LaTeX documents (UX-3).
- **Math Semantics Validation**: Deep validation for math environments and command arguments.
- **Package Management Integration**: Auto-detects missing packages and prompts for installation via `tlmgr` or `miktex`.
- **Build Infrastructure**: Support for linking against system libraries (`harfbuzz`, `icu`, `openssl`) on Linux/macOS.
- **Testing**: Achieved >90% code coverage across core crates.

## [0.17.0] - 2025-12-21

### Added

- **Comprehensive Settings System**: 46 configurable settings for complete customization (UX-7).
- **Marketplace Improvements**: Added version, downloads, and license badges to the marketplace page.

### Changed

- Updated LICENSE file with full Apache-2.0 text.

### Fixed

- Build command handler implementation.

## [0.16.0] - 2025-12-21

### Added

- **Zero-Config Build**: Automatically downloads and installs Tectonic if no TeX engine is found.
- **Self-Contained PDF Viewer**: Bundled PDF.js directly into the extension for offline use.
- **Rich Hovers**: Added math formatting and citation detail previews on hover.
- **Human-Readable Error Index**: Expanded database to translate common LaTeX errors into actionable advice.

## [0.15.0] - 2025-12-21

### Added

- **Snippet Pack**: Added 130+ LaTeX snippets for Math, Greek, and Environments.
- **Magic Comments**: Support for `%!TEX root = ...` to override build root.
- **Dynamic Package Metadata**: Context-aware auto-completion for major LaTeX packages.

## [0.14.2] - 2025-12-20

### Fixed

- **Marketplace Metadata**: Corrected LICENSE and `.vscodeignore` for Open VSX publishing compliance.

## [0.14.1] - 2025-12-20

### Added

- **Release Automation**: Integrated Open VSX auto-publish workflow and improved marketplace metadata.

## [0.14.0] - 2025-12-20

### Added

- **Schema Stabilization**: Established compatibility guarantees for log event IR.
- **Extension Testing**: Introduced mocha-based test suite and automated extension testing.
- **VSIX Packaging**: Automated packaging and CI artifacts for the VS Code extension.

## [0.13.0] - 2025-12-20

### Added

- **Integrated Environment**:
  - Built-in PDF previewer with SyncTeX inverse search.
  - Forward search from code to PDF.
- **Smart Diagnostics**: Missing package detection with installation prompts.

## [0.12.0] - 2025-12-20

### Added

- **Status Bar Integration**: Visual build progress feedback via LSP.
- **Image Paste Wizard**: Prompts for filename and inserts `\includegraphics` snippet.

## [0.11.0] - 2025-12-20

### Added

- **Build Orchestration**: Introduced DAG-based orchestrator with `latexmk` support by default.

## [0.10.0] - 2025-12-20

### Added

- **Formatting**: Conservative structural formatter for LaTeX indentation.

## [0.9.0] - 2025-12-20

### Added

- **Semantic Highlighting**: Full support for semantic tokens in the editor.
- **Folding Ranges**: Support for environments, groups, and sections.
- **Workspace Symbols**: Global search for labels, sections, and BibTeX entries.

## [0.8.0] - 2025-12-20

### Added

- **Bibliography Support**: Automatic discovery and real-time watching of `.bib` files.
- **Citation Intelligence**: Autocomplete and diagnostics for citations.

### Changed

- Migrated all crates to the Rust 2024 edition.

## [0.7.0] - 2025-12-20

### Added

- **Label Management**: Full support for label definitions, references, and renaming.
- **Workspace Indexing**: Recursive startup scan and real-time file watching.

## [0.6.0] - 2025-12-19

### Added

- **Project Model**: Introduced workspace include graph tracking.
- **Document Links**: Support for navigating to included files via `\input` or `\include`.

## [0.5.0] - 2025-12-19

### Added

- **LaTeX Parser**: Fault-tolerant CST parser based on `rowan`.
- **Source Diagnostics**: Real-time syntax error reporting for unmatched braces.

## [0.4.0] - 2025-12-19

### Added

- **LSP Server**: Initial Language Server Protocol implementation.
- **VS Code Extension**: Bootstrapped client for the FerroTeX platform.

## [0.3.0] - 2025-12-19

### Added

- **Streaming Log Ingestion**: Incremental log parsing for real-time diagnostics.

## [0.2.0] - 2025-12-19

### Added

- **Offline Log Parser**: Implementation of typed log event IR (`ferrotex-log`).
- **CLI**: Initial `ferrotex-cli parse` command.

## [0.1.0] - 2025-12-19

### Added

- Architectural documentation and specification set.

## [0.0.0] - 2025-12-19

### Added

- Initial repository structure.

[Unreleased]: https://github.com/jxoesneon/FerroTeX/compare/v0.24.0...HEAD
[0.24.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.23.0...v0.24.0
[0.23.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.22.0...v0.23.0
[0.22.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.21.0...v0.22.0
[0.21.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.20.1...v0.21.0
[0.20.1]: https://github.com/jxoesneon/FerroTeX/compare/v0.20.0...v0.20.1
[0.20.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.19.2...v0.20.0
[0.19.2]: https://github.com/jxoesneon/FerroTeX/compare/v0.19.1...v0.19.2
[0.19.1]: https://github.com/jxoesneon/FerroTeX/compare/v0.19.0...v0.19.1
[0.19.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.18.0...v0.19.0
[0.18.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.17.0...v0.18.0
[0.17.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.16.0...v0.17.0
[0.16.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.15.0...v0.16.0
[0.15.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.14.2...v0.15.0
[0.14.2]: https://github.com/jxoesneon/FerroTeX/compare/v0.14.1...v0.14.2
[0.14.1]: https://github.com/jxoesneon/FerroTeX/compare/v0.14.0...v0.14.1
[0.14.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.13.0...v0.14.0
[0.13.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.12.0...v0.13.0
[0.12.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.0.0...v0.1.0
[0.0.0]: https://github.com/jxoesneon/FerroTeX/releases/tag/v0.0.0
