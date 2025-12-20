# FerroTeX Language Server (`ferrotexd`)

This is the reference implementation of the FerroTeX Language Server Protocol (LSP) server.

## Features

- **v0.7.0**:
    - **Label Management**: Go to Definition, Find References, and Rename for `\label`/`\ref`.
    - **Label Diagnostics**: Detect duplicate labels and undefined references.
    - **Workspace Scanning**: Indexes all `.tex` files on startup.
    - **File Watching**: Reacts to file changes, additions, and deletions.
- **v0.6.0**:
    - **Project Model**: Tracks file dependencies (`\input`, `\include`).
    - **Document Links**: Navigate to included files.
    - **Cycle Detection**: Detects recursive includes.
    - **Document Symbols**: Outline for environments and sections.
    - **Syntax Diagnostics**: Real-time syntax checking.
    - **Build Diagnostics**: Streaming `.log` ingestion.
    - **Standard LSP**: Runs over stdio.

## Usage

Start the server over stdio:

```bash
cargo run -p ferrotexd
```

Or configure your editor to run the binary.
