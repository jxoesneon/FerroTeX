---
title: "crates/ferrotexd/src/build/mod.rs"
type: source-code
language: rust
crate: ferrotexd
layer: daemon
lines: 48
tags:
  - crate/ferrotexd
  - lang/rust
  - layer/daemon
  - type/source-code
summary: "![allow(dead_code)] — Source-code file (rust) in ferrotexd subsystem (48 lines)."
---

# `crates/ferrotexd/src/build/mod.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotexd` (daemon)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `48`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotexd** module within the `daemon` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `struct` | `BuildRequest` |
| `struct` | `BuildLog` |
| `enum` | `BuildStatus` |
| `trait` | `BuildEngine` |

## Graph Links & Dependencies

- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotexd/Cargo.toml|Cargo.toml]] (`crates/ferrotexd/Cargo.toml`)
- [[crates/ferrotexd/README.md|README.md]] (`crates/ferrotexd/README.md`)
- [[crates/ferrotexd/src/build/latexmk.rs|latexmk.rs]] (`crates/ferrotexd/src/build/latexmk.rs`)
- [[crates/ferrotexd/src/build/tectonic.rs|tectonic.rs]] (`crates/ferrotexd/src/build/tectonic.rs`)
- [[crates/ferrotexd/src/lib.rs|lib.rs]] (`crates/ferrotexd/src/lib.rs`)
- [[crates/ferrotexd/src/main.rs|main.rs]] (`crates/ferrotexd/src/main.rs`)
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/adrs/0005-universal-build-dag.md|0005-universal-build-dag.md]] (`docs/adrs/0005-universal-build-dag.md`)
- [[docs/architecture/overview.md|overview.md]] (`docs/architecture/overview.md`)
- [[docs/spec/build-orchestration.md|build-orchestration.md]] (`docs/spec/build-orchestration.md`)
- [[docs/spec/lsp.md|lsp.md]] (`docs/spec/lsp.md`)
- [[editors/vscode/snippets/latex.json|latex.json]] (`editors/vscode/snippets/latex.json`)

