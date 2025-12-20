# Changelog

All notable changes to this project will be documented in this file.

The format is based on **Keep a Changelog**, and this project intends to follow **Semantic Versioning** once a stable public API exists.

## [Unreleased]

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

[Unreleased]: https://github.com/jxoesneon/FerroTeX/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.7.0...v0.8.0
[0.2.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.0.0...v0.1.0
[0.0.0]: https://github.com/jxoesneon/FerroTeX/releases/tag/v0.0.0
