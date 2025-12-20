# FerroTeX VS Code Extension

Official VS Code extension for [FerroTeX](https://github.com/jxoesneon/FerroTeX).

## Features

- **v0.7.0**:
    - **Language Support**:
        - Go to Definition (`\label` <-> `\ref`).
        - Find References.
        - Rename Symbol.
        - Workspace-wide Label Diagnostics.
        - Syntax checking (real-time).
        - Document Outline (symbols).
        - Document Links (navigation for `\input`).
        - Circular dependency detection.
    - **Build Integration**:
        - Live diagnostics from `.log` files.
    - Bundles `ferrotexd` v0.7.0.
- **v0.6.0**:

## Configuration

- `ferrotex.serverPath`: Path to the `ferrotexd` executable (defaults to `ferrotexd` in PATH).
