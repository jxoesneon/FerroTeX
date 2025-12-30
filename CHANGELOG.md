# Changelog

All notable changes to this project will be documented in this file.

The format is based on **Keep a Changelog**, and this project intends to follow **Semantic Versioning** once a stable public API exists.

## [Unreleased]

## [0.19.2] - 2025-12-29

### Documentation

- **Fixed**: Marketplace images now use absolute GitHub URLs to resolve 404 errors on Open VSX.

## [0.19.1] - 2025-12-29

### Asset Refresh

- Updated marketplace assets:
  - New `hero_banner.png` with modern branding.
  - Refreshed feature screenshots (`completion`, `diagnostics`, `hover`, `settings`).
  - Improved `icon.png` visibility.

## [0.19.0] - 2025-12-29

### Added

- **Windows Support**: Officially supported on Windows 10/11 (x64).
  - Verified full E2E test suite passing on Windows CI.
  - Fixed binary discovery logic to support `.exe` extension.
  - Resolved `ferrotex-core` compilation issues on MSVC.
  - Updated build pipeline to correctly link against system dependencies on Windows.

## [0.18.0] - 2025-12-22

### Added

- **Image Paste Wizard**: Seamlessly paste images from clipboard into LaTeX documents (UX-3).
  - Automatically saves image to configured directory (default: `figures/`).
  - Generates unique filenames and inserts `\includegraphics` snippet.
- **Math Semantics Validation**: Deep validation for math environments.
  - Checks for mismatched delimiters (`(`, `[`, `{`, `\langle`, etc.).
  - Validates command arguments (`\frac`, `\sqrt`, etc.).
- **Package Management Integration**:
  - Auto-detects missing packages from build logs.
  - Prompts to install via `tlmgr` or `miktex`.
- **Build Infrastructure**:
  - Added full support for linking against system libraries (`harfbuzz`, `icu`, `openssl`) on Linux/macOS for the language server.
  - Improved CI robustness with comprehensive system dependency installation.
  - Enabled `external-harfbuzz` feature for Tectonic engine integration.
- **Testing**: Achieved >90% code coverage across core crates.
  - Integrated `cargo-tarpaulin` and Codecov for continuous monitoring.

- **Comprehensive Settings System**: 46 configurable settings for complete customization (UX-7)
  - Build engine selection (auto, tectonic, latexmk, pdflatex, xelatex, lualatex)
  - Custom engine paths with validation
  - Granular lint rule controls (master switch + individual rules)
  - Preview customization (zoom, sync, scroll mode)
  - Completion and formatting options
  - Workspace configuration (scan, file size limits, exclude patterns)
- **Marketplace Improvements**:
  - Apache-2.0 LICENSE for proper GitHub license detection
  - Version, downloads, and license badges on marketplace page
  - Ko-fi donation badge for community support
  - GitHub Discussions QnA link

### Fixed

- Build command handler implementation

### Changed

- LICENSE file now contains full Apache-2.0 text (dual-license notice moved to README.md)

## [0.16.0] - 2025-12-21

### Added

- **Zero-Config Build**: Automatically downloads and installs **Tectonic** if no TeX engine is found (UX-ZeroConfig).
- **Self-Contained PDF Viewer**: Bundled PDF.js directly into the extension, resolving empty preview issues on offline/restricted environments.
- **Rich Hovers**: Added math formatting and citation detail previews on hover (UX-1).
- **Human-Readable Error Index**: Expanded error database to translate common LaTeX errors into actionable advice (UX-5).
- **Marketplace Overhaul**: New branding assets, hero banner, and feature-focused README.
- **Comprehensive Settings System**: Fully configurable build engines (`build.engine`), linting rules, and preview behavior (UX-7).

## [0.15.0] - 2025-12-21

### Added

- **Snippet Pack**: Added 130+ LaTeX snippets for Math, Greek, and Environments (UX-2).
- **Magic Comments**: Added support for `%!TEX root = ...` to override build root (PM-Override).
- **Dynamic Package Metadata**: Added context-aware auto-completion for `amsmath`, `tikz`, `geometry`, `hyperref`, and `graphicx` (CS-3).
- **Linting**: Fixed various clippy lints (`collapsible_if`, `unused_imports`) in the codebase.

## [0.14.0] - 2025-12-21

### Added

- **Schema Stabilization**:
  - Introduced `ferrotex_log::SCHEMA_VERSION = "1.0.0"` constant for log event IR.
  - Updated `docs/spec/log-event-ir.md` with compatibility guarantees.
- **Test Coverage**:
  - Added `coverage.yml` GitHub Actions workflow with cargo-tarpaulin.
  - Integrated Codecov for coverage reporting.
  - Added CI/coverage badges to README.
- **Extension Testing**:
  - Added VS Code extension test suite with mocha framework.
  - Added `@vscode/test-electron` for automated extension testing.
