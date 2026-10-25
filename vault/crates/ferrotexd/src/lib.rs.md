---
title: "crates/ferrotexd/src/lib.rs"
type: source-code
language: rust
crate: ferrotexd
layer: daemon
tags:
  - crate/ferrotexd
  - type/rust
  - layer/daemon
  - lsp/server
summary: "Core library implementation of the FerroTeX Language Server Protocol (LSP) daemon (ferrotexd). Defines the shared stateful Backend struct and implements tower_lsp::LanguageServer for document synchronization, diagnostics, completion, hover, formatting, code actions, semantic tokens, workspace commands, and SyncTeX forward/inverse search."
---

# `crates/ferrotexd/src/lib.rs`

## Purpose & Architecture Role

`crates/ferrotexd/src/lib.rs` provides the central Language Server Protocol (LSP) engine for FerroTeX. It encapsulates the daemon's core stateful server backend (`Backend`) and implements the asynchronous `tower_lsp::LanguageServer` trait. 

As the orchestration hub of the daemon layer, this module coordinates lower-level FerroTeX subsystems:
- **Concrete Syntax Trees (CST)**: Ingests AST/CST trees parsed by [[crates/ferrotex-syntax/src/parser.rs|ferrotex-syntax]] to compute semantic tokens, inspect nodes under cursor, and calculate line/column mappings.
- **Cross-File Symbol Indexing**: Leverages [[crates/ferrotexd/src/workspace.rs|workspace.rs]] to maintain project-wide include graphs, cross-document label declarations, citations, and multi-file definitions.
- **Diagnostics Pipeline**: Runs multi-pass validation across syntactic errors, math semantics via [[crates/ferrotexd/src/diagnostics/mod.rs|diagnostics module]], label collisions/dangling references, deprecated LaTeX 2.09 constructs, and build log warnings via `ferrotex-log`.
- **Intelligent Features**: Dispatches requests to specialized subsystems including [[crates/ferrotexd/src/completer.rs|completer.rs]], [[crates/ferrotexd/src/hover.rs|hover.rs]], and [[crates/ferrotexd/src/fmt.rs|fmt.rs]].
- **Build & SyncTeX Orchestration**: Executes compiler workflows via [[crates/ferrotexd/src/build/mod.rs|build module]] and handles bidirectional PDF/source synchronization through [[crates/ferrotexd/src/synctex.rs|synctex.rs]].

---

## Submodules

| Module | Relative Path | Architectural Role |
|---|---|---|
| `build` | [[crates/ferrotexd/src/build/mod.rs\|build module]] | Build engine abstraction (`BuildEngine`, `BuildRequest`, `LatexmkAdapter`) for headless LaTeX document compilation. |
| `completer` | [[crates/ferrotexd/src/completer.rs\|completer.rs]] | Auto-completion provider for commands, environments, packages, labels, citations, and file paths. |
| `diagnostics` | [[crates/ferrotexd/src/diagnostics/mod.rs\|diagnostics module]] | Specialized diagnostic linters, including mathematical matrix/delimiter validation and deprecated syntax checks. |
| `fmt` | [[crates/ferrotexd/src/fmt.rs\|fmt.rs]] | LaTeX document formatting and indentation engine based on syntax tree traversal. |
| `hover` | [[crates/ferrotexd/src/hover.rs\|hover.rs]] | Tooltip documentation and context provider for commands, packages, environments, and bibliography entries. |
| `macros` | [[crates/ferrotexd/src/macros.rs\|macros.rs]] | Shared declarative macro definitions and helper routines across the daemon crate. |
| `synctex` | [[crates/ferrotexd/src/synctex.rs\|synctex.rs]] | Bidirectional forward and inverse search synchronization between TeX source code and generated PDF coordinates. |
| `workspace` | [[crates/ferrotexd/src/workspace.rs\|workspace.rs]] | Multi-file workspace index manager tracking document dependency graphs, label definitions, citations, and build roots. |

---

## Key Symbols & Interfaces

### 1. The `Backend` Server Struct

The `Backend` struct maintains the global, concurrent state of the language server instance across asynchronous requests:

```rust
#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    pub documents: Arc<DashMap<Url, String>>,
    pub workspace: Arc<Workspace>,
    pub root_uri: Arc<Mutex<Option<Url>>>,
    pub syntax_diagnostics: Arc<DashMap<Url, Vec<Diagnostic>>>,
    pub package_manager: Arc<Mutex<package_manager::PackageManager>>,
    pub package_index: Arc<Mutex<Option<PackageIndex>>>,
    pub build_engine: Arc<dyn BuildEngine>,
}
```

