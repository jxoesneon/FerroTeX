# ADR 0002: Uncertainty as a First-Class Output

- **Status:** Accepted
- **Date:** 2025-12-18

## Context

Log interpretation has unavoidable ambiguity:

- parentheses in filenames
- wrapped paths
- missing close parens
- package outputs that mimic tokens

Traditional tools often guess silently, producing confidently wrong diagnostics.

## Decision

Represent uncertainty explicitly via:

- confidence scores on events and diagnostics
- the ability to emit diagnostics without a file/range mapping

The UI should surface low confidence distinctly.

## Alternatives Considered

- Always guess the “most likely” mapping
  - rejected: harms trust and makes debugging harder

- Refuse to emit any diagnostic when uncertain
  - rejected: loses useful information; better to report with uncertainty

## Consequences

- Downstream consumers must display confidence appropriately.
- Evaluation includes calibration of confidence vs correctness.
