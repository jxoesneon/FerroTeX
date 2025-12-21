# FerroTeX VS Code Extension

Official VS Code extension for [FerroTeX](https://github.com/jxoesneon/FerroTeX).

## Features

- **v0.15.0 ("Intelligence" Update)**:
  - **Magic Comments**: `%!TEX root = ...` support for overriding build root.
  - **Dynamic Package Data**: Context-aware completion for packages like `amsmath`, `tikz`.
  - **Snippet Pack**: 130+ snippets for Math, Greek, Environments.
  - **Linting**: Fixed collapsible if statements and unused code.
- **v0.14.0**:
  - **Coverage**: Added `cargo-tarpaulin` coverage workflow.
  - **Open VSX**: Publishing support added.

## Configuration

- `ferrotex.serverPath`: Path to the `ferrotexd` executable (defaults to `ferrotexd` in PATH).
