---
title: "crates/ferrotex-syntax/src/bibtex.rs"
type: source-code
language: rust
crate: ferrotex-syntax
layer: syntax
lines: 451
tags:
  - crate/ferrotex-syntax
  - lang/rust
  - layer/syntax
  - type/source-code
summary: "[derive(Debug, Clone, PartialEq, Eq)] — Source-code file (rust) in ferrotex-syntax subsystem (451 lines)."
---

# `crates/ferrotex-syntax/src/bibtex.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotex-syntax` (syntax)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `451`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotex-syntax** module within the `syntax` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `struct` | `BibEntry` |
| `struct` | `BibFile` |
| `fn` | `parse_bibtex` |

## Graph Links & Dependencies

- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-log/src/parser.rs|parser.rs]] (`crates/ferrotex-log/src/parser.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[crates/ferrotex-math-semantics/src/delimiters.rs|delimiters.rs]] (`crates/ferrotex-math-semantics/src/delimiters.rs`)
- [[crates/ferrotex-syntax/Cargo.toml|Cargo.toml]] (`crates/ferrotex-syntax/Cargo.toml`)
- [[crates/ferrotex-syntax/README.md|README.md]] (`crates/ferrotex-syntax/README.md`)
- [[crates/ferrotex-syntax/src/lib.rs|lib.rs]] (`crates/ferrotex-syntax/src/lib.rs`)
- [[crates/ferrotex-syntax/src/parser.rs|parser.rs]] (`crates/ferrotex-syntax/src/parser.rs`)
- [[crates/ferrotex-syntax/tests/resilience.rs|resilience.rs]] (`crates/ferrotex-syntax/tests/resilience.rs`)
- [[crates/ferrotexd/src/macros.rs|macros.rs]] (`crates/ferrotexd/src/macros.rs`)
- [[docs/adrs/0004-streaming-incremental-parsing.md|0004-streaming-incremental-parsing.md]] (`docs/adrs/0004-streaming-incremental-parsing.md`)
- [[docs/spec/formatting.md|formatting.md]] (`docs/spec/formatting.md`)
- [[docs/spec/incremental-analysis.md|incremental-analysis.md]] (`docs/spec/incremental-analysis.md`)

