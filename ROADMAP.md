# FerroTeX Roadmap

This roadmap describes **versioned releases** for FerroTeX from design to an industry-standard LaTeX language platform.

FerroTeX targets two pillars:

- **Language platform**: source parsing + project model + indexing + LSP features.
- **Build observability**: engine execution + log ingestion + structured diagnostics.

## Guiding Principles

- **Structured truth over brittle heuristics**: prefer typed event models and explicit state reconstruction to ad-hoc regex matching.
- **Incremental by design**: every subsystem should support streaming and partial recomputation.
- **Engine/toolchain adapters at the edge**: keep parsing/reconstruction core independent from latexmk/tectonic specifics.
- **Measure everything**: performance and correctness claims require benchmarks and labeled datasets.

## Release Plan (Semantic Versioning)

Until `1.0.0`, breaking changes are allowed but MUST be documented in `CHANGELOG.md`.

For `v1.0.0` scope control, FerroTeX uses the normative **Feature Matrix**:

- `docs/spec/feature-matrix.md`
- Export behaviors are specified in:
  - `docs/spec/export.md`

Each release below includes:

- **Scope**: what is added
- **Acceptance criteria**: conditions for calling the release complete
- **Feature Matrix coverage**: which Feature Matrix IDs are delivered by that release

### v0.1.0 — Documentation and Specification Baseline (current)

**Scope**

- Normative specifications for:
  - log event IR and parsing strategy
  - LSP contract (draft)
  - language platform foundations (draft)
- Development and research methodology documents.

**Acceptance criteria**

- Documentation set is internally consistent and defines:
  - invariants
  - terminology
  - schema versioning rules

**Feature Matrix coverage**

- None (documentation-only).

### v0.2.0 — Offline Build/Log Parser MVP (JSON IR) (Completed)

**Scope**

- `ferrotex-cli parse <log>`:
  - stable JSON event/diagnostic schema (0.x)
  - file stack reconstruction
  - error blocks (`!`, `l.<n>`)
  - common warnings (LaTeX warnings, over/underfull boxes)
  - wrap handling (bounded joining)
- Golden tests for curated fixtures.
- Basic fuzz target for parser panic-freedom.

**Acceptance criteria**

- Deterministic output for fixtures.
- Parser does not panic on fuzz corpus.
- Emits explicit confidence for ambiguous file transitions.

**Feature Matrix coverage**

- BO-3, BO-4, BO-5, BO-6
- QG-1, QG-2 (initial)
- QG-4 (build/log IR determinism for fixtures)

### v0.3.0 — Streaming Log Ingestion + Incremental Diagnostics

**Scope**

- Append-only log following (`.log` tail) with synchronization anchors.
- Incremental updates without reparsing from byte 0.
- Confidence-gated interim diagnostics.

**Acceptance criteria**

- Incremental update time is bounded and documented in `docs/development/benchmarks.md`.
- Diagnostics stability: no large-scale remapping flapping on partial log appends.

**Feature Matrix coverage**

- BO-2

### v0.4.0 — Server Skeleton + VS Code Extension Bootstrap

**Scope**

- `ferrotexd` runs as a long-lived process (stdio LSP transport).
- VS Code extension:
  - server lifecycle
  - configuration plumbing
  - commands: compile/clean/reparse/open log excerpt
- Publish build diagnostics via LSP.

**Acceptance criteria**

- End-to-end: run a build, see diagnostics in the Problems pane.
- Provenance: each diagnostic can open its log excerpt.

**Feature Matrix coverage**

- LSP-1, LSP-2, LSP-3 (build diagnostics path + cancellation)
- LSP-19

### v0.5.0 — LaTeX Lexer + Fault-Tolerant CST (Single File)

**Scope**

- Source lexer for LaTeX tokens (commands, groups, comments, math delimiters).
- Fault-tolerant CST construction for a single `.tex` document.
- Source diagnostics:
  - basic parse recovery errors
  - unmatched group/environment constructs where detectable
