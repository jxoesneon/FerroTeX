---
title: "crates/ferrotex-log/src/ir.rs"
type: source-code
language: rust
crate: ferrotex-log
layer: log-ir
tags:
  - crate/ferrotex-log
  - type/rust
  - layer/log-ir
  - log/events
summary: "Typed Intermediate Representation (IR) defining structured TeX log events, diagnostics, source spans, confidence scoring, and provenance tracking."
---

# `crates/ferrotex-log/src/ir.rs`

## Purpose & Architecture Role

`crates/ferrotex-log/src/ir.rs` defines the **canonical Intermediate Representation (IR)** for parsed TeX engine logs within FerroTex. TeX engines (`pdftex`, `xetex`, `luatex`) output unstructured, terminal-wrapped diagnostic streams that lack formal machine-readable schemas. This module establishes a strongly typed, versioned data model that transforms raw log output into actionable event streams and diagnostic records.

```text
┌────────────────────────────────────────────────────────┐
│ TeX Engine Log Output (pdfTeX, XeTeX, LuaTeX, etc.)    │
└───────────────────────────┬────────────────────────────┘
                            │ Raw byte stream / incremental chunks
                            ▼
           [[crates/ferrotex-log/src/parser.rs|log parser.rs]]
                            │ Emits stream of
                            ▼
                  LogEvent (IR Schema)
    ├── Span (log byte range start..end)
    ├── Confidence (0.0..1.0 uncertainty rating)
    └── EventPayload (Discriminated Union)
         ├── FileEnter / FileExit (File stack tracking)
         ├── ErrorStart / ErrorLineRef / ErrorContextLine
         ├── Warning (Overfull/Underfull boxes, package warnings)
         ├── Info (Engine/package informational messages)
         ├── OutputArtifact (PDF, aux, sync files)
         └── BuildSummary (success/failure)
                            │ Aggregated & converted
                            ▼
                    Diagnostic Record
    ├── Severity (Error, Warning, Information, Hint)
    ├── Message & File / Range (LspRange)
    └── Provenance (Log byte span + File stack)
                            │
                            ▼
     [[crates/ferrotexd/src/diagnostics/mod.rs|ferrotexd diagnostics]] ──► LSP Client (VS Code)
```

The IR serves several critical architectural goals:
1. **Decoupling Parsing from Consumption**: Insulates downstream consumers—such as [[crates/ferrotexd/src/diagnostics/mod.rs|ferrotexd diagnostics]] and the language server protocol daemon—from the volatile heuristics of raw log string parsing.
2. **Provenance Preservation**: Retains exact byte offsets in the engine log for every detected event, allowing IDE clients to trace diagnostics directly back to engine log excerpts as required by [[docs/spec/provenance.md|Provenance Model]].
3. **Uncertainty as a First-Class Concept**: Integrates numeric `Confidence` scores directly into events and diagnostics, acknowledging that heuristic log inference may be ambiguous (governed by [[docs/adrs/0002-uncertainty-as-first-class.md|ADR 0002: Uncertainty as First-Class]]).
4. **Stable Wire Protocol**: Formally versioned via `SCHEMA_VERSION` in [[crates/ferrotex-log/src/lib.rs|log lib.rs]] and designed according to [[docs/adrs/0001-typed-event-ir.md|ADR 0001: Typed Event IR]] and [[docs/spec/log-event-ir.md|Log Event IR Specification]].

---

## Key Symbols & Interfaces

### Core IR Data Structures

#### `Span`
Represents an exact contiguous slice within the log file buffer.
- `pub start: usize`: Start byte offset (inclusive).
- `pub end: usize`: End byte offset (exclusive).
- `pub fn new(start: usize, end: usize) -> Self`: Constructor.
- **Role**: Provides zero-copy friendly provenance tracking into the log stream.

#### `Confidence`
A newtype wrapper representing the parser's certainty in its semantic classification.
- `pub struct Confidence(pub f64)`: Floating-point certainty score between `0.0` (pure heuristic) and `1.0` (high confidence deterministic match).
- `impl Default for Confidence`: Returns `Confidence(1.0)`.

