---
title: "crates/ferrotex-build/src/error.rs"
type: source-code
language: rust
crate: ferrotex-build
layer: build
lines: 821
tags:
  - crate/ferrotex-build
  - lang/rust
  - layer/build
  - type/source-code
summary: "[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)] — Source-code file (rust) in ferrotex-build subsystem (821 lines)."
---

# `crates/ferrotex-build/src/error.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotex-build` (build)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `821`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotex-build** module within the `build` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `struct` | `SourceLocation` |
| `struct` | `SourceSpan` |
| `struct` | `AnalysisContext` |
| `struct` | `ConfigurationErrorDetails` |
| `enum` | `FerroTeXError` |
| `type` | `FerroTeXResult` |
| `fn` | `new` |
| `fn` | `start` |
| `fn` | `new` |
| `fn` | `point` |
| `fn` | `new` |
| `fn` | `with_hint` |
| `fn` | `new` |
| `fn` | `with_actual` |
| `fn` | `parse_error` |
| `fn` | `parse_error_with_snippet` |
| `fn` | `analysis_error` |
| `fn` | `analysis_error_with_location` |
| `fn` | `io_error` |
| `fn` | `with_path` |
| `fn` | `configuration_error` |
| `fn` | `generic_error` |
| `fn` | `with_location` |
| `fn` | `message` |
| `fn` | `location` |
| ... | *(5 additional symbols)* |

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/Cargo.toml|Cargo.toml]] (`crates/ferrotex-build/Cargo.toml`)
- [[crates/ferrotex-build/README.md|README.md]] (`crates/ferrotex-build/README.md`)
- [[crates/ferrotex-build/src/lib.rs|lib.rs]] (`crates/ferrotex-build/src/lib.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-log/src/parser.rs|parser.rs]] (`crates/ferrotex-log/src/parser.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[crates/ferrotex-math-semantics/src/analysis.rs|analysis.rs]] (`crates/ferrotex-math-semantics/src/analysis.rs`)
- [[crates/ferrotex-syntax/src/parser.rs|parser.rs]] (`crates/ferrotex-syntax/src/parser.rs`)
- [[docs/_config.yml|_config.yml]] (`docs/_config.yml`)
- [[docs/_sass/custom/custom.scss|custom.scss]] (`docs/_sass/custom/custom.scss`)
- [[docs/adrs/0005-universal-build-dag.md|0005-universal-build-dag.md]] (`docs/adrs/0005-universal-build-dag.md`)
- [[docs/adrs/index.md|index.md]] (`docs/adrs/index.md`)
- [[docs/architecture/index.md|index.md]] (`docs/architecture/index.md`)
- [[docs/development/index.md|index.md]] (`docs/development/index.md`)
- [[docs/index.md|index.md]] (`docs/index.md`)
- [[docs/research/index.md|index.md]] (`docs/research/index.md`)
- [[docs/spec/build-orchestration.md|build-orchestration.md]] (`docs/spec/build-orchestration.md`)
- [[docs/spec/configuration.md|configuration.md]] (`docs/spec/configuration.md`)
- [[docs/spec/index.md|index.md]] (`docs/spec/index.md`)
- [[editors/vscode/snippets/latex.json|latex.json]] (`editors/vscode/snippets/latex.json`)
- [[editors/vscode/src/test/suite/index.ts|index.ts]] (`editors/vscode/src/test/suite/index.ts`)

