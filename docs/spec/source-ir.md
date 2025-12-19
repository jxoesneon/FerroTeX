# Source IR (CST/AST + Index Export)

## Status

- **Type:** Normative
- **Stability:** Draft (0.x)

## Purpose

FerroTeX’s runtime data structures are optimized for incremental parsing and indexing.

This specification defines a **stable export format** for:

- offline analysis and debugging
- golden tests
- research artifacts

This is intentionally separate from the build/log event IR.

## Schema Version

Exports MUST include:

- `schema_version: "0.x"`

## Core Concepts

- **DocumentSnapshot**: a particular text version of a document.
- **Token**: a lexical unit with a source range.
- **CST Node**: a lossless, hierarchical representation.
- **AST Node**: an optional lowered representation for selected constructs.
- **IndexRecord**: a queryable semantic record (labels, citations, etc.).

## Ranges

Ranges use LSP-style 0-indexed positions.

```json
{
  "start": { "line": 0, "character": 0 },
  "end": { "line": 0, "character": 1 }
}
```

## DocumentSnapshot

```json
{
  "schema_version": "0.2",
  "uri": "file:///.../main.tex",
  "version": 12,
  "language": "latex",
  "tokens": [
    /* Token[] */
  ],
  "cst": {
    /* Node */
  },
  "ast": {
    /* Node | null */
  },
  "index": [
    /* IndexRecord[] */
  ],
  "diagnostics": [
    /* DiagnosticRecord[] */
  ]
}
```

## Token

```json
{
  "kind": "CommandName",
  "text": "\\section",
  "range": { "start": { "line": 10, "character": 0 }, "end": { "line": 10, "character": 8 } },
  "confidence": 1.0
}
```

Tokens SHOULD be bounded in size for export. If a token is huge, export MAY truncate `text` and preserve full span.

## Node (CST/AST)

Nodes are discriminated unions.

```json
{
  "kind": "Environment",
  "range": { "start": { "line": 20, "character": 0 }, "end": { "line": 30, "character": 0 } },
  "children": [
    /* Node[] */
  ],
  "data": { "name": "figure" },
  "confidence": 0.95
}
```

### Required Fields

- `kind`
- `range`
- `children`
- `data`
- `confidence`

## IndexRecord

Index records align with `docs/spec/symbol-index.md`.

```json
{
  "kind": "LabelDefinition",
  "name": "sec:intro",
  "uri": "file:///.../main.tex",
  "range": { "start": { "line": 42, "character": 8 }, "end": { "line": 42, "character": 17 } },
  "confidence": 0.9
}
```

## DiagnosticRecord

The diagnostic record format is shared with build diagnostics.

- See `docs/spec/log-event-ir.md` (diagnostic record section)
- Codes should follow `docs/spec/diagnostic-codes.md`

## Compatibility Requirements

Consumers MUST:

- ignore unknown node kinds
- ignore unknown token kinds
- ignore unknown fields