- **`client: Client`**: Handle for sending asynchronous JSON-RPC requests, notifications (`publish_diagnostics`, `show_message`, `log_message`, `Progress`), and window telemetry back to the IDE client.
- **`documents: Arc<DashMap<Url, String>>`**: Thread-safe in-memory virtual filesystem holding the active contents of all open buffers.
- **`workspace: Arc<Workspace>`**: Cross-file index store holding label registries, bibliography sources, include trees, and root target heuristics.
- **`root_uri: Arc<Mutex<Option<Url>>>`**: Root directory URI configured during workspace initialization.
- **`syntax_diagnostics: Arc<DashMap<Url, Vec<Diagnostic>>>`**: In-memory cache of syntax and lint diagnostics per document.
- **`package_manager: Arc<Mutex<PackageManager>>`**: TeX package manager integration (e.g. `tlmgr`, `miktex`) for package queries and background installs.
- **`package_index: Arc<Mutex<Option<PackageIndex>>>`**: In-memory package database loaded from disk cache or discovered via `PackageScanner`.
- **`build_engine: Arc<dyn BuildEngine>`**: Dynamic compilation backend (typically `LatexmkAdapter`).

---

### 2. LSP Trait Implementation (`tower_lsp::LanguageServer`)

`Backend` implements all core LSP endpoints required by the [[docs/spec/lsp.md|LSP Specification]]:

