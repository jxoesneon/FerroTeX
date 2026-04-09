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

## Pillars of Technical Excellence

To provide a highly refined and thoughtful experience for the research community, FerroTeX focuses on four key pillars of technical excellence:

1. **Incremental Everything (The "Speed" Aspiration)**: Moving from manual triggers to keystroke-level feedback. Aims for 50ms re-analysis for 500-page documents using persistent incremental state.
2. **"Time-Travel" Debugging (The "Observability" Aspiration)**: A mindful approach to macro expansion that supports stepping backward through the TeX stomach to understand exactly which token guided a state change.
3. **Formal Verification of Math (The "Trust" Aspiration)**: Moving from "it looks right" to "it is mathematically sound." Provides automated checks for matrix dimensions and variable consistency via symbolic execution.
4. **The "Safe-TeX" Sandbox (The "Security" Aspiration)**: A secure foundation for sensitive research environments using a global Virtual File System (VFS) and capabilities-based permissions.

## Current Status

**Version**: v0.24.0 — all features through v0.24.0 shipped.

| Pillar                                            | Status               |
| ------------------------------------------------- | -------------------- |
| Fault-tolerant CST (`ferrotex-syntax`, `rowan`)   | ✅ Shipped           |
| Multi-file workspace index + include graph        | ✅ Shipped           |
| Label/ref goto-definition, references, rename     | ✅ Shipped (v0.23.1) |
| Citation index + `.bib` parser                    | ✅ Shipped           |
| Completion, semantic tokens, folding, symbols     | ✅ Shipped           |
| Formatting + code actions                         | ✅ Shipped           |
| Engine adapters (latexmk, Tectonic)               | ✅ Shipped           |
| PDF viewer + bidirectional SyncTeX                | ✅ Shipped           |
| Rich hovers + human-readable error index          | ✅ Shipped           |
| Snippet pack + magic comments                     | ✅ Shipped           |
| Image paste wizard                                | ✅ Shipped           |
| DAP — step-in/over, register inspection           | ✅ Shipped           |
| `ferrotex.lock` hermetic builds                   | ✅ Shipped           |
| Semantic math analysis (matrix/delimiter)         | ✅ Shipped           |
| Multi-platform distribution (Linux/macOS/Windows) | ✅ Shipped           |
| Security hardening + CodeQL                       | ✅ Shipped           |
| VS Code test suite (53 passing, 0 failing)        | ✅ Shipped (v0.23.1) |
| Deprecated-command diagnostics + code actions     | ✅ Shipped (v0.24.0) |
| `\RequirePackage` package scanning                | ✅ Shipped (v0.24.0) |
| `\addbibresource` BibLaTeX indexing               | ✅ Shipped (v0.24.0) |

---

## Release Plan (Semantic Versioning)

Until `1.0.0`, breaking changes are allowed but MUST be documented in `CHANGELOG.md`.

Each release below includes **Scope**, **Acceptance criteria**, and **Priority** (`P1` = blocks users today, `P2` = significant quality improvement, `P3` = research-grade differentiator).

---

### v0.24.0 — "Diagnostic Completeness" ✅ Shipped

**Priority**: P1 — closes known gaps in the static analysis pipeline that already exist in the indexer but are not surfaced through LSP.

**Scope**

- **Wire deprecated-command diagnostics**: `Workspace::validate_deprecated()` exists and detects `{\bf ...}`, `$$...$$`, and obsolete packages (`times`, `a4wide`, etc.) but is never called from `validate_document()`. Surface these through the standard `textDocument/publishDiagnostics` flow.
- **Code actions for deprecated commands**: pair each deprecated diagnostic with a quick-fix (`\bf` → `\textbf`, `$$` → `\[...\]`).
- **`\RequirePackage` in citation index**: the package scanner regex currently only matches `\usepackage`, not `\RequirePackage`. Extend to cover both (affects completion and package-awareness features in class/style files).
- **BibLaTeX entry-type awareness**: extend the citation index to understand `\addbibresource` (already partially handled) and BibLaTeX-specific entry types (`@online`, `@software`, `@dataset`). This is IDE-layer only — the compile-time bibliography backend remains the runner's concern.

**Acceptance criteria**

- `{\bf bold}` in an open document produces a `warning` diagnostic with a quick-fix.
- `$$x = y$$` produces a `hint` diagnostic with a quick-fix to `\[x = y\]`.
- `\usepackage{times}` produces a `warning` suggesting `\usepackage{mathptmx}`.
- `\RequirePackage{amsmath}` is treated identically to `\usepackage{amsmath}` for completion purposes.
- `@online` entries in a `.bib` file do not produce false-positive "unknown entry type" diagnostics.

**Feature Matrix coverage**

- SD-1 (expanded), SD-5 (deprecated patterns)
- LSP-15 (code actions)
- IDX-3 (BibLaTeX extension)

---

### v0.25.0 — "Incremental Analysis" (Speed Pillar)

