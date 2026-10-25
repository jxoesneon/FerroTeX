---
title: "crates/ferrotex-analysis/src/lib.rs"
type: source-code
language: rust
crate: ferrotex-analysis
layer: analysis
lines: 225
tags:
  - crate/ferrotex-analysis
  - lang/rust
  - layer/analysis
  - type/source-code
summary: "[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] — Source-code file (rust) in ferrotex-analysis subsystem (225 lines)."
---

# `crates/ferrotex-analysis/src/lib.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotex-analysis` (analysis)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `225`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotex-analysis** module within the `analysis` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `struct` | `AbstractState` |
| `struct` | `AbstractMachine` |
| `enum` | `AbstractValue` |
| `fn` | `new` |
| `fn` | `step` |

## Graph Links & Dependencies

- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-analysis/Cargo.toml|Cargo.toml]] (`crates/ferrotex-analysis/Cargo.toml`)
- [[crates/ferrotex-analysis/README.md|README.md]] (`crates/ferrotex-analysis/README.md`)
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[crates/ferrotex-math-semantics/src/analysis.rs|analysis.rs]] (`crates/ferrotex-math-semantics/src/analysis.rs`)

