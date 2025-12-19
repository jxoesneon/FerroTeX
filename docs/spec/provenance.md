# Provenance Model

## Status

- **Type:** Normative
- **Stability:** Draft

## Motivation

A diagnostic without traceability is difficult to trust. FerroTeX treats provenance as a first-class output.

## Requirements

- Every event MUST contain a `log_span` referencing the source log buffer.
- Every diagnostic MUST contain provenance that allows the user (or tooling) to locate the originating log excerpt.

## Provenance Fields

Diagnostics SHOULD include:

- `provenance.log_span`
- `provenance.log_excerpt` (optional, bounded length)
- `provenance.file_stack` (optional)
- `provenance.engine` (optional: engine name/version)

## Bounding and Safety

- Excerpts MUST be length-bounded.
- Avoid including absolute paths unless explicitly configured.

## UI Implications

The VS Code extension SHOULD provide a quick way to:

- open the log excerpt
- show file stack at time of emission
- display confidence
