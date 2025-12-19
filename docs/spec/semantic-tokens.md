# Semantic Tokens Specification

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

Provide semantic highlighting beyond regex-based syntax coloring.

## Token Categories

The server SHOULD at minimum classify:

- command names
- environment names
- math delimiters / math content regions
- comments
- strings/paths in include-like commands
- label/citation keys

## Source of Truth

Semantic tokens MUST be produced from:

- CST/AST structure
- symbol index (for label/citation roles)

## LSP Support

- `textDocument/semanticTokens/full`
- `textDocument/semanticTokens/delta` (target)

## Performance

- Prefer incremental token updates for open documents.
- Avoid scanning full documents on every request.

## Degradation

If parsing is incomplete:

- fall back to best-effort tokenization
- reduce confidence and avoid incorrect classifications
