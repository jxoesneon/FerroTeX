---
title: "crates/ferrotexd/src/completer.rs"
type: source-code
language: rust
crate: ferrotexd
layer: daemon
tags:
  - crate/ferrotexd
  - type/rust
  - layer/daemon
  - lsp/completion
summary: "LaTeX completion engine for ferrotexd. Provides static package completion data, dynamic completion from the parsed document AST and workspace indices, package inventory scanning, and user macro extraction."
---

# `crates/ferrotexd/src/completer.rs`

## Purpose & Architecture Role

`crates/ferrotexd/src/completer.rs` implements the completion provider and item generation engine for **`ferrotexd`**, the FerroTeX Language Server Protocol (LSP) daemon. Invoked during LSP `textDocument/completion` requests by [[crates/ferrotexd/src/lib.rs|ferrotexd lib.rs]], it collects, categorizes, deduplicates, and formats code completions and snippet items for client editors.

### Architectural Responsibilities:
1. **Multi-Source Completion Aggregation**: Combines static built-in package definitions, dynamically indexed CTAN packages, and workspace-discovered user macros.
2. **Context-Sensitive Classification**: Categorizes suggestions into command calls (`CompletionItemKind::FUNCTION`) and structural environments/snippets (`CompletionItemKind::SNIPPET`).
3. **Workspace AST Integration**: Queries [[crates/ferrotexd/src/workspace.rs|workspace.rs]] to ingest dynamic macros (`\newcommand`, `\def`), label definitions, and cross-references.
4. **Citation & Bibliography Linkage**: Coordinates with [[crates/ferrotex-syntax/src/bibtex.rs|bibtex]] via workspace indexes for BibTeX citation key auto-completion.
5. **Deduplication & Hygiene**: Enforces set-based filtering (`seen_cmds`, `seen_envs`) ensuring repeated package imports or shadowed macros produce deterministic completions.

```text
       LSP `textDocument/completion` Request
                         │
                         ▼
          [[crates/ferrotexd/src/lib.rs|Backend::completion]]
                         │
                         ▼
    ┌─────────────────────────────────────────────────────────┐
    │     [[crates/ferrotexd/src/completer.rs|completer.rs]]  │
    │             `get_package_completions(...)`              │
    └──────┬─────────────────────┬────────────────────┬───────┘
           │                     │                    │
           ▼                     ▼                    ▼
   [Tier 1: Static]      [Tier 2: Dynamic]    [Tier 3: Workspace]
     `PACKAGE_DATA`        `PackageIndex`         `Workspace`
   (amsmath, tikz, ...)  (`ferrotex-package`) (AST Macros & Symbols)
           │                     │                    │
           └─────────────────────┼────────────────────┘
                                 │
                                 ▼
                     Deduplication & Formatting
                       (`seen_cmds`, `seen_envs`)
                                 │
                                 ▼
             Tuple: (Vec<CompletionItem>, Vec<CompletionItem>)
                   (Commands/Macros & Environments)
```

---

## Code Breakdown & Key Symbols

### Types & Data Structures

#### `PackageCompletion`
```rust
#[derive(Debug, Clone)]
pub struct PackageCompletion {
    pub commands: Vec<String>,
    pub environments: Vec<String>,
}
```
Represents pre-computed completion items associated with a LaTeX package:
- `commands`: Vector of command identifiers without the leading backslash (e.g., `"eqref"`, `"includegraphics"`).
- `environments`: Vector of environment identifiers (e.g., `"align"`, `"tikzpicture"`).

#### `PACKAGE_DATA: Lazy<HashMap<&'static str, PackageCompletion>>`
Static, lazily-initialized registry housing built-in completion dictionaries for standard packages:
- **`amsmath`**:
  - *Commands*: `\text`, `\tag`, `\eqref`, `\numberwithin`, `\dddot`, `\ddddot`, `\boldsymbol`
  - *Environments*: `align`, `align*`, `gather`, `gather*`, `flalign`, `flalign*`, `alignat`, `alignat*`, `split`, `cases`, `matrix`, `pmatrix`, `bmatrix`
- **`tikz`**:
  - *Commands*: `\draw`, `\node`, `\coordinate`, `\fill`, `\clip`, `\path`, `\usetikzlibrary`
  - *Environments*: `tikzpicture`, `scope`
- **`geometry`**:
  - *Commands*: `\geometry`, `\newgeometry`, `\restoregeometry`
- **`hyperref`**:
  - *Commands*: `\href`, `\url`, `\hypersetup`, `\autorek`
- **`graphicx`**:
  - *Commands*: `\includegraphics`, `\graphicspath`, `\rotatebox`, `\scalebox`

---

### Core Routines