| Handler | Protocol Method | Description & Implementation Details |
|---|---|---|
| `initialize` | `initialize` | Registers server capabilities (Full text document sync, hover, completion triggers `\`, `{`, `(`, symbols, formatting, rename, code actions, semantic tokens). Launches background worker to load/cache `PackageIndex` and reports progress via LSP `Progress`. |
| `initialized` | `initialized` | Spawns a filesystem watcher (`notify::RecommendedWatcher`) on the workspace root to monitor `.log` file changes and publish TeX build diagnostics. |
| `shutdown` | `shutdown` | Gracefully closes background processes and returns `Ok(())`. |
| `did_open` | `textDocument/didOpen` | Stores document text in `documents` map and triggers asynchronous multi-pass validation via `validate_document`. |
| `did_change` | `textDocument/didChange` | Updates in-memory document buffer and triggers re-validation via `validate_document`. |
| `execute_command` | `workspace/executeCommand` | Handles custom workspace commands: `ferrotex.internal.build`, `ferrotex.internal.installPackage`, `ferrotex.synctex_forward`, and `ferrotex.synctex_inverse`. |
| `document_symbol` | `textDocument/documentSymbol` | Extracts document outline (sections, environments, labels, macros) from `workspace.query_symbols`. |
| `goto_definition` | `textDocument/definition` | Identifies label/macro at cursor via `label_at_position` and resolves targets using `workspace.find_definitions`. |
| `references` | `textDocument/references` | Finds all usages of a label across the entire project via `workspace.find_references`, optionally appending definitions if `include_declaration` is set. |
| `prepare_rename` | `textDocument/prepareRename` | Verifies whether the symbol at cursor position is a renameable label token via `find_label_token_at` and returns its exact source range. |
| `rename` | `textDocument/rename` | Computes a project-wide `WorkspaceEdit` replacing all definition and reference occurrences of the target label. |
| `hover` | `textDocument/hover` | Resolves CST token at cursor and delegates to [[crates/ferrotexd/src/hover.rs\|hover.rs]] (`find_hover`). |
| `completion` | `textDocument/completion` | Queries imported packages from `workspace`, checks `package_index`, and retrieves command/environment items via [[crates/ferrotexd/src/completer.rs\|completer.rs]]. |
| `formatting` | `textDocument/formatting` | Parses document into a CST and applies syntax-aware formatting via [[crates/ferrotexd/src/fmt.rs\|fmt.rs]] (`format_document`). |
| `code_action` | `textDocument/codeAction` | Inspects diagnostics in range and constructs quick-fix edits (`deprecated_quick_fix`) for obsolete LaTeX commands. |
| `semantic_tokens_full` | `textDocument/semanticTokens/full` | Walks syntax tree and computes delta-encoded tokens matching `SEMANTIC_TOKEN_TYPES` and `SEMANTIC_TOKEN_MODIFIERS`. |

---

### 3. Core Internal Routines

- **`validate_document(&self, uri: Url)`**:
  1. Updates the `Workspace` index with new document content.
  2. Runs CST syntax parser (`ferrotex_syntax::parse`) and converts syntax errors to LSP `Diagnostic`s.
  3. Executes mathematical formula checks via `diagnostics::math::check_math`.
  4. Validates cross-file labels via `self.workspace.validate_labels()` (detecting undefined labels, collisions).
  5. Scans for deprecated commands and packages via `self.workspace.validate_deprecated()`.
  6. Publishes collected diagnostics via `client.publish_diagnostics`.
  7. Parses build `.log` file if present using `ferrotex_log::LogParser` to append compiler warnings.
- **`run_build(&self, uri: Url)`**:
  - Resolves main build root via `self.workspace.resolve_build_target(&uri)` (redirecting subfiles to parent root documents).
  - Submits asynchronous `BuildRequest` to the configured `build_engine`.
- **`compute_semantic_tokens(&self, text: &str) -> Vec<SemanticToken>`**:
  - Encodes syntax elements into relative token tuples (`delta_line`, `delta_start`, `length`, `token_type`, `token_modifiers_bitset`).
- **`find_label_token_at(root, offset)`**:
  - Extracts the exact string and `TextRange` for `\label{...}`, `\ref{...}`, and multi-key citations `\cite{a,b}` under cursor.
- **`deprecated_code`, `deprecated_message`, `deprecated_quick_fix`**:
  - Maps deprecated font commands (e.g. `\bf` -> `\textbf`, `\it` -> `\textit`, `$$` -> `\[...\]`) and obsolete packages (`times`, `a4wide`, `epsfig`) to actionable LSP quick-fixes.

---

### 4. Supporting Enums & Constants

- **`CompletionKind`**:
  ```rust
  pub enum CompletionKind {
      None,
      Citation,
      Environment,
      Command,
      File,
  }
  ```
- **`COMMANDS` & `ENVIRONMENTS`**: Built-in static slices of core LaTeX commands and environments.
- **`SEMANTIC_TOKEN_TYPES`**: Macro (`\cmd`), Keyword (`\begin`, `\end`), String (arguments), Comment (`% ...`), Parameter (optional args), Variable (labels/citations).
- **`SEMANTIC_TOKEN_MODIFIERS`**: Declaration, Definition, Readonly.

---

## Dependencies & Cross-Module Links

### Internal FerroTex Crates
- **[[crates/ferrotex-syntax/src/parser.rs|ferrotex-syntax]]**: Provides CST node parsing (`parse`), token kinds (`SyntaxKind`), and `TextRange` types.
- **[[crates/ferrotex-package/src/lib.rs|ferrotex-package]]**: Supplies `PackageIndex` and `PackageScanner` for LaTeX distribution cataloging.
- **[[crates/ferrotex-core/src/lib.rs|ferrotex-core]]**: Integrates `package_manager::PackageManager` for tlmgr/miktex CLI operations.
- **[[crates/ferrotex-log/src/lib.rs|ferrotex-log]]**: Parses compiler logs (`LogParser`, `ir::EventPayload`) to surface LaTeX engine errors and warnings.

### External Libraries
- **`tower-lsp`**: High-performance asynchronous JSON-RPC protocol abstractions (`LanguageServer`, `Client`, `Server`).
- **`dashmap`**: Thread-safe concurrent map (`DashMap`) powering real-time lock-free document buffer lookups.
- **`notify`**: Filesystem event watcher for real-time `.log` file updates.
- **`line-index`**: Efficient translation between UTF-8 byte offsets and line/character coordinates (`LineIndex`, `LineCol`).
- **`tokio`**: Asynchronous runtime driving concurrent tasks, background compilation, and thread-blocking workers.

---

## Consumers & Dependents

- **[[crates/ferrotexd/src/main.rs|main.rs]]**: Instantiates `Backend` with production `LatexmkAdapter` and starts the `LspService` stdio server.
- **IDE Extensions**:
  - **[[editors/vscode/src/extension.ts|VS Code Extension]]**: Communicates with `Backend` via JSON-RPC over stdio, invoking formatting, hover, completion, commands (`ferrotex.internal.build`, `ferrotex.synctex_forward`), and receiving diagnostics.
  - **Helix, Neovim, Emacs, Sublime Text**: Standard LSP clients interfacing with `ferrotexd` through standard input/output.

---

## Related Specifications & Architectural Documentation

- **[[docs/spec/lsp.md|LSP Specification]]**: Full protocol specification for FerroTex LSP capabilities and custom JSON-RPC methods.
- **[[docs/architecture/overview.md|Architecture Overview]]**: Architectural role of `ferrotexd` within the FerroTeX dual-pipeline system.
- **[[docs/spec/build-orchestration.md|Build Orchestration Spec]]**: Details compilation lifecycles, target discovery, and engine adapters.
- **[[docs/spec/vfs.md|VFS & Document Model Spec]]**: Rules governing document synchronization, caching, and line indexing.
