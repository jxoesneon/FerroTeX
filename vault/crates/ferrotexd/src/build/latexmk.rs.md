---
title: "crates/ferrotexd/src/build/latexmk.rs"
type: source-code
language: rust
crate: ferrotexd
layer: daemon
lines: 391
tags:
  - crate/ferrotexd
  - lang/rust
  - layer/daemon
  - type/source-code
summary: "[derive(Debug)] — Source-code file (rust) in ferrotexd subsystem (391 lines)."
---

# `crates/ferrotexd/src/build/latexmk.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotexd` (daemon)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `391`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotexd** module within the `daemon` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `struct` | `LatexmkAdapter` |
| `fn` | `new` |
| `fn` | `with_binary` |
| `fn` | `create_command` |

## Graph Links & Dependencies

- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[crates/ferrotexd/Cargo.toml|Cargo.toml]] (`crates/ferrotexd/Cargo.toml`)
- [[crates/ferrotexd/README.md|README.md]] (`crates/ferrotexd/README.md`)
- [[crates/ferrotexd/src/lib.rs|lib.rs]] (`crates/ferrotexd/src/lib.rs`)
- [[crates/ferrotexd/src/main.rs|main.rs]] (`crates/ferrotexd/src/main.rs`)
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/adrs/0005-universal-build-dag.md|0005-universal-build-dag.md]] (`docs/adrs/0005-universal-build-dag.md`)
- [[docs/architecture/overview.md|overview.md]] (`docs/architecture/overview.md`)
- [[docs/spec/build-orchestration.md|build-orchestration.md]] (`docs/spec/build-orchestration.md`)
- [[docs/spec/lsp.md|lsp.md]] (`docs/spec/lsp.md`)
- [[editors/vscode/snippets/latex.json|latex.json]] (`editors/vscode/snippets/latex.json`)
- [[editors/vscode/src/extension.ts|extension.ts]] (`editors/vscode/src/extension.ts`)

