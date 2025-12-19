# Formatting Specification

## Status

- **Type:** Normative
- **Stability:** Draft

## Scope

Formatting in LaTeX must be conservative.

FerroTeX aims for:

- stable whitespace formatting
- safe indentation for environments/groups
- non-destructive behavior with comments and math

## Formatting Modes

- `textDocument/formatting` (whole document)
- `textDocument/rangeFormatting` (selected region)

## Rules (initial)

- Indent nested environments.
- Preserve comment-only lines.
- Do not reflow paragraphs unless explicitly enabled.
- Preserve math content verbatim by default.

## Safety and Idempotence

Formatting MUST be:

- idempotent (formatting twice yields same result)
- bounded (no exponential behavior)

## Configuration

Formatting behavior is controlled via configuration (see `configuration.md`).
