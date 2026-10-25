---
title: "crates/ferrotex-syntax/src/lib.rs"
type: source-code
language: rust
crate: ferrotex-syntax
layer: syntax
lines: 253
tags:
  - crate/ferrotex-syntax
  - lang/rust
  - layer/syntax
  - type/source-code
summary: "[cfg(test)] — Source-code file (rust) in ferrotex-syntax subsystem (253 lines)."
---

# `crates/ferrotex-syntax/src/lib.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotex-syntax` (syntax)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `253`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotex-syntax** module within the `syntax` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `enum` | `SyntaxKind` |
| `enum` | `FerroTexLanguage` |
| `type` | `SyntaxNode` |
| `type` | `SyntaxToken` |
| `type` | `SyntaxElement` |

## Graph Links & Dependencies

- [[.github/workflows/coverage.yml|coverage.yml]] (`.github/workflows/coverage.yml`)
- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-log/src/parser.rs|parser.rs]] (`crates/ferrotex-log/src/parser.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[crates/ferrotex-syntax/Cargo.toml|Cargo.toml]] (`crates/ferrotex-syntax/Cargo.toml`)
- [[crates/ferrotex-syntax/README.md|README.md]] (`crates/ferrotex-syntax/README.md`)
- [[crates/ferrotex-syntax/src/additional_tests.rs|additional_tests.rs]] (`crates/ferrotex-syntax/src/additional_tests.rs`)
- [[crates/ferrotex-syntax/src/bibtex.rs|bibtex.rs]] (`crates/ferrotex-syntax/src/bibtex.rs`)
- [[crates/ferrotex-syntax/src/coverage_tests.rs|coverage_tests.rs]] (`crates/ferrotex-syntax/src/coverage_tests.rs`)
- [[crates/ferrotex-syntax/src/lexer.rs|lexer.rs]] (`crates/ferrotex-syntax/src/lexer.rs`)
- [[crates/ferrotex-syntax/src/parser.rs|parser.rs]] (`crates/ferrotex-syntax/src/parser.rs`)
- [[docs/adrs/0004-streaming-incremental-parsing.md|0004-streaming-incremental-parsing.md]] (`docs/adrs/0004-streaming-incremental-parsing.md`)
- [[docs/spec/formatting.md|formatting.md]] (`docs/spec/formatting.md`)
- [[docs/spec/incremental-analysis.md|incremental-analysis.md]] (`docs/spec/incremental-analysis.md`)

