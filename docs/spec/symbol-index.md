# Symbol Index Specification

## Status

- **Type:** Normative
- **Stability:** Draft

## Goals

- Provide fast queries for navigation, completion, and diagnostics.
- Support multi-file LaTeX projects.
- Avoid false precision: represent uncertainty where semantics are unclear.

## Symbol Kinds

The index MUST support at least:

- `LabelDefinition` (`\\label{...}`)
- `LabelReference` (`\\ref{...}`, `\\autoref{...}`, etc.)
- `CitationReference` (`\\cite{...}` variants)
- `BibEntry` (from `.bib` parsing if enabled)
- `CommandDefinition` (`\\newcommand`, `\\DeclareMathOperator`, etc.)
- `EnvironmentDefinition` (`\\newenvironment`, etc.)
- `PackageUse` (`\\usepackage{...}`)
- `InputInclude` (`\\input`, `\\include`)

## Record Structure

Every indexed record MUST include:

- `name` (string)
- `kind` (symbol kind)
- `uri` (document)
- `range` (source range)
- `confidence` (0..1)

Records SHOULD include:

- `container` (e.g., surrounding environment/section)
- `raw_text` excerpt (bounded)

## Queries

The index MUST support:

- `find_definitions(kind, name)`
- `find_references(kind, name)`
- `workspace_symbols(query)`

## Incremental Updates

When a document changes:

- update only that document’s records
- recompute cross-file diagnostics that depend on global state:
  - duplicates
  - unresolved references

## Collision Policy

Name collisions are common in LaTeX.

- For label definitions, multiple definitions SHOULD emit a diagnostic.
- For command definitions, collisions MAY be allowed if scoped (future work).

## Mapping to LSP

- Definitions and references map to `Location` results.
- Document outline maps to `DocumentSymbol`.
- Rename requires a safe rewrite strategy; see `lsp.md` for capability gating.