#### `LogEvent`
The atomic unit produced by the streaming parser.
- `pub span: Span`: Byte offset range of the event in the log file.
- `pub confidence: Confidence`: Confidence level of the heuristic parser.
- `pub payload: EventPayload`: Flattened enum carrying the specific variant data (`#[serde(flatten)]`).

---

### Event Kinds & Taxonomies

#### `EventPayload`
A tagged discriminated union (`#[serde(tag = "kind", content = "data")]`) representing every recognized TeX log phenomenon:

| Variant | Fields | Description & LaTeX Log Semantics |
|---|---|---|
| `FileEnter` | `path: String` | Engine opened a new input file (`(./main.tex` or `(/usr/share/.../pkg.sty)`). Pushes onto the file stack. |
| `FileExit` | *(none)* | Engine closed the active file (matching closing parenthesis `)`). Pops the file stack. |
| `ErrorStart` | `message: String` | Critical TeX error condition starting with `!` (e.g., `! Undefined control sequence`). |
| `ErrorLineRef` | `line: u32`, `source_excerpt: Option<String>` | Source line number pointer (e.g., `l.42 \badcommand`) identifying exact point of failure. |
| `ErrorContextLine` | `text: String` | Continuation lines following an error reporting macro expansion state. |
| `Warning` | `message: String` | Non-fatal diagnostic warning. Covers LaTeX warnings (`LaTeX Warning:`), Package warnings (`Package hyperref Warning:`), and box dimension anomalies (`Overfull \hbox`, `Underfull \vbox`). |
| `Info` | `message: String` | Informational status notifications from engine packages, font loaders, or memory managers. |
| `OutputArtifact` | `path: Option<String>`, `format: Option<String>`, `role: Option<String>` | Emitted output files generated by the compilation cycle (`.pdf`, `.dvi`, `.aux`, `.synctex.gz`). |
| `BuildSummary` | `success: bool` | Overall status summary indicating whether the TeX engine exited cleanly. |

#### Semantic Event Categories
In downstream processing, events fall into specific functional buckets:
- **FileContext & FileStack**: `FileEnter`, `FileExit` track input inclusion hierarchies.
- **Errors**: `ErrorStart`, `ErrorLineRef`, `ErrorContextLine` form structured compiler errors.
- **Warnings & Overfull/Underfull Boxes**: `Warning` unifies package warnings with TeX horizontal/vertical box capacity diagnostics (`Overfull \hbox (15.2pt too wide)`, `Underfull \vbox`).
- **Informational**: `Info`, `OutputArtifact`, `BuildSummary`.

---

### Diagnostics & LSP Projection

#### `Diagnostic`
A standardized compiler diagnostic derived from one or more `LogEvent`s, tailored for LSP consumption.
- `pub severity: Severity`: Diagnostic level.
- `pub message: String`: Cleaned human-readable message text.
- `pub file: Option<String>`: Source file path associated with the diagnostic (resolved via the active `file_stack`).
- `pub range: Option<LspRange>`: Position in the source file, if identifiable.
- `pub confidence: Confidence`: Accuracy confidence score.
- `pub provenance: Provenance`: Log traceability data.

#### `Severity`
Diagnostic severity levels mapping directly to the Language Server Protocol (`DiagnosticSeverity`):
- `Severity::Error` $\to$ LSP Error (1)
- `Severity::Warning` $\to$ LSP Warning (2)
- `Severity::Information` $\to$ LSP Information (3)
- `Severity::Hint` $\to$ LSP Hint (4)

#### `LspRange` & `LspPosition`
Zero-indexed LSP coordinates locating diagnostics in LaTeX source files:
- `LspRange`: `start: LspPosition`, `end: LspPosition`.
- `LspPosition`: `line: u32`, `character: u32`.