#### `get_package_completions`
```rust
pub fn get_package_completions(
    packages: &[String],
    index: Option<&PackageIndex>,
    workspace: Option<&crate::workspace::Workspace>,
) -> (Vec<CompletionItem>, Vec<CompletionItem>)
```
The primary entry point that assembles completion candidates for a given document:
1. **Static Lookup**: Iterates over active `packages`, querying `PACKAGE_DATA`.
2. **Dynamic Index**: Falls back to the scanned disk index (`PackageIndex` from `ferrotex-package`) if not found in static tables.
3. **User Workspace Macros**: Inspects `workspace.indices` across tracked files to extract user-defined macros scanned from the parsed document AST.
4. **Item Construction**: Transforms identifiers into LSP `CompletionItem` records, automatically prefixing backslashes for commands and assigning appropriate `CompletionItemKind` and metadata labels.

#### `add_items`
```rust
fn add_items(
    cmd_items: &mut Vec<CompletionItem>,
    env_items: &mut Vec<CompletionItem>,
    seen_cmds: &mut std::collections::HashSet<String>,
    seen_envs: &mut std::collections::HashSet<String>,
    pkg: &str,
    commands: &[String],
    environments: &[String],
)
```
Internal worker function ensuring deduplication and metadata population:
- Appends `\` to commands and tags them as `CompletionItemKind::FUNCTION` with detail `Package: <pkg>`.
- Tags environments as `CompletionItemKind::SNIPPET` with detail `Package: <pkg>`.

---

## Completion Context Classification & Roadmap

In compliance with the [[docs/spec/completion.md|Completion Spec]], `ferrotexd` defines completion behaviors across distinct syntactic contexts:

| Syntactic Context | Cursor Position / Trigger | Primary Source | LSP Kind |
|---|---|---|---|
| **Command** | Following `\` | `PACKAGE_DATA`, `PackageIndex`, Workspace AST Macros | `FUNCTION` |
| **Environment** | Inside `\begin{...}` or `\end{...}` | `PACKAGE_DATA`, `PackageIndex` environments | `SNIPPET` |
| **Citation** | Inside `\cite{...}`, `\citep{...}`, `\citet{...}` | `Workspace.bib_indices` via [[crates/ferrotex-syntax/src/bibtex.rs|bibtex]] | `REFERENCE` |
| **Label / Cross-Ref** | Inside `\ref{...}`, `\eqref{...}`, `\pageref{...}` | `Workspace.indices.definitions` (labels) | `REFERENCE` |
| **Preamble / Package** | Inside `\usepackage{...}` or `\documentclass{...}` | Available package inventory | `MODULE` |
| **File Path** | Inside `\input{...}`, `\include{...}`, `\includegraphics{...}` | Workspace file system traversal | `FILE` |
| **Math Mode** | Inside `$...$`, `$$...$$`, `\(...\)`, `\[...\]` | Math commands (`amsmath` symbols, Greek letters) | `OPERATOR` / `FUNCTION` |

---

## Dependencies & Cross-Module Links

### Internal FerroTex Components
- **[[crates/ferrotexd/src/lib.rs|ferrotexd lib.rs]]**: Consumes `get_package_completions` inside `Backend::completion` to handle client language server requests.
- **[[crates/ferrotexd/src/workspace.rs|workspace.rs]]**: Supplies AST symbol indexes (`FileIndex.macros`, `FileIndex.definitions`, `FileIndex.citations`) and parsed bibliography stores (`Workspace.bib_indices`).
- **[[crates/ferrotex-syntax/src/parser.rs|syntax parser]]**: Parses LaTeX document CSTs from which macros, labels, and package inclusions are extracted.
- **[[crates/ferrotex-syntax/src/bibtex.rs|bibtex]]**: Parses `.bib` bibliography databases to supply citation keys for `\cite` completion.

### External & Library Dependencies
- `ferrotex_package::PackageIndex`: Scanned metadata representation of TeX packages installed on the host system or distribution.
- `tower_lsp::lsp_types`: Standard LSP protocol models (`CompletionItem`, `CompletionItemKind`).
- `lazy_static`: Safe global static initialization for `PACKAGE_DATA`.
- `std::collections::{HashMap, HashSet}`: In-memory dictionary lookups and deduplication tracking.

---

## Consumers & Dependents

- **[[crates/ferrotexd/src/lib.rs|crates/ferrotexd/src/lib.rs]]**: Calls `completer::get_package_completions` inside the async `completion()` handler:
  ```rust
  let (cmds, envs) = completer::get_package_completions(
      &packages,
      index_guard.as_ref(),
      Some(&self.workspace),
  );
  ```
- **LSP Clients**: Visual Studio Code, Neovim, Emacs, and Helix via JSON-RPC `textDocument/completion`.

---

## Related Specifications & ADRs

- **[[docs/spec/completion.md|Completion Spec]]**: Normative specification defining completion categories, context detection rules, ranking heuristics, and latency limits.
- **[[docs/spec/completion-sources.md|Completion Sources Spec]]**: Normative specification defining data tiers:
  - *S1: Built-in Inventory* (fulfilled by `PACKAGE_DATA`).
  - *S2: Workspace Discovery* (fulfilled by `workspace.rs` macro scanning).
  - *S3: Package/CTAN Metadata* (fulfilled by `ferrotex-package::PackageIndex`).
- **[[docs/spec/lsp.md|LSP Protocol Spec]]**: Protocol conventions and capabilities for completion triggers and response formatting.
