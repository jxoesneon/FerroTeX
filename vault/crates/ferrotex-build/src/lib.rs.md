---
title: "crates/ferrotex-build/src/lib.rs"
type: source-code
language: rust
crate: ferrotex-build
layer: build
lines: 325
tags:
  - crate/ferrotex-build
  - lang/rust
  - layer/build
  - type/source-code
summary: "[derive(Debug, Clone, Serialize, Deserialize, Default)] — Source-code file (rust) in ferrotex-build subsystem (325 lines)."
---

# `crates/ferrotex-build/src/lib.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotex-build` (build)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `325`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotex-build** module within the `build` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `struct` | `Lockfile` |
| `struct` | `ArtifactId` |
| `struct` | `BuildGraph` |
| `trait` | `Artifact` |
| `trait` | `Transform` |
| `fn` | `new` |
| `fn` | `save` |
| `fn` | `load` |
| `fn` | `new` |
| `fn` | `add_artifact` |
| `fn` | `add_transform` |
| `fn` | `validate` |

## Graph Links & Dependencies

- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/Cargo.toml|Cargo.toml]] (`crates/ferrotex-build/Cargo.toml`)
- [[crates/ferrotex-build/README.md|README.md]] (`crates/ferrotex-build/README.md`)
- [[crates/ferrotex-build/src/artifacts.rs|artifacts.rs]] (`crates/ferrotex-build/src/artifacts.rs`)
- [[crates/ferrotex-build/src/compiler.rs|compiler.rs]] (`crates/ferrotex-build/src/compiler.rs`)
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/adrs/0005-universal-build-dag.md|0005-universal-build-dag.md]] (`docs/adrs/0005-universal-build-dag.md`)
- [[docs/spec/build-orchestration.md|build-orchestration.md]] (`docs/spec/build-orchestration.md`)
- [[docs/spec/export.md|export.md]] (`docs/spec/export.md`)
- [[editors/vscode/snippets/latex.json|latex.json]] (`editors/vscode/snippets/latex.json`)

