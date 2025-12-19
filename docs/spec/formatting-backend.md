# Formatting Backend Strategy

## Status

- **Type:** Normative
- **Stability:** Draft

## Goal

Provide safe formatting while acknowledging LaTeX’s complexity.

## Backend Options

### Option A: Built-in Structural Formatter (Required)

FerroTeX MUST provide a built-in formatter that:

- indents environments and groups
- preserves comments
- preserves math and verbatim-like regions by default

This formatter targets safety and idempotence.

### Option B: External Formatter Integration (Optional, but supported by v1.0.0)

FerroTeX MAY integrate with `latexindent` as an external tool.

Constraints:

- opt-in via configuration
- deterministic invocation
- sandboxed argument passing (no shell interpolation)

## Safety Rules

- Do not rewrite verbatim/minted/listings bodies unless explicitly enabled.
- Maintain idempotence.

## Diagnostics

If external formatting fails:

- surface a diagnostic or user-facing error
- fall back to built-in formatting if configured