- `textDocument/documentSymbol` for a single file (best-effort).

**Acceptance criteria**

- CST is produced for incomplete documents.
- Parsing is incremental for `didChange` (bounded reparse region).
- Document symbols do not require compilation.

**Feature Matrix coverage**

- LP-1, LP-2, LP-3, LP-4
- SD-1 (initial)
- LSP-1, LSP-2 (source diagnostics path)
- LSP-10 (initial, single-file)

### v0.6.0 — Project Model + Include Graph (Multi-File)

**Scope**

- Workspace project detection + configured entrypoints.
- Include graph resolution:
  - `\input`, `\include` (best-effort)
- `textDocument/documentLink` for includes/resources.
- Cross-file indexing pipeline foundation.

**Acceptance criteria**

- Project graph is stable under incremental edits.
- Cycles are detected and surfaced as diagnostics.

**Feature Matrix coverage**

- PM-1, PM-2, PM-3, PM-4
- LSP-13
- TP-1
- LSP-17
- LSP-18

### v0.7.0 — Label Index + Go-to-Definition/References/Rename (Labels)

**Scope**

- Index:
  - `\label{}` definitions
  - `\ref{}`-family references
- LSP:
  - `textDocument/definition` for label references
  - `textDocument/references` for labels
  - `textDocument/rename` for labels (safe rename within workspace)
- Diagnostics:
  - duplicate label definitions
  - unresolved label references

**Acceptance criteria**

- Rename updates all references in workspace with preview.
- Diagnostics update incrementally and deterministically.

**Feature Matrix coverage**

- IDX-1, IDX-2
- SD-2, SD-3
- LSP-7, LSP-8, LSP-9

### v0.8.0 — Bibliography Support + Citation Intelligence

**Scope**

- `.bib` parsing (best-effort; robust to common BibTeX syntax).
- Citation index:
  - `\cite{}` variants
  - completion for citation keys
- Diagnostics:
  - unresolved citation keys

**Acceptance criteria**

- Completion for cite keys responds in bounded time on medium corpora.
- Parser is resilient to malformed `.bib` entries.

**Feature Matrix coverage**

- IDX-3, IDX-4
- SD-4
- FW-1

### v0.9.0 — Completion, Semantic Tokens, Folding, Workspace Symbols

**Scope**

- Completion:
  - commands (built-in set + discovered definitions where safe)
  - environments
  - labels and citations
  - file paths for include-like commands
- Semantic tokens (full) from CST + index.
- Folding ranges for environments/groups (best-effort).
- `workspace/symbol` backed by the index.

**Acceptance criteria**

- Completion is cancellable and does not block the server loop.
- Semantic token classification degrades gracefully under parse errors.

**Feature Matrix coverage**

- IDX-5
- LSP-4
- LSP-5
- LSP-11
- LSP-12
- LSP-14
- LSP-16
- CS-1, CS-2

### v0.10.0 — Formatting + Code Actions + Quality Hardening

**Scope**

- Formatting:
  - document formatting
  - range formatting
- Code actions:
  - quick fixes for common diagnostics (e.g., create missing label placeholder)
- Performance and robustness:
  - expanded fuzzing
  - benchmark suite with historical tracking (mechanism TBD)

**Acceptance criteria**

- Formatting is idempotent.
- No regressions in benchmark targets.

**Feature Matrix coverage**

- LSP-6
- LSP-15
- QG-2 (expanded), QG-3 (initial)
- FB-1
- PF-1

### v0.11.0 — Engine Runner Adapters (latexmk/tectonic/direct)

**Scope**

- Runner adapters per `docs/spec/engine-adapters.md`.
  - **Tectonic support** prioritized for zero-config users.
- Unified build event stream across adapters.
- Improved diagnostic mapping using source analysis (line→range best-effort).

**Acceptance criteria**

- Same project can be compiled with at least two runners with consistent diagnostic IR.

**Feature Matrix coverage**

