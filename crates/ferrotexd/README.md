# FerroTeX Language Server (`ferrotexd`)

This is the reference implementation of the FerroTeX Language Server Protocol (LSP) server.

## Features

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
