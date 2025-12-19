# Testing Strategy

FerroTeX is a parser-heavy system; testing must combine correctness fixtures with robustness testing.

## Test Classes

## 1) Unit Tests

- Tokenization primitives
- Path recognition heuristics
- State machine transitions
- Confidence propagation rules

## 2) Golden Tests (Parser Output)

Golden tests validate that parsing a fixture log yields stable output.

- Input: `.log` fixtures (real and synthetic)
- Output: stable JSON event stream + diagnostics

Guidelines:

- Keep fixtures minimal and purpose-built.
- Add one fixture per bug class when possible.
- Always record why the fixture exists (in the file name or an adjacent README once fixtures exist).

## 3) Property / Fuzz Testing

The parser must be resilient to hostile input.

- Fuzz targets:
  - log normalization
  - tokenization
  - full parse pipeline

Success criteria:

- no panics
- bounded runtime / memory
- graceful recovery and error reporting

## 4) Integration Tests

End-to-end tests validate:

- engine runner → log ingestion → parser → LSP diagnostics

Where possible:

- run in CI with a pinned TeX distribution
- test multiple engines/adapters

## Regression Discipline

Every bug fix should include at least one of:

- a new fixture + golden update
- a new unit test covering the root cause
- a fuzz seed preserved as a regression input