#### `Provenance`
Encapsulates complete provenance information for auditability and editor features:
- `pub log_span: Span`: The byte span in the `.log` file that generated this diagnostic.
- `pub file_stack: Vec<String>`: Snapshot of the nested file hierarchy (`["main.tex", "sections/intro.tex"]`) at the moment the event was triggered.

---

## Serialization & Wire Format

All types in `ir.rs` derive `serde::Serialize` and `serde::Deserialize`.

- **Event Serialization**: `EventPayload` utilizes `tag = "kind"` and `content = "data"`, while `LogEvent` flattens the payload.
  ```json
  {
    "kind": "FileEnter",
    "span": { "start": 10, "end": 42 },
    "confidence": 1.0,
    "data": { "path": "./chapter1.tex" }
  }
  ```
- **Diagnostic Serialization**: Skips `None` values for `file` and `range`:
  ```json
  {
    "severity": "Error",
    "message": "Undefined control sequence",
    "file": "./main.tex",
    "range": {
      "start": { "line": 44, "character": 0 },
      "end": { "line": 44, "character": 1 }
    },
    "confidence": 0.85,
    "provenance": {
      "log_span": { "start": 9001, "end": 9055 },
      "file_stack": ["./main.tex", "./chapter1.tex"]
    }
  }
  ```
- **Schema Compatibility**: Defined under `SCHEMA_VERSION = "1.0.0"`. Consumers must tolerate unknown `kind` variants for forward compatibility.

---

## Provenance Tracking & Uncertainty

1. **FileStack Resolution**: TeX log files notoriously emit error messages without identifying which file caused them. The parser tracks nested parentheses (`(file.tex` and `)`) into a stack (`file_stack`). When an error or warning occurs, the top of the `file_stack` identifies the culprit file.
2. **Log Span Anchoring**: By recording `log_span` on every event and diagnostic, FerroTex enables IDE actions:
   - "Go to Log Source": Jump from an editor gutter squiggle directly to the exact line in `main.log`.
   - "Inspect Context": Inspect surrounding log output for macro expansions and font load logs.
3. **Confidence Scoring**: Heuristic parsers may misidentify text in parentheses as files (e.g., `(preloaded format=pdflatex)`). By tagging events with a `Confidence` score, downstream consumers can filter noisy false positives or surface confidence indicators in the user interface.

---

## Dependencies

- **External Crates**:
  - `serde::{Deserialize, Serialize}` — High-performance JSON serialization and schema generation.
- **Internal Crate Modules**:
  - [[crates/ferrotex-log/src/lib.rs|log lib.rs]] — Exports `ir` module and defines `SCHEMA_VERSION`.

---

## Consumers & Dependents

- [[crates/ferrotex-log/src/parser.rs|log parser.rs]]: Core producer that instantiates `LogEvent`, `Span`, `Confidence`, and `EventPayload` during streaming and one-shot log parsing.
- [[crates/ferrotexd/src/diagnostics/mod.rs|ferrotexd diagnostics]]: Consumes structured log events and diagnostics to publish LSP diagnostics to editors.
- `crates/ferrotexd/src/lib.rs`: Processes `LogEvent` payloads (`Warning`, `ErrorStart`, `FileEnter`) during build and document save workflows.
- `crates/ferrotex-cli/src/lib.rs`: CLI interface for headless build verification and JSON event inspection.

---

## Related Specs & ADRs

- [[docs/adrs/0001-typed-event-ir.md|ADR 0001: Typed Event IR]] — Architectural decision establishing typed events over unstructured regex scrapers.
- [[docs/adrs/0002-uncertainty-as-first-class.md|ADR 0002: Uncertainty as First-Class]] — Design mandate for the `Confidence` rating system.
- [[docs/spec/log-event-ir.md|Log Event IR Specification]] — Normative specification defining event schemas, field requirements, and JSON examples.
- [[docs/spec/provenance.md|Provenance Model Specification]] — Specification governing diagnostic traceability and log span references.
- [[docs/spec/diagnostic-mapping.md|Diagnostic Mapping Specification]] — Rules for translating log events into LSP diagnostic objects.
