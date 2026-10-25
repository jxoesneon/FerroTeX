---
title: "crates/ferrotexd/src/diagnostics/error_index.rs"
type: source-code
language: rust
crate: ferrotexd
layer: daemon
lines: 260
tags:
  - crate/ferrotexd
  - lang/rust
  - layer/daemon
  - type/source-code
summary: "[cfg(test)] — Source-code file (rust) in ferrotexd subsystem (260 lines)."
---

# `crates/ferrotexd/src/diagnostics/error_index.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotexd` (daemon)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `260`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotexd** module within the `daemon` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `struct` | `ErrorExplanation` |
| `struct` | `CommandInfo` |
| `fn` | `explain` |
| `fn` | `extract_undefined_command` |
| `fn` | `suggest_package` |

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
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
- [[docs/_sass/custom/custom.scss|custom.scss]] (`docs/_sass/custom/custom.scss`)
- [[docs/architecture/overview.md|overview.md]] (`docs/architecture/overview.md`)
- [[docs/development/setup.md|setup.md]] (`docs/development/setup.md`)
- [[docs/spec/configuration.md|configuration.md]] (`docs/spec/configuration.md`)
- [[docs/spec/formatting.md|formatting.md]] (`docs/spec/formatting.md`)
- [[docs/spec/lsp.md|lsp.md]] (`docs/spec/lsp.md`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[editors/vscode/snippets/latex.json|latex.json]] (`editors/vscode/snippets/latex.json`)
- [[package.json|package.json]] (`package.json`)

