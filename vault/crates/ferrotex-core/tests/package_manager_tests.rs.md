---
title: "crates/ferrotex-core/tests/package_manager_tests.rs"
type: source-code
language: rust
crate: ferrotex-core
layer: core
lines: 47
tags:
  - crate/ferrotex-core
  - lang/rust
  - layer/core
  - type/source-code
summary: "![allow(unsafe_code)] // env::set_var is unsafe in Rust 2024; needed for PATH-isolation tests. — Source-code file (rust) in ferrotex-core subsystem (47 lines)."
---

# `crates/ferrotex-core/tests/package_manager_tests.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotex-core` (core)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `47`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotex-core** module within the `core` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Document Sections

- ![allow(unsafe_code)] // env::set_var is unsafe in Rust 2024; needed for PATH-isolation tests.
- [test]

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-cli/tests/integration.rs|integration.rs]] (`crates/ferrotex-cli/tests/integration.rs`)
- [[crates/ferrotex-core/Cargo.toml|Cargo.toml]] (`crates/ferrotex-core/Cargo.toml`)
- [[crates/ferrotex-core/README.md|README.md]] (`crates/ferrotex-core/README.md`)
- [[crates/ferrotex-core/src/lib.rs|lib.rs]] (`crates/ferrotex-core/src/lib.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[docs/spec/tex-path-resolution.md|tex-path-resolution.md]] (`docs/spec/tex-path-resolution.md`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[package.json|package.json]] (`package.json`)

