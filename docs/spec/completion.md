# Completion Specification

## Status

- **Type:** Normative
- **Stability:** Draft

## Goal

Provide fast, context-aware completion for LaTeX editing.

## Completion Categories

- Commands (built-in + indexed)
- Environments
- Labels (for `\\ref`-like commands)
- Citations (for `\\cite`-like commands)
- Packages (for `\\usepackage`)
- File paths (for `\\input`, `\\include`, `\\includegraphics`)

## Context Detection

Completion must be context-sensitive:

- if cursor is after `\\`, suggest commands
- if inside `{}` of `\\ref{...}`, suggest labels
- if inside `{}` of `\\cite{...}`, suggest bib keys
- if inside `\\begin{...}`, suggest environment names

Context detection SHOULD be based on:

- CST node ancestry
- local token neighborhood (bounded)

## Ranking

Ranking signals (best-effort):

- proximity (same file)
- reference count
- recency (recently used)
- exact prefix match

## Snippets

Completion items MAY include snippets for:

- `\\begin{env} ... \\end{env}`
- common environments (itemize, enumerate, figure, table)

## Performance

Completion MUST be:

- cancellable
- bounded in time
- served from indices where possible
