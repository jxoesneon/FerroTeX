---
layout: default
title: LSP Contract
parent: Specifications
nav_order: 1
---

# Language Server Protocol (LSP) Contract

## Status

- **Type:** Normative
- **Stability:** Draft

## Design Principles

- The Rust server is the **single source of truth** for analysis and diagnostics.
- The VS Code extension should be a **thin** LSP client.
- Custom behaviors MUST be documented and versioned.

FerroTeX provides:

- **Source analysis** features derived from parsing and indexing LaTeX documents.
- **Build observability** features derived from TeX engine output and `.log` parsing.

## Baseline Capabilities (Required for MVP)

The server SHOULD implement:

- `textDocument/didOpen`, `didChange`, `didSave`, `didClose`
- `textDocument/publishDiagnostics`
- `workspace/executeCommand`

Optional enhancements:

- `textDocument/hover` for diagnostic provenance
- `textDocument/codeAction` for fix suggestions

## Industry-Standard Language Feature Set (Target)

The following capabilities represent the target feature set expected of mature language tooling.

### Document Intelligence

- `textDocument/documentSymbol` (outline)
- `textDocument/foldingRange`
- `textDocument/documentLink` (navigate `\\input`, `\\include`, `\\includegraphics`, bibliography resources)
- `textDocument/selectionRange`

### Navigation

- `textDocument/definition`
- `textDocument/references`
- `textDocument/rename` (labels, bib keys, local command/environment definitions where safe)
- `workspace/symbol`

Rename MUST be safety-gated:

- labels are supported
- citations MAY be supported when keys are known and edits are unambiguous
- command/environment rename MUST be refused unless the server can prove the definition/reference set is safe

### Authoring

- `textDocument/completion` (commands, environments, packages, labels, citations)
- `textDocument/signatureHelp` (macro argument hints where inferable)

### Semantics and Rendering

- `textDocument/semanticTokens` (full/delta)
- `textDocument/hover` (symbol info + provenance)

### Formatting

- `textDocument/formatting`
- `textDocument/rangeFormatting`

### Quality-of-Life

- `textDocument/codeAction` (quick fixes, refactors)
- `textDocument/codeLens` (optional; e.g., reference counts for labels)

### Workspace and Lifecycle

- `workspace/didChangeConfiguration`
- `workspace/didChangeWatchedFiles`
- Cancellation support for long-running requests

## Source vs Build Diagnostics

FerroTeX may publish diagnostics from multiple producers:

- **Source diagnostics**: syntax recovery errors, unresolved references, duplicate labels, invalid citation keys, etc.
- **Build diagnostics**: TeX engine errors and warnings, derived from `.log` parsing.

Diagnostics SHOULD set `source = "ferrotex"` and use stable codes (see `diagnostic-codes.md`).

## Commands

Commands are namespaced:

- `ferrotex.compile`
- `ferrotex.clean`
- `ferrotex.reparseLog`
- `ferrotex.openLogExcerpt`

Additional commands (target):

- `ferrotex.indexWorkspace`
- `ferrotex.openProjectGraph`

- `ferrotex.openOutput`
- `ferrotex.forwardSearch`

Each command MUST specify:

- parameters
- side effects
- expected error states

## Diagnostic Payload

Diagnostics MUST include:

- severity
- message
- range (best-effort)

Diagnostics SHOULD include:

- `code` (stable identifier)
- `source = "ferrotex"`
- `relatedInformation` containing provenance

Confidence SHOULD be surfaced via:

- `Diagnostic.codeDescription` (URL) OR
- `Diagnostic.data` via a custom extension (editor-dependent)

## Incremental Updates

The server SHOULD publish diagnostics:

- on compilation completion
- during streaming compilation when stable

The server MUST avoid flapping:

- partial parses should not cause frequent large-scale remapping unless confidence justifies it

## Configuration

Configuration keys are documented in `configuration.md`.

The server MUST react to configuration changes without requiring restart when feasible.

## File Watching

The server SHOULD handle:

- `workspace/didChangeWatchedFiles`

to support incremental invalidation and reindexing (see `file-watching.md`).
