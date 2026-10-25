---
title: "crates/ferrotex-core/src/package_manager/mod.rs"
type: source-code
language: rust
crate: ferrotex-core
layer: core
lines: 475
tags:
  - crate/ferrotex-core
  - lang/rust
  - layer/core
  - type/source-code
summary: "[derive(Debug, Clone, PartialEq, Eq)] — Source-code file (rust) in ferrotex-core subsystem (475 lines)."
---

# `crates/ferrotex-core/src/package_manager/mod.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotex-core` (core)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `475`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotex-core** module within the `core` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `struct` | `InstallStatus` |
| `struct` | `RealCommandExecutor` |
| `struct` | `MockCommandExecutor` |
| `struct` | `TlmgrBackend` |
| `struct` | `MiktexBackend` |
| `struct` | `NoOpBackend` |
| `struct` | `PackageManager` |
| `enum` | `InstallState` |
| `trait` | `CommandExecutor` |
| `trait` | `PackageBackend` |
| `fn` | `new` |
| `fn` | `with_executor` |
| `fn` | `new` |
| `fn` | `with_executor` |
| `fn` | `new` |
| `fn` | `with_backend` |
| `fn` | `install` |
| `fn` | `search` |
| `fn` | `is_available` |
| `fn` | `get_ctan_link` |

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-core/Cargo.toml|Cargo.toml]] (`crates/ferrotex-core/Cargo.toml`)
- [[crates/ferrotex-core/README.md|README.md]] (`crates/ferrotex-core/README.md`)
- [[crates/ferrotex-core/src/lib.rs|lib.rs]] (`crates/ferrotex-core/src/lib.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/ctan_db.rs|ctan_db.rs]] (`crates/ferrotex-core/src/package_manager/ctan_db.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[docs/_sass/custom/custom.scss|custom.scss]] (`docs/_sass/custom/custom.scss`)
- [[docs/development/testing.md|testing.md]] (`docs/development/testing.md`)
- [[docs/spec/tex-path-resolution.md|tex-path-resolution.md]] (`docs/spec/tex-path-resolution.md`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[package.json|package.json]] (`package.json`)

