# FerroTeX

[![CI](https://github.com/jxoesneon/FerroTeX/actions/workflows/ci.yml/badge.svg)](https://github.com/jxoesneon/FerroTeX/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/jxoesneon/FerroTeX/branch/main/graph/badge.svg)](https://codecov.io/gh/jxoesneon/FerroTeX)
[![License](https://img.shields.io/badge/license-Apache--2.0%20%2F%20MIT-blue)](LICENSE-CHOICE.md)

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

## Status

- **Phase:** Design + research proposal (pre-implementation).
- **Primary deliverable (near-term):** Offline parsers that emit stable, versioned schemas for:
  - build/log events and diagnostics
  - source syntax trees (CST/AST) and workspace indices

## Goals

- **Industry-standard language features**
  - Full-document parsing with incremental, error-tolerant recovery.
  - Cross-file indexing (includes, labels/refs, citations, commands/environments).
  - LSP features expected of mature language tooling.

- **Correctness-first diagnostics (source + build)**
  - Source diagnostics from parsing and semantic resolution.
  - Build diagnostics from engine/log observability.
  - Explicit uncertainty modeling for ambiguous mappings.

- **High throughput / low latency**
  - Zero-copy or low-allocation parsing strategies.
  - Incremental updates suitable for live editor feedback.

- **Engine-agnostic foundations**
  - Support the dominant engines (pdfTeX, XeTeX, LuaTeX) via a unified event IR.

- **Research-grade evaluation**
  - Benchmarkable metrics, datasets, and reproducible methodology.

## Non-Goals (initially)

- Proving full causality of TeX errors to user source (undecidable in general).
- Replacing TeX engines or re-implementing TeX.
- Perfect semantic understanding of arbitrary package output.

## Repository Layout

- `docs/`
  - Specifications, architecture, protocol surfaces (LSP/DAP), parsing model, and evaluation methodology.
- `docs/adrs/`
  - Architecture Decision Records (ADRs) documenting key design choices.

## Key Documents

- `docs/README.md` — Documentation index.
- `docs/architecture/overview.md` — System architecture and data flow.
- `docs/spec/feature-matrix.md` — v1.0.0 feature matrix and scope control.
- `docs/spec/language-platform.md` — Source parsing, indexing, and language feature model.
- `docs/spec/project-model.md` — Workspace, include graph, and file classification.
- `docs/spec/tex-path-resolution.md` — TeX-like path resolution (TEXINPUTS/kpathsea strategies).
- `docs/spec/symbol-index.md` — Symbols, references, and query APIs.
- `docs/spec/lsp.md` — LSP contract and behaviors.
- `docs/spec/source-ir.md` — CST/AST and index export format (for tests and research).
- `docs/spec/export.md` — Build targets and artifact export (pdf/dvi/ps/html/svg).
- `docs/spec/synctex.md` — Forward/inverse search workflow for PDF.
- `docs/spec/compatibility-matrix.md` — Supported engines/runners/targets/platforms for v1.0.0.
- `docs/spec/log-event-ir.md` — Build/log event IR and diagnostic representation.
- `docs/spec/log-grammar.md` — Log grammar, tokenization rules, wrap handling.
- `docs/research/evaluation-plan.md` — Metrics, datasets, baselines, reproducibility.

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