- BO-1
- EX-1, EX-2 (at least PDF + one alternate target where feasible)
- BO-7, BO-8
- SX-1

### v0.12.0 — Production UX Stabilization

**Scope**

- Stability work:
  - reduce diagnostic flapping
  - improve confidence calibration
- Extension UX polish:
  - consistent commands
  - confidence visualization
  - Extension UX polish:
  - consistent commands
  - confidence visualization
  - provenance view ergonomics
  - **Image Paste Wizard**: Handle clipboard image paste events.
  - **Magic Comments**: Respect `%!TEX root` and `%!TEX program`.
  - **Status Bar Integration**: Visual feedback for build state (`UX-4`).
  - **Notifications**: Actionable toasts for build failures (`UX-5`).

**Acceptance criteria**

- Manual acceptance suite documented and repeatable.

**Feature Matrix coverage**

- QG-1, QG-3
- QG-4 (source IR export determinism)
- EX-3
- SX-2
- PF-2

### v0.13.0 — Integrated Environment Complete (PDF + Packages + Math)

**Scope**

- **Integrated PDF Viewer**:
  - Webview-based PDF viewer in VS Code.
  - Bidirectional SyncTeX (click-to-jump).
- **Package Management**:
  - Detect missing packages from logs.
  - Prompt to specific `tlmgr` / `miktex` install commands.
- **Math Semantics & UX**:
  - Deep validation for math environments.
  - **Hover Preview**: Render LaTeX equations in editor tooltips (MathJax/KaTeX).

**Acceptance criteria**

- PDF Viewer renders correctly and synchronizes cursors.
- Missing package errors allow one-click install flow.

**Feature Matrix coverage**

- LP-5
- BO-9
- EX-4
- SX-2, SX-3 (Integrated)

### v0.15.0 — The "Intelligence" Update (Writing Experience)

**Scope**

- **Snippet Pack**: Comprehensive library of Math/Greek/Environment snippets (UX-2).
- **Magic Comments**: Support for `%!TEX root` and `%!TEX program` (PM-Override).
- **Dynamic Package Metadata**: Index loaded packages and provide completions from a structured database (CS-3).

**Acceptance criteria**

- `\alpha` tab-expands to `α`.
- `%!TEX root = ../main.tex` is respected without configuration.
- `\usepackage{tikz}` enables `\node` completions.

**Feature Matrix coverage**

- UX-2
- PM-Override
- CS-3

### v0.16.0 — The "Observability" Update (Feedback Loop)

**Scope**

- **Real-Time Log Streaming**: Migrate build adapter from buffered output to real-time `stdout`/`stderr` streaming (BO-2).
- **Human-Readable Errors**: Translation layer for common TeX log errors (UX-5).
- **Rich Hovers**: MathJax-rendered previews for equations and citations (UX-1).

**Acceptance criteria**

- Build logs appear line-by-line in the output panel.
- "Underfull \hbox" is explained as "Bad line break".
- Hovering `\begin{equation}` shows rendered math.

**Feature Matrix coverage**

- BO-2
- UX-5
- UX-1

### v0.17.0 — The "Ecosystem" Update (Deep Integration)

**Scope**

- **Package Manager Integration**: Detect missing packages and offer `tlmgr install` actions (BO-9).
- **Math Mode Semantics**: Deep parsing for matrix/align environments with validation (LP-5).

**Acceptance criteria**

- Missing package error offers "Install" button.
- Mismatched matrix delimiters are flagged.

**Feature Matrix coverage**

- BO-9
- LP-5

### v1.0.0 — The "Gold" Release (Ceremonial)

**Scope**

- **Final Stability Audit**: Zero critical bugs for 2 weeks.
- **Documentation Polish**: "Enjoy!" update to README.
- **SemVer Guarantee**: 1.0.0 schema lock.

**Acceptance criteria**

- Deployment of the "Cherry on top".

**Feature Matrix coverage**

- All previous.
