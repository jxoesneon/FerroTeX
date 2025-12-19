# Diagnostic Mapping

## Status

- **Type:** Normative
- **Stability:** Draft

## Goal

Map log-derived diagnostic evidence to editor-grade diagnostics:

- file path
- line/column range when possible
- severity
- message
- provenance and confidence

## Inputs

- Event stream (typed)
- Reconstructed file context stack timeline
- Workspace root(s)

## Mapping Rules

### Rule D1 — File Association

When an error or warning is emitted, associate it with the file at the top of the file context stack.

- If stack is empty, emit a diagnostic with `file = null`.
- Confidence MUST decrease when:
  - stack reconstruction is uncertain
  - the diagnostic block contains contradictory file hints

### Rule D2 — Line Reference

If an `ErrorLineRef` is present, map `l.<n>` to 1-indexed line `n`.

- Column is generally unknown.
- Default range SHOULD be a zero-width position at `(line-1, 0)`.

If a source excerpt exists, a best-effort column MAY be inferred via substring search in the corresponding source line, but MUST be confidence-gated.

### Rule D3 — Severity

- `!` blocks → `Error`
- known warning prefixes → `Warning`
- informational engine lines → `Information`

### Rule D4 — Related Information

Diagnostics SHOULD include related information when available:

- stack trace of files
- package provenance hints
- log excerpt spans

### Rule D5 — Confidence

Confidence is propagated from:

- parser confidence of relevant events
- stack reconstruction confidence
- line-reference confidence

Downstream UI should treat confidence as advisory; the core requirement is that uncertainty is _explicit_.

## Path Normalization

The server SHOULD attempt to normalize paths to workspace URIs.

- Prefer workspace-relative paths in diagnostics where possible.
- Retain original log path in provenance.

## Failure Modes

- Path is present but not resolvable in workspace
- Line reference exists for non-source files (e.g., `.sty`)
- Multiple diagnostics interleave (rare but possible)

In these cases, degrade gracefully and preserve provenance.