- **VSIX Packaging**:
  - Added `vsce` and `npm run package` script for VSIX builds.
  - Added `extension` CI job to build VSIX artifacts.
- **Documentation**:
  - Premium documentation design with Just the Docs theme.
  - Wiki automation (`wiki-sync.yml`).

## [0.13.0] - 2025-12-20

### Added

- **Integrated Environment**:
  - **PDF Viewer**: Built-in PDF previewer (`ferrotex.pdfPreview`) with:
    - Zoom controls and Toolbar.
    - SyncTeX inverse search (Ctrl+Click).
  - **SyncTeX Forward Search**: New `ferrotex.syncToPdf` command navigates from code to PDF.
  - **Math Hover**: Preview math equations and inline math rendered via Markdown/MathJax.
- **Smart Diagnostics**:
  - **Missing Package Detection**: Auto-detects `! LaTeX Error: File 'foo.sty' not found` and prompts for installation using `tlmgr`.

## [0.12.0] - 2025-12-20

### Added

- **UX Polish (Delighters)**:
  - **Status Bar Integration**: Visual feedback ("Building...") via `$/progress` notifications during compilation.
  - **Notifications**: Toast notifications for build success/failure (`window/showMessage`) with clear outcomes.
  - **Magic Comments**: Support for `%!TEX root = ...` to transparently redirect builds from sub-files to the main document.
  - **Image Paste Wizard**: Pasting an image into the editor prompts for a filename, saves it, and inserts `\includegraphics{...}`.

## [0.11.0] - 2025-12-20

### Added

- **Build Orchestration**: Introduced `ferrotexd/src/build` module to orchestrate compiler execution.
  - **Latexmk Adapter**: Default build engine uses `latexmk` for robust, multi-pass compilation.
  - **LSP Integration**: New `ferrotex.build` command triggers builds from the editor.
  - **Client**: Added `FerroTeX: Build Document` command (Ctrl+Alt+B / Cmd+Alt+B).
- **Architecture**: Defined `BuildEngine` trait for future extensibility (e.g., Tectonic).

## [0.10.0] - 2025-12-20

### Added

- **Formatting**: Introduced `ferrotexd/src/fmt.rs`, a conservative structural formatter for LaTeX documents.
  - Automatically corrects indentation for nested environments.
  - Supports `\begin{...}` and `\end{...}` blocks.
  - Does not alter line breaks or reflow text to ensure data safety.
- **LSP Features**:
  - Implemented `textDocument/formatting` handler.
  - Implemented `textDocument/codeAction` stub (foundation for Quick Fixes).
- **Client**:
  - Registered `ferrotex.build` command foundation (stashed for v0.11.0).
  - Cleaned up dependency versions.

## [0.9.0] - 2025-12-20

### Added

- **Semantic Highlighting**:
  - Full support for `textDocument/semanticTokens/full`.
  - Highlights commands (`\foo`), environments (`\begin`, `\end`), comments (`%`), and parameters.
  - Distinguishes between control flow (keywords) and content.
- **Folding Ranges**:
  - Foldable regions for environments (`\begin`...`\end`), groups (`{...}`), and sections.
- **Workspace Symbols**:
  - `workspace/symbol` support for searching labels (`\label{...}`) and sections (`\section{...}`) across the entire project.
  - Searchable BibTeX entries by citation key.
