# SyncTeX Workflow (Forward/Inverse Search)

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

Industry-standard LaTeX editing requires high-quality synchronization between:

- source positions in `.tex` files
- rendered PDF positions

This is commonly achieved through **SyncTeX**.

FerroTeX treats SyncTeX as a first-class workflow:

- **Forward search**: source → PDF location
- **Inverse search**: PDF location → source

## Requirements

- The build runner MUST be able to enable SyncTeX when producing PDF.
- The system MUST expose forward search as an editor command.
- The system SHOULD support inverse search via viewer integration.

## Engine Flags

For PDF-producing engines, SyncTeX is typically enabled via:

- `-synctex=1`

The runner MUST ensure SyncTeX behavior is explicit and reproducible.

## Artifact Model

A PDF build with SyncTeX produces artifacts:

- PDF (primary)
- `.synctex.gz` (secondary)

The runner SHOULD publish both using `OutputArtifact` events (see `log-event-ir.md`) with:

- `format = "pdf"` / `format = "synctex"`
- `role = "primary"` / `role = "secondary"`

## Forward Search

### Server Behavior

The server SHOULD expose:

- `ferrotex.forwardSearch` command

Parameters (conceptual):

- `uri`
- `position` (line/character)

The server resolves:

- project entrypoint
- current PDF artifact
- SyncTeX file path

and returns a viewer instruction payload.

### Extension Behavior

The extension SHOULD:

- call `ferrotex.forwardSearch`
- open the configured PDF viewer (or VS Code-integrated viewer)
- pass the SyncTeX jump instruction

## Inverse Search

Inverse search is viewer-dependent.

The extension SHOULD support inverse search where feasible by:

- registering a URI handler or listening endpoint (viewer integration dependent)
- mapping incoming SyncTeX coordinates back to source locations

If inverse search cannot be supported on a platform/viewer, the extension MUST document this limitation.

## Failure Modes

- Missing `.synctex.gz`
- Multiple PDFs in multi-project workspace
- Stale SyncTeX after partial rebuild

In these cases, FerroTeX MUST fail gracefully:

- surface a diagnostic or user-facing error
- allow fallback to opening the PDF without jumping
