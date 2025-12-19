# Data Flow

## End-to-End Flow

### 1) Source Change

The user edits `.tex` files in VS Code.

The editor sends incremental text synchronization events to the server.

### 2) Source Analysis (Immediate)

On `didOpen`/`didChange`/`didSave`, the server updates the **document store**:

- snapshot text
- update CST/AST incrementally
- update per-document index entries

The server then updates **workspace indices** and publishes:

- source diagnostics (parse recovery, unresolved references)
- language features (completion, definition, references, outline, semantic tokens)

### 3) Build Trigger

One of:

- explicit command (compile)
- save-triggered compilation
- file watcher-triggered compilation

The VS Code extension triggers server commands; the server triggers engine execution.

### 4) Engine Execution

The server uses an adapter (latexmk / tectonic / direct engine).

Outputs are ingested through one or both channels:

- **stdout/stderr capture** (low latency)
- **log file tailing** (canonical reference)

### 5) Build Parsing + Reconstruction

The parser produces:

- a typed event stream
- a reconstructed file context stack timeline

The reconstruction layer associates diagnostics with:

- file path (best-effort)
- line reference (when present)
- confidence score

### 6) Diagnostics Publication

The server emits LSP diagnostics.

Diagnostics may be produced by:

- **Source pipeline** (CST/AST + indexing)
- **Build pipeline** (engine/log observability)

Diagnostics are stable under incremental updates:

- newly appended bytes should only affect downstream diagnostics if they complete a previously partial structure or introduce new diagnostics

### 7) User Interaction

The user clicks diagnostics, opens log excerpts, or triggers code actions.

## Observability Goals

- Ability to show “why this diagnostic is mapped here” using:
  - log byte spans (for build diagnostics)
  - file stack at time of build diagnostic emission
  - CST node identity / source range (for source diagnostics)

## Failure Modes

- Ambiguous file transitions (parenthesis-in-filename)
- Wrapped lines breaking path recognition
- Engine outputs mimicking file stack tokens

In these cases, outputs MUST degrade gracefully:

- emit diagnostics with reduced confidence
- preserve raw log excerpts for user inspection