- **Enhanced Completion**:
  - **Environments**: Autocomplete for `\begin{...}` based on standard LaTeX environments.
  - **Commands**: Autocomplete for common LaTeX commands (starting with `\`).
  - **File Paths**: Autocomplete for `\input{...}` and `\include{...}` scanning the workspace for `.tex` files.
  - **Context Aware**: Smarter triggering inside braces and commands.

## [0.8.0] - 2025-12-20

### Added

- **Bibliography Support**:
  - **Discovery**: Automatically detects `.bib` files referenced via `\bibliography{...}` and `\addbibresource{...}`.
  - **Watching**: Monitors referenced `.bib` files for changes and updates the index in real-time.
  - **BibTeX Parsing**: Robust parsing of BibTeX entries to extract citation keys.
- **Citation Intelligence**:
  - **Completion**: Autocomplete support for `\cite{...}` using indexed keys from all referenced bibliographies.
  - **Diagnostics**:
    - **Undefined Citations**: Reports errors for citations not found in loaded bibliographies.
    - **Missing Bibliography**: Reports errors if a referenced `.bib` file is missing or unreadable.
    - **Smart Validation**: Suppresses "undefined citation" noise if the referenced bibliography file itself is missing.

### Changed

- **Rust 2024 Edition**: Migrated all crates (`ferrotexd`, `ferrotex-syntax`, `ferrotex-log`, `ferrotex-cli`) to Rust 2024 edition.
  - Updated control flow to use `let-chains` (`if let ... && ...`).
  - modernized loops (`while let` -> `loop { let ... else { break } }`) for strict compliance.

## [0.7.0] - 2025-12-19

### Added

- **Label Management**: Full support for `\label` and `\ref`.
  - **Go to Definition**: Jump from `\ref{...}` to `\label{...}`.
  - **Find References**: List all references to a label.
  - **Rename**: Rename a label and update all references across the workspace.
- **Label Diagnostics**:
  - **Duplicate Definitions**: Reports error if a label is defined multiple times.
  - **Undefined References**: Reports error if a `\ref` points to a non-existent label.
- **Workspace Indexing**:
  - **Startup Scan**: Recursively indexes all `.tex` files in the workspace root on startup.
  - **File Watching**: Monitors `.tex` files for creation, modification, and deletion to keep the index in sync.

## [0.6.0] - 2025-12-19

### Added

- **Project Model**: Introduced `Workspace` to track the include graph of LaTeX documents.
- **Document Links**: `ferrotexd` now supports `textDocument/documentLink`.
  - Resolves `\input{...}` and `\include{...}` paths relative to the current file.
  - Allows navigation to included files.
- **Cycle Detection**:
  - Detects include cycles (e.g., A includes B, B includes A).
  - Reports cycles as diagnostics on the include command.

## [0.5.0] - 2025-12-19

### Added

- **LaTeX Parser**: Introduced `ferrotex-syntax` crate with a fault-tolerant CST parser based on `rowan`.
  - Supports LaTeX lexing (commands, groups, environments).
  - Handles recovery from syntax errors (e.g., missing braces).
- **Document Symbols**: `ferrotexd` now supports `textDocument/documentSymbol`.
  - Hierarchical outline for environments (`\begin{...}`) and groups.
  - Section extraction (`\section{...}`) for navigation.
- **Source Diagnostics**: Real-time syntax error reporting in the editor.
  - Reports unmatched braces and unclosed environments.
  - Maps source ranges accurately using `line-index`.

## [0.4.0] - 2025-12-19

### Added

- **LSP Server**: Initial release of `ferrotexd`, a Language Server Protocol implementation for FerroTeX.
  - Supports `textDocument/publishDiagnostics` via streaming log ingestion.
  - Watches workspace `.log` files for changes using `notify`.
- **VS Code Extension**: New extension (`editors/vscode`) to bootstrap the client.
  - Launches `ferrotexd` automatically.
  - Configurable server path via `ferrotex.serverPath`.

## [0.3.0] - 2025-12-19

### Added

- **Streaming Log Ingestion**: `LogParser` now supports incremental updates via `update()` method.
- **CLI Watch Command**: New `ferrotex watch <file>` command to tail log files in real-time.
- **Peek-Line Strategy**: Robust handling of split lines and path wrapping during streaming.

## [0.2.0] - 2024-05-20

### Added

- **Offline Log Parser**: Implemented typed log event IR (`ferrotex-log`) for parsing TeX logs.
- **CLI**: Added `ferrotex-cli parse` command to output log events as JSON.
- **Diagnostics**:
  - Detection of error blocks (`!`) and line references (`l.<n>`).
  - Parsing of `LaTeX Warning` and `Overfull`/`Underfull` hbox warnings.
  - File stack reconstruction (`FileEnter`/`FileExit`) handling `(` and `)`.
- **Robustness**:
  - Guarded joining for 79-character wrapped lines.
  - Snapshot testing infrastructure (`golden_tests`).
  - Fuzzing target for parser panic safety.
- **CI**: Added Rust build and test workflow.

## [0.1.0] - 2025-12-19

### Added

- Documentation set defining:
  - architecture
  - typed log event IR
  - parsing strategy
  - LSP/DAP surfaces
  - evaluation methodology

## [0.0.0] - 2025-12-19

### Added

- Initial repository documentation.

[0.0.0]: https://github.com/jxoesneon/FerroTeX/releases/tag/v0.0.0
[0.1.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.0.0...v0.1.0
[0.2.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.1.0...v0.2.0
[0.3.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.2.0...v0.3.0
[0.4.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.3.0...v0.4.0
[0.5.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.4.0...v0.5.0
[0.6.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.5.0...v0.6.0
[0.7.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.6.0...v0.7.0
[0.8.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.7.0...v0.8.0
[0.9.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.8.0...v0.9.0
[0.10.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.9.0...v0.10.0
[0.11.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.10.0...v0.11.0
[0.12.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.11.0...v0.12.0
[0.13.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.12.0...v0.13.0
[0.14.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.13.0...v0.14.0
[0.15.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.14.0...v0.15.0
[0.16.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.15.0...v0.16.0
[Unreleased]: https://github.com/jxoesneon/FerroTeX/compare/v0.16.0...HEAD
