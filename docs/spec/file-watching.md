# File Watching and Incremental Invalidation

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

Industry-standard editor tooling must react to filesystem changes, not only open buffers.

This specification defines how FerroTeX reacts to:

- edits to `.tex` files not currently open
- `.bib` changes
- added/removed included files
- image/resource changes

## Inputs

- LSP `workspace/didChangeWatchedFiles`
- extension-side file watchers (if needed)

## Invalidation Model

When a file changes, the server MUST:

- update the project model (include graph)
- invalidate affected indices
- recompute dependent diagnostics (labels/citations)

### Dependency Tracking

The project model MUST maintain enough dependency information to answer:

- “which entrypoints include this file?”

## Watching Scope

Recommended watch patterns:

- `**/*.tex`
- `**/*.bib`
- `**/*.sty`, `**/*.cls`
- images referenced by `\includegraphics` (best-effort)

## Performance

- Invalidation MUST be bounded.
- Large-scale rescans SHOULD be debounced.

## Failure Modes

- Large workspaces with many files
- Frequent file events causing thrash

Mitigation:

- debounce
- incremental graph updates
- limit scan scope to project roots
