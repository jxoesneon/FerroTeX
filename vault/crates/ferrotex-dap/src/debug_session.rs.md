---
title: "crates/ferrotex-dap/src/debug_session.rs"
type: source-code
language: rust
crate: ferrotex-dap
layer: dap
lines: 1106
tags:
  - crate/ferrotex-dap
  - lang/rust
  - layer/dap
  - type/source-code
summary: "![allow(unexpected_cfgs)] — Source-code file (rust) in ferrotex-dap subsystem (1106 lines)."
---

# `crates/ferrotex-dap/src/debug_session.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotex-dap` (dap)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `1106`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotex-dap** module within the `dap` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `struct` | `MockDebugSession` |
| `struct` | `SteppingIoProvider` |
| `struct` | `TectonicDebugSession` |
| `enum` | `EngineEvent` |
| `enum` | `EngineCommand` |
| `trait` | `DebugDriver` |
| `fn` | `new` |
| `fn` | `new` |

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
- [[.github/workflows/coverage.yml|coverage.yml]] (`.github/workflows/coverage.yml`)
- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-dap/Cargo.toml|Cargo.toml]] (`crates/ferrotex-dap/Cargo.toml`)
- [[crates/ferrotex-dap/README.md|README.md]] (`crates/ferrotex-dap/README.md`)
- [[crates/ferrotex-dap/src/lib.rs|lib.rs]] (`crates/ferrotex-dap/src/lib.rs`)
- [[crates/ferrotex-dap/src/main.rs|main.rs]] (`crates/ferrotex-dap/src/main.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[crates/ferrotex-syntax/src/bibtex.rs|bibtex.rs]] (`crates/ferrotex-syntax/src/bibtex.rs`)
- [[crates/ferrotexd/src/build/tectonic.rs|tectonic.rs]] (`crates/ferrotexd/src/build/tectonic.rs`)
- [[crates/ferrotexd/src/macros.rs|macros.rs]] (`crates/ferrotexd/src/macros.rs`)
- [[docs/_sass/custom/custom.scss|custom.scss]] (`docs/_sass/custom/custom.scss`)
- [[docs/spec/dap.md|dap.md]] (`docs/spec/dap.md`)
- [[editors/vscode/snippets/latex.json|latex.json]] (`editors/vscode/snippets/latex.json`)
- [[editors/vscode/src/extension.ts|extension.ts]] (`editors/vscode/src/extension.ts`)

