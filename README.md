# FerroTeX

[![CI](https://github.com/jxoesneon/FerroTeX/actions/workflows/ci.yml/badge.svg)](https://github.com/jxoesneon/FerroTeX/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/jxoesneon/FerroTeX/branch/main/graph/badge.svg)](https://codecov.io/gh/jxoesneon/FerroTeX)
[![License](https://img.shields.io/badge/license-Apache--2.0%20%2F%20MIT-blue)](LICENSE-CHOICE.md)
[![Ko-fi](https://img.shields.io/badge/Donate-Ko--fi-FF5E5B?logo=ko-fi&logoColor=white)](https://ko-fi.com/jxoesneon)

**FerroTeX** is a research-driven, high-performance, type-safe **LaTeX language platform** for modern editors.

It closes two long-standing gaps in the TeX ecosystem:

- **Source understanding**: fully parse LaTeX documents into a lossless syntax tree, build a workspace index, and provide industry-standard IDE features (completion, definitions, references, rename, outline, semantic tokens, formatting).
- **Build observability**: transform TeX engine output (`.log`, stdout/stderr) into structured events and deterministic diagnostics with provenance and explicit uncertainty.

This repository currently contains **project documentation and specifications** intended to guide implementation of:

- A **Rust** core server that:
  - parses LaTeX source into a fault-tolerant CST/AST
  - maintains project and symbol indices across multi-file workspaces
  - ingests TeX engine output and/or `.log` files into a typed event stream
  - serves features to editors via **Language Server Protocol (LSP)** and (optionally) **Debug Adapter Protocol (DAP)**.
- A **VS Code extension** that acts as a thin client, managing server lifecycle and UX.

## CLI Usage

FerroTeX provides a command-line interface for interacting with its tools.

### Parse

Parse a TeX log file and emit structured JSON events.

```bash
ferrotex parse main.log
```

### Watch

Watch a TeX log file for changes in real-time and stream JSON events as they occur. This is useful for integrating with build tools or editors.

```bash
ferrotex watch main.log
```

## Status: v0.20.1 (Stable Beta)

FerroTeX is currently in **Stable Beta**. The core engine is feature-complete for standard LaTeX workflows and validated against massive codebases.

**Latest Performance Metrics (M8 Pro):**

- **Log Parsing**: >50 MB/s (Zero-allocation event stream)
- **Syntax Parsing**: ~1.5ms per 10k LOC (Fault-tolerant)
- **Startup Time**: <50ms (Language Server)

## Key Features

### 1. Fault-Tolerant Parsing

FerroTeX uses a **lossless syntax tree** (Rowan) that preserves every whitespace and comment. It recovers gracefully from:

- Unclosed groups (`{...`) and environments (`\begin...`).
- Mismatched delimiters.
- Invalid syntax.

### 2. Semantic Analysis

- **Matrix Shape Verification**: Detects jagged rows in `matrix`, `cases`, and `aligned` environments, accounting for `\multicolumn` merges.
- **Reference checking**: Cross-file label resolution and citation validation.
- **Macro Analysis**: Abstract interpretation to detect infinite recursion loop risks.

### 3. Integrated Debugger (DAP)

Full **Debug Adapter Protocol** support for `tectonic` and `pdftex` (via log parsing):

- **Stepping**: Step-by-step execution through your TeX source.
- **Variables**: Inspect live register values (`\count0`, `\dimen0`) and macro definitions.
- **Tokio-powered**: Async I/O handles user interaction without freezing the build.

### 4. Build System

- **Determinisic**: Content-addressable builds with `ferrotex.lock`.
- **Reproducible**: Universal DAG execution model.

## Implementation Roadmap (high level)

See `ROADMAP.md`.

## Contributing

See `CONTRIBUTING.md` and `docs/development/setup.md`.

## Citation

If you use FerroTeX in academic work, see `CITATION.cff`.

## License

Licensed under either of:

- Apache License, Version 2.0 (`LICENSE-APACHE`)
- MIT license (`LICENSE-MIT`)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.
