# Language Platform Specification

## Status

- **Type:** Normative
- **Stability:** Draft

## Scope

This specification defines FerroTeX as a **LaTeX language platform**.

It covers:

- the document model (snapshots)
- parsing (fault-tolerant CST/AST)
- project model (workspace graph)
- indexing (symbols and references)
- feature producers (completion, navigation, semantic tokens, formatting)

Build/log observability is specified separately (see `log-event-ir.md`, `log-grammar.md`).

## Principles

- **P1: Lossless parsing first**
  - Prefer a CST that preserves tokens and trivia so formatting and refactors remain safe.

- **P2: Error tolerance**
  - Parsing must succeed even on incomplete documents.
  - Diagnostics must distinguish between hard errors and recovered states.

- **P3: Incrementality**
  - Reparse minimal affected regions on `didChange`.
  - Support cross-file incremental indexing.

- **P4: Separation of concerns**
  - Parsing produces syntax trees.
  - Indexing builds semantic views.
  - LSP responds from cached structures, not ad-hoc scanning.

## Language Coverage Goals

FerroTeX targets **practical completeness** for modern LaTeX editing.

The system MUST support (baseline):

- commands (`\\command`)
- environments (`\\begin{env}` / `\\end{env}`)
- groups (`{...}`)
- math mode transitions (`$...$`, `\\[ ... \\]`, environments)
- comments (`% ...`)
- file inclusion (`\\input`, `\\include`, `\\subfile` where feasible)
- resource references (`\\includegraphics`, bibliography inputs)

The system SHOULD support (progressive):

- argument parsing for common commands (best-effort)
- `expl3`-style constructs as a compatibility goal

## Semantic Limits and Safety

LaTeX is not a context-free language in general. Macro expansion, catcode changes, and package-defined syntax can invalidate purely static interpretations.

Therefore:

- The CST/AST is intended to be **structural and lossless**, not a full engine execution model.
- Semantic features (indexing, completion, rename) MUST be **confidence-gated**.
- When confidence is low, FerroTeX MUST prefer being conservative (e.g., do not offer a rename) rather than performing unsafe edits.

## Document Model

### Snapshots

For each open document, the server maintains:

- `uri`
- `version` (LSP version)
- `text` (snapshot)
- `cst` (fault-tolerant)
- `ast` (optional lowered form)

### Fault Tolerance

The CST MUST be constructible from any byte sequence representable in the editor.

- Unknown sequences should become `ErrorNode` entries with spans.
- Recovery should be local and bounded.

## Parsing Pipeline

1. **Lexing**
   - produces tokens (command, text, brace, bracket, comment, math delimiter, etc.)
2. **CST construction**
   - produces a tree capturing grouping and environment structure
3. **Lowering (optional)**
   - produces AST nodes for selected constructs used by indexing

## Indexing

The index is a set of queryable tables built from one or more documents:

- labels (`\\label{...}`)
- refs (`\\ref{...}`, `\\autoref{...}`, etc.)
- citations (`\\cite{...}` variants)
- bibliography keys (from `.bib` parsing if enabled)
- command/environment definitions (local `\\newcommand`, `\\newenvironment`, etc.)
- package usage (`\\usepackage{...}`)

The index MUST support:

- symbol lookup by name
- reference lookup by name
- reverse reference queries

## Feature Producers

Feature producers consume the CST/AST + indices:

- completion
- definition
- references
- rename
- document symbols
- semantic tokens
- formatting

Each producer MUST:

- be cancellable
- be bounded in time
- avoid reparsing the world per request

## Diagnostics

Source diagnostics are produced from:

- lex/parse recovery errors
- unresolved references (label/cite)
- duplicate label definitions
- malformed `\\begin`/`\\end` pairs (when detectable)

Diagnostics MUST use stable codes (see `diagnostic-codes.md`).

## Interactions with Build Observability

Build diagnostics can be enhanced by source analysis:

- map build diagnostic line numbers to ranges
- attach related information (symbol context)

However:

- build parsing must remain correct even when source parsing is incomplete.
