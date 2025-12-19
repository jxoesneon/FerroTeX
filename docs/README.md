# FerroTeX Documentation Index

This directory contains the normative specifications and research-grade methodology documents for FerroTeX.

## Reading Order (recommended)

- `architecture/overview.md`
- `spec/feature-matrix.md`
- `spec/language-platform.md`
- `spec/project-model.md`
- `spec/tex-path-resolution.md`
- `spec/file-watching.md`
- `spec/symbol-index.md`
- `spec/completion.md`
- `spec/completion-sources.md`
- `spec/semantic-tokens.md`
- `spec/formatting.md`
- `spec/formatting-backend.md`
- `spec/performance-slos.md`
- `spec/source-ir.md`
- `spec/export.md`
- `spec/synctex.md`
- `spec/build-orchestration.md`
- `spec/compatibility-matrix.md`
- `spec/log-event-ir.md`
- `spec/log-grammar.md`
- `spec/diagnostic-mapping.md`
- `spec/lsp.md`
- `development/setup.md`
- `research/evaluation-plan.md`

## Architecture

- `architecture/overview.md` — components, responsibilities, invariants
- `architecture/data-flow.md` — end-to-end flow (engine → parser → IR → LSP → editor)

## Specifications (normative)

- `spec/feature-matrix.md` — v1.0.0 scope control and feature accounting
- `spec/language-platform.md` — source parsing, indexing, and feature producers
- `spec/project-model.md` — workspace roots, entrypoints, include graph
- `spec/tex-path-resolution.md` — path resolution policy (TEXINPUTS/kpathsea)
- `spec/file-watching.md` — filesystem watching and incremental invalidation
- `spec/symbol-index.md` — symbols/references and query APIs
- `spec/completion.md` — completion model and context detection
- `spec/completion-sources.md` — completion inventories explaining data provenance
- `spec/semantic-tokens.md` — semantic highlighting model
- `spec/formatting.md` — conservative formatting model

- `spec/formatting-backend.md` — built-in formatter and optional `latexindent` integration
- `spec/performance-slos.md` — latency/memory targets and cancellation requirements

- `spec/source-ir.md` — CST/AST and index export format (for tests and research)

- `spec/export.md` — build targets and artifact export (pdf/dvi/ps/html/svg)

- `spec/synctex.md` — forward/inverse search workflow and `.synctex.gz` artifacts
- `spec/build-orchestration.md` — multi-phase builds (latexmk/biber/makeindex)
- `spec/compatibility-matrix.md` — supported engines/runners/targets/platforms for v1.0.0

- `spec/log-event-ir.md` — typed event model and diagnostic record
- `spec/log-grammar.md` — log tokenization, parsing rules, wrap handling
- `spec/diagnostic-mapping.md` — mapping from events to source locations + confidence
- `spec/configuration.md` — configuration schema and resolution
- `spec/lsp.md` — LSP behaviors and custom extensions
- `spec/dap.md` — DAP exploration scope and semantics

## Extension

- `extension/overview.md` — VS Code extension responsibilities and UX contract

## Development

- `development/setup.md` — local development setup (Rust + Node)
- `development/testing.md` — test strategy (unit/golden/fuzz)
- `development/benchmarks.md` — benchmarking workflow and performance gates
- `development/release.md` — release process (to be implemented)

## Research & Evaluation

- `research/evaluation-plan.md` — datasets, metrics, baselines, threats to validity
- `research/reproducibility.md` — artifact packaging and CI expectations

## Architecture Decision Records

- `adrs/` — rationale and decisions for major design choices
