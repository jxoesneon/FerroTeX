# Feature Matrix (v1.0.0)

## Status

- **Type:** Normative
- **Stability:** Stable (v1.0.0 Frozen)

## Purpose

Define the **official feature set** for FerroTeX `v1.0.0`.

## 1. Language Platform (Source Analysis)

### 1.1 Parsing

- [ ] ****: Fault-tolerant LaTeX lexer
- [ ] ****: Fault-tolerant CST for incomplete documents
- [ ] ****: Incremental reparse for `didChange`

### 1.2 Project Model

- [ ] ****: Multi-root workspace support
- [ ] ****: Entrypoint detection + override configuration
- [ ] ****: Include graph (`\\input`, `\\include`) with cycle detection
- [ ] ****: Resource link resolution (includegraphics/bib)

### 1.2.1 TeX Path Resolution

- [ ] ****: Workspace-relative path resolution baseline (including extension inference)
- [ ] ****: Environment-aware resolution (TEXINPUTS/output directory where applicable)
- [ ] ****: Optional `kpsewhich`-assisted resolution (opt-in, bounded)

### 1.3 Indexing

- [ ] ****: Label definitions (`\\label{}`)
- [ ] ****: Label references (`\\ref{}` family)
- [ ] ****: Citation references (`\\cite{}` family)
- [ ] ****: `.bib` parsing for bib keys (best-effort)
- [ ] ****: Workspace symbol query surface

### 1.3.1 File Watching / Invalidation

- [ ] ****: Watch workspace files (`.tex`, `.bib`, `.sty`, `.cls`) and react to changes
- [ ] ****: Dependency-aware invalidation (reindex only affected entrypoints)

### 1.4 Source Diagnostics

- [ ] ****: Parse recovery diagnostics with ranges
- [ ] ****: Duplicate label diagnostics
- [ ] ****: Unresolved label reference diagnostics
- [ ] ****: Unresolved citation diagnostics

## 2. LSP Feature Set

### 2.1 Core

- [ ] ****: `didOpen`/`didChange`/`didSave`/`didClose`
- [ ] ****: `publishDiagnostics`
- [ ] ****: cancellation support for long-running requests
- [ ] ****: `workspace/didChangeConfiguration`
- [ ] ****: `workspace/didChangeWatchedFiles`
- [ ] ****: `workspace/executeCommand`

### 2.2 Authoring

- [ ] ****: `completion` (commands/environments/labels/citations/paths)
- [ ] ****: `signatureHelp` (best-effort)
- [ ] ****: `codeAction` (quick fixes)

### 2.2.1 Completion Data Sources

- [ ] ****: Built-in completion inventory (versioned, deterministic)
- [ ] ****: Workspace discovery for safe local definitions (confidence-gated)
- [ ] ****: Optional bundled package metadata database (no runtime network)

### 2.3 Navigation

- [ ] ****: `definition` (labels; citations if supported)
- [ ] ****: `references` (labels; citations if supported)
- [ ] ****: `rename` (labels; citations if supported)
- [ ] ****: `documentSymbol` (outline)
- [ ] ****: `workspace/symbol`

### 2.4 Structure & UX

- [ ] ****: `foldingRange`
- [ ] ****: `documentLink` (includes/resources)
- [ ] ****: `semanticTokens/full`
- [ ] ****: `formatting` and `rangeFormatting`
- [ ] ****: `hover` (symbol info + provenance)

### 2.5 Formatting Backend

- [ ] ****: Built-in structural formatter (safe, idempotent)
- [ ] ****: Optional external `latexindent` integration (opt-in, deterministic invocation)

## 3. Build Observability (Engine/Log)

- [ ] ****: Runner adapters (latexmk, tectonic). Tectonic MUST be supported for zero-config workflows.
- [ ] ****: Streaming `.log` follow
- [ ] ****: Structured parsing of `!` error blocks
- [ ] ****: Structured parsing of common warnings
- [ ] ****: Provenance log excerpts
- [ ] ****: Confidence-aware file/line association

### 3.0 Build Orchestration

- [ ] ****: Multi-phase build sessions (latexmk delegated mode + explicit pipeline mode)

### 3.1 Export / Build Outputs

- [ ] ****: Explicit build target selection (pdf/dvi/ps/html/svg)

### 3.2 SyncTeX (PDF Preview Workflow)

- [ ] ****: SyncTeX artifact support for PDF builds (`.synctex.gz` published as an artifact)
- [ ] ****: Forward search (source → PDF) command
- [ ] ****: Inverse search (PDF → source) best-effort integration

### 3.3 UX & Feedback (Visuals)

- [ ] ****: Status Bar Element (Spinning icon + "Building..." text) for active builds.
- [ ] ****: Build Failure Notification (Toast with "Open Logs" action) for fatal errors.
- [ ] ****: Integrated PDF Viewer via VS Code **Custom Editor API** (Webview-based) with SyncTeX.

- [ ] ****: Deterministic output for exported schemas

### 4.1 Performance

- [ ] ****: Published performance SLO targets for key LSP operations
- [ ] ****: Benchmarks include interactive latency and incremental update metrics

## 5. Operational and Compatibility Guarantees

- [ ] ****: `schema_version: 1.0` for build/log event IR and diagnostic records
- [ ] ****: Documented configuration keys and stability policy
- [ ] ****: Documented supported engines/runners and known limitations
- [ ] ****: Published compatibility matrix for engines/runners/targets/platforms

## 6. Future Scope (Post-v1.0.0)

The following features are deferred to future major/minor versions (e.g., v1.1.0+):

### Language Platform

- [ ] ****: Basic environment/group structure recovery
- [ ] ****: Deep math semantics (matrix validation, math mode structure)

### Build Orchestration

- [ ] ****: Bibliography and index tool support (biber/bibtex, makeindex/xindy) with rerun caps
- [ ] ****: Package manager integration (auto-install missing packages via tlmgr/miktex)

### Export / Build Outputs

- [ ] ****: Artifact discovery and reporting as structured events
- [ ] ****: Ability to open/reveal primary artifact from the editor
- [ ] ****: Integrated PDF Viewer (Webview-based) with SyncTeX support

### UX & Delighters

- [ ] ****: Math Hover Preview (render equations in tooltip without full compilation)
- [ ] ****: Input Delighters (Magic Comments `%!TEX`, Auto-Fraction, Bracket Matching)
- [ ] ****: Image Paste Wizard (Clipboard -> File + \includegraphics)
