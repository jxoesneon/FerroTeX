# Log Event IR (Specification)

## Status

- **Type:** Normative
- **Stability:** Draft (breaking changes allowed until first stable schema release)

## Design Requirements

- Zero-copy friendly: events SHOULD refer to the log buffer via byte spans.
- Versioned: schema MUST carry a version.
- Extensible: new event kinds should not break consumers.
- Uncertainty-aware: ambiguous interpretations must be represented.

## Terminology

- **Log buffer**: byte string representing engine output and/or `.log` file.
- **Span**: byte offsets into the log buffer.

## Schema Versioning

The IR MUST include:

- `schema_version: "0.x"`

A stable release will define a `1.0` schema with compatibility guarantees.

## Core Types

### Span

A span identifies provenance within the log buffer.

```json
{ "start": 1234, "end": 1301 }
```

- `start` is inclusive
- `end` is exclusive

### Confidence

A floating-point score in `[0, 1]`.

- `1.0` means “high confidence” (not mathematically certain)
- values below a configured threshold MAY be rendered as “uncertain” in the UI

### FileRef

A file reference MAY be one of:

- absolute path
- workspace-relative path
- unknown

To remain platform-neutral, the IR uses strings plus optional normalization metadata.

## Event Model

An event is a discriminated union:

```json
{
  "kind": "FileEnter",
  "span": { "start": 10, "end": 42 },
  "confidence": 0.95,
  "data": { "path": "./chapter1.tex" }
}
```

### Required Fields

- `kind` (string)
- `span` (Span)
- `confidence` (number)
- `data` (object)

### Event Kinds (initial set)

- `FileEnter { path }`
- `FileExit {}`
- `ErrorStart { message }`
- `ErrorLineRef { line: u32, source_excerpt?: string }`
- `ErrorContextLine { text }`
- `Warning { message }`
- `Info { message }`
- `OutputArtifact { path?: string, format?: string, role?: string }`
- `BuildSummary { success: bool }`

## Diagnostic Record

Diagnostics are emitted from one or more events.

```json
{
  "severity": "Error",
  "message": "Undefined control sequence",
  "file": "./main.tex",
  "range": { "start": { "line": 44, "character": 0 }, "end": { "line": 44, "character": 1 } },
  "confidence": 0.8,
  "provenance": {
    "log_span": { "start": 9001, "end": 9055 },
    "file_stack": ["./main.tex", "./chapter1.tex"]
  }
}
```

### Required Fields

- `severity` ∈ {`Error`, `Warning`, `Information`, `Hint`}
- `message` (string)
- `confidence` (number)
- `provenance.log_span` (Span)

### Optional Fields

- `file` (string)
- `range` (LSP-style range)
- `code` (string)
- `related` (array of related diagnostics)

## Stability Notes

Consumers (extension) MUST tolerate unknown `kind` values.
