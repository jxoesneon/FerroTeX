# Changelog

All notable changes to this project will be documented in this file.

The format is based on **Keep a Changelog**, and this project intends to follow **Semantic Versioning** once a stable public API exists.

## [Unreleased]

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

[Unreleased]: https://github.com/jxoesneon/FerroTeX/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jxoesneon/FerroTeX/compare/v0.0.0...v0.1.0
[0.0.0]: https://github.com/jxoesneon/FerroTeX/releases/tag/v0.0.0
