# Feature Matrix (v1.0.0)

## Status

- **Type:** Normative
- **Stability:** Stable (v1.0.0 Frozen)

## Purpose

Define the **official feature set** for FerroTeX `v1.0.0`.

## 1. Language Platform (Source Analysis)

### 1.1 Parsing

- **LP-1**: Fault-tolerant LaTeX lexer
- **LP-2**: Fault-tolerant CST for incomplete documents
- **LP-3**: Incremental reparse for `didChange`

### 1.2 Project Model

- **PM-1**: Multi-root workspace support
- **PM-2**: Entrypoint detection + override configuration
- **PM-3**: Include graph (`\\input`, `\\include`) with cycle detection
- **PM-4**: Resource link resolution (includegraphics/bib)

### 1.2.1 TeX Path Resolution

- **TP-1**: Workspace-relative path resolution baseline (including extension inference)
- **TP-2**: Environment-aware resolution (TEXINPUTS/output directory where applicable)
- **TP-3**: Optional `kpsewhich`-assisted resolution (opt-in, bounded)

### 1.3 Indexing

- **IDX-1**: Label definitions (`\\label{}`)
- **IDX-2**: Label references (`\\ref{}` family)
- **IDX-3**: Citation references (`\\cite{}` family)
- **IDX-4**: `.bib` parsing for bib keys (best-effort)
- **IDX-5**: Workspace symbol query surface

### 1.3.1 File Watching / Invalidation

- **FW-1**: Watch workspace files (`.tex`, `.bib`, `.sty`, `.cls`) and react to changes
- **FW-2**: Dependency-aware invalidation (reindex only affected entrypoints)

### 1.4 Source Diagnostics

- **SD-1**: Parse recovery diagnostics with ranges
- **SD-2**: Duplicate label diagnostics
- **SD-3**: Unresolved label reference diagnostics
- **SD-4**: Unresolved citation diagnostics

## 2. LSP Feature Set

### 2.1 Core

- **LSP-1**: `didOpen`/`didChange`/`didSave`/`didClose`
- **LSP-2**: `publishDiagnostics`
- **LSP-3**: cancellation support for long-running requests
- **LSP-17**: `workspace/didChangeConfiguration`
- **LSP-18**: `workspace/didChangeWatchedFiles`
- **LSP-19**: `workspace/executeCommand`

### 2.2 Authoring

- **LSP-4**: `completion` (commands/environments/labels/citations/paths)
- **LSP-5**: `signatureHelp` (best-effort)
- **LSP-6**: `codeAction` (quick fixes)

### 2.2.1 Completion Data Sources

- **CS-1**: Built-in completion inventory (versioned, deterministic)
- **CS-2**: Workspace discovery for safe local definitions (confidence-gated)
- **CS-3**: Optional bundled package metadata database (no runtime network)

### 2.3 Navigation

- **LSP-7**: `definition` (labels; citations if supported)
- **LSP-8**: `references` (labels; citations if supported)
- **LSP-9**: `rename` (labels; citations if supported)
- **LSP-10**: `documentSymbol` (outline)
- **LSP-11**: `workspace/symbol`

### 2.4 Structure & UX

- **LSP-12**: `foldingRange`
- **LSP-13**: `documentLink` (includes/resources)
- **LSP-14**: `semanticTokens/full`
- **LSP-15**: `formatting` and `rangeFormatting`
- **LSP-16**: `hover` (symbol info + provenance)

### 2.5 Formatting Backend

- **FB-1**: Built-in structural formatter (safe, idempotent)
- **FB-2**: Optional external `latexindent` integration (opt-in, deterministic invocation)

## 3. Build Observability (Engine/Log)

- **BO-1**: Runner adapters (latexmk, tectonic). Tectonic MUST be supported for zero-config workflows.
- **BO-2**: Streaming `.log` follow
- **BO-3**: Structured parsing of `!` error blocks
- **BO-4**: Structured parsing of common warnings
- **BO-5**: Provenance log excerpts
- **BO-6**: Confidence-aware file/line association

### 3.0 Build Orchestration

- **BO-7**: Multi-phase build sessions (latexmk delegated mode + explicit pipeline mode)

### 3.1 Export / Build Outputs

- **EX-1**: Explicit build target selection (pdf/dvi/ps/html/svg)

### 3.2 SyncTeX (PDF Preview Workflow)

- **SX-1**: SyncTeX artifact support for PDF builds (`.synctex.gz` published as an artifact)
- **SX-2**: Forward search (source → PDF) command
- **SX-3**: Inverse search (PDF → source) best-effort integration

### 3.3 UX & Feedback (Visuals)

- **UX-4**: Status Bar Element (Spinning icon + "Building..." text) for active builds.
- **UX-5**: Build Failure Notification (Toast with "Open Logs" action) for fatal errors.
- **EX-4**: Integrated PDF Viewer via VS Code **Custom Editor API** (Webview-based) with SyncTeX.

- **QG-4**: Deterministic output for exported schemas

### 4.1 Performance

- **PF-1**: Published performance SLO targets for key LSP operations
- **PF-2**: Benchmarks include interactive latency and incremental update metrics

## 5. Operational and Compatibility Guarantees

- **OC-1**: `schema_version: 1.0` for build/log event IR and diagnostic records
- **OC-2**: Documented configuration keys and stability policy
- **OC-3**: Documented supported engines/runners and known limitations
- **OC-4**: Published compatibility matrix for engines/runners/targets/platforms

## 6. Future Scope (Post-v1.0.0)

The following features are deferred to future major/minor versions (e.g., v1.1.0+):

### Language Platform

- **LP-4**: Basic environment/group structure recovery
- **LP-5**: Deep math semantics (matrix validation, math mode structure)

### Build Orchestration

- **BO-8**: Bibliography and index tool support (biber/bibtex, makeindex/xindy) with rerun caps
- **BO-9**: Package manager integration (auto-install missing packages via tlmgr/miktex)

### Export / Build Outputs

- **EX-2**: Artifact discovery and reporting as structured events
- **EX-3**: Ability to open/reveal primary artifact from the editor
- **EX-4**: Integrated PDF Viewer (Webview-based) with SyncTeX support

### UX & Delighters

- **UX-1**: Math Hover Preview (render equations in tooltip without full compilation)
- **UX-2**: Input Delighters (Magic Comments `%!TEX`, Auto-Fraction, Bracket Matching)
- **UX-3**: Image Paste Wizard (Clipboard -> File + \includegraphics)
