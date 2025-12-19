# CLI Specification (`ferrotex-cli`)

## Status

- **Type:** Normative
- **Stability:** Draft

## Purpose

The CLI provides offline tooling:

- parse `.log` files to structured JSON
- run benchmarks
- validate fixtures

It is the foundation for golden tests and reproducible research artifacts.

## Commands

### `ferrotex-cli parse <path-to-log>`

Parses a `.log` file and emits:

- event stream (typed IR)
- diagnostics
- summary statistics

Output formats (planned):

- JSON to stdout
- optional `--output <path>`

Key options (planned):

- `--schema-version <v>`
- `--include-provenance`
- `--confidence-threshold <0..1>`

### `ferrotex-cli bench`

Runs benchmark suite.

### `ferrotex-cli validate-fixtures`

Validates fixtures and golden outputs.

## Output Stability

The CLI output MUST be deterministic for a given input log, configuration, and schema version.

## Exit Codes

- `0`: success
- `1`: parse failure or invalid input
- `2`: internal error (should be rare; indicates bug)