**Priority**: P2 — currently re-analysis on `didChange` re-parses the entire document. Acceptable for small files; becomes perceptible on theses (100k+ tokens).

**Scope**

- **Reactive dependency graph**: refactor `ferrotex-analysis` to use a `salsa`-style demand-driven computation model. Analysis queries invalidate only the minimum affected nodes.
- **Bounded incremental reparse**: limit `didChange` reparse to the edited region plus its containing environment/group boundary. Preserve unchanged subtrees from the previous CST.
- **Keystroke-level diagnostics**: eliminate the need for a manual build trigger for static diagnostics. Parse + analyze on every `didChange` with debounce ≤ 150ms.

**Acceptance criteria**

- Re-analysis of a 100,000-token document on a single-line `didChange` completes in <50ms (p95).
- Benchmark result added to `docs/development/benchmarks.md`.
- No diagnostic flapping on rapid consecutive edits.

**Feature Matrix coverage**

- LP-2 (incremental reparse)
- PF-1, PF-2

---

### v0.26.0 — "Time-Travel Debugging" (Observability Pillar)

**Priority**: P3 — extends the existing DAP implementation with reversible execution, the primary research-grade differentiator.

**Scope**

- **Reversible DAP**: snapshot-based backward stepping in `ferrotex-dap`. Each macro expansion step that modifies register state captures a delta; `stepBack` restores the previous snapshot.
- **Ghost expansion hover**: hovering a macro name shows its fully expanded token stream in a hover tooltip, without executing the engine.
- **Breakpoint persistence**: debug breakpoints survive incremental document edits (currently cleared on any edit).
- **Watch expressions**: monitor arbitrary control sequences across expansion steps.

**Acceptance criteria**

- VS Code "Step Back" button is enabled and functional in the TeX debug session.
- Hovering `\mycmd` (defined via `\newcommand`) shows the expanded body.
- A breakpoint on line 10 survives inserting a new line above it.

**Feature Matrix coverage**

- DAP-3 (reversible execution)
- DAP-4 (ghost expansion)

---

### v0.27.0 — "Formal Math Verification" (Trust Pillar)

**Priority**: P3 — extends the existing semantic math analysis with symbolic execution for structural correctness.

**Scope**

- **Symbolic dimension checking**: track matrix column counts across `&`-separated cells and `\\`-terminated rows. Flag mismatches as `error` diagnostics before compilation.
- **Variable consistency**: detect when a math variable is used with inconsistent dimensions across equations in the same document (e.g., `A` used as both scalar and matrix).
- **`ferrotex ci verify` CLI command**: wrap `ferrotex-build` in a CI-oriented subcommand that exits non-zero on any semantic math error, enabling pre-commit hooks.
- **Experimental Lean/Coq bridge** _(stretch)_: export the symbolic math model as a Lean 4 proof obligation for external verification. Gated behind a feature flag.

**Acceptance criteria**

- A matrix with 3 columns in row 1 and 2 columns in row 2 produces an `error` diagnostic at the mismatched row.
- `ferrotex ci verify path/to/doc.tex` exits 1 on the above document.
- No regressions in existing math analysis tests.

**Feature Matrix coverage**

- LP-5 (expanded)
- BO-10 (CI integration)

---

### v0.28.0 — "Safe-TeX Sandbox" (Security Pillar)

**Priority**: P2 for academic/institutional users; P3 otherwise.

**Scope**

- **VFS enforcement**: wrap all file I/O in `ferrotex-build` behind a virtual file system abstraction. Every read/write goes through the VFS; paths outside the project root require an explicit capability grant.
- **Capabilities system**: surface `shell-escape` and network-fetch requests as VS Code prompts. User grants are recorded in `ferrotex.lock` for reproducibility.
- **Zero-trust defaults**: `shell-escape` is disabled by default. Documents that require it must declare it explicitly.
- **Audit log**: all capability grants and file accesses outside the project root are logged to `ferrotex-build`'s structured event stream.

**Acceptance criteria**

- A document attempting `\write18{rm -rf /}` is blocked at the VFS layer and produces an `error` diagnostic.
- The user sees a VS Code prompt before any `shell-escape` executes.
- Capability grants appear in `ferrotex.lock`.

**Feature Matrix coverage**

- SEC-1, SEC-2, SEC-3
- BO-11 (audit trail)

---

### v1.0.0 — "Gold" Release

**Priority**: Ceremonial — represents the point at which all four technical excellence pillars have shipped their primary scope.

**Scope**

- Final stability audit: zero P1 bugs open for 2 weeks.
- `CHANGELOG.md` and `docs/` fully reflect shipped behavior (no aspirational prose).
- SemVer public API guarantee: breaking changes to LSP contract, log IR schema, and CLI flags require a major version bump from this point forward.
- README "Enjoy!" update.

**Acceptance criteria**

- All Feature Matrix IDs in `docs/spec/feature-matrix.md` are marked `shipped` or `deferred`.
- `cargo test --workspace` and `npm test` both pass on Linux, macOS, and Windows in CI.

**Feature Matrix coverage**

- All previous.
