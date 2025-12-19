# Diagnostic Codes

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

Provide stable identifiers for diagnostics emitted by FerroTeX.

This supports:

- filtering and suppression
- analytics
- consistent UX

## Principles

- Codes identify **classes** of diagnostics, not specific messages.
- Codes are stable across releases once `1.0` schema is published.

## Code Namespace

- `FTX` prefix for FerroTeX.

## Initial Code Set (proposed)

### Source Analysis

- `FTX0100` — SourceParseRecovery: parser recovered; syntax may be incomplete
- `FTX0101` — UnmatchedEnvironment: detected mismatched `\\begin`/`\\end` (best-effort)
- `FTX0102` — UnmatchedGroup: detected unmatched `{`/`}` (best-effort)

- `FTX0200` — DuplicateLabelDefinition
- `FTX0201` — UnresolvedLabelReference

- `FTX0300` — UnresolvedCitation
- `FTX0301` — BibParseError (best-effort)

- `FTX0400` — IncludeCycleDetected
- `FTX0401` — IncludeResolutionFailed

### Log Parsing / Reconstruction

- `FTX1000` — ParserRecovery: ambiguity encountered; confidence reduced
- `FTX1001` — UnmatchedFileExit: `)` observed with empty file stack
- `FTX1002` — SuspiciousFileEnter: `(` observed but path recognition uncertain

### Engine Diagnostics (normalized)

- `FTX2000` — TeXError: normalized `!` error block
- `FTX2001` — LaTeXWarning: normalized `LaTeX Warning:`
- `FTX2002` — OverfullHBox
- `FTX2003` — UnderfullHBox

### Toolchain

- `FTX3000` — EngineInvocationFailed
- `FTX3001` — LogNotFound

## Mapping

Where feasible, map engine diagnostics into these categories while preserving the original message in `message` and raw excerpt in provenance.
