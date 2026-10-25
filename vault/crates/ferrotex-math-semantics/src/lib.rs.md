---
title: "crates/ferrotex-math-semantics/src/lib.rs"
type: source-code
language: rust
crate: ferrotex-math-semantics
layer: math-semantics
lines: 150
tags:
  - crate/ferrotex-math-semantics
  - lang/rust
  - layer/math-semantics
  - type/source-code
summary: "[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] — Source-code file (rust) in ferrotex-math-semantics subsystem (150 lines)."
---

# `crates/ferrotex-math-semantics/src/lib.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotex-math-semantics` (math-semantics)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `150`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotex-math-semantics** module within the `math-semantics` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `enum` | `Shape` |
| `enum` | `Dimension` |
| `fn` | `is_compatible_add` |
| `fn` | `is_compatible_mul` |

## Graph Links & Dependencies

- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[crates/ferrotex-math-semantics/Cargo.toml|Cargo.toml]] (`crates/ferrotex-math-semantics/Cargo.toml`)
- [[crates/ferrotex-math-semantics/README.md|README.md]] (`crates/ferrotex-math-semantics/README.md`)
- [[crates/ferrotex-math-semantics/src/analysis.rs|analysis.rs]] (`crates/ferrotex-math-semantics/src/analysis.rs`)
- [[crates/ferrotex-math-semantics/src/delimiters.rs|delimiters.rs]] (`crates/ferrotex-math-semantics/src/delimiters.rs`)
- [[docs/spec/math-verification.md|math-verification.md]] (`docs/spec/math-verification.md`)

