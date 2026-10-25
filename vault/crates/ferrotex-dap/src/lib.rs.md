---
title: "crates/ferrotex-dap/src/lib.rs"
type: source-code
language: rust
crate: ferrotex-dap
layer: dap
lines: 942
tags:
  - crate/ferrotex-dap
  - lang/rust
  - layer/dap
  - type/source-code
summary: "![allow(unexpected_cfgs)] — Source-code file (rust) in ferrotex-dap subsystem (942 lines)."
---

# `crates/ferrotex-dap/src/lib.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotex-dap` (dap)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `942`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotex-dap** module within the `dap` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `struct` | `DebugSession` |
| `struct` | `TectonicAdapter` |
| `enum` | `ProtocolMessage` |
| `trait` | `DebugAdapter` |
| `fn` | `new` |
| `fn` | `run_loop` |
| `fn` | `run_session` |
| `fn` | `new` |
| `fn` | `run_mock_session` |
| `fn` | `run_mock_session_with_io` |
| `fn` | `run_jxoesneon_tectonic_session` |

## Graph Links & Dependencies

- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-dap/Cargo.toml|Cargo.toml]] (`crates/ferrotex-dap/Cargo.toml`)
- [[crates/ferrotex-dap/README.md|README.md]] (`crates/ferrotex-dap/README.md`)
- [[crates/ferrotex-dap/src/debug_session.rs|debug_session.rs]] (`crates/ferrotex-dap/src/debug_session.rs`)
- [[crates/ferrotex-dap/src/main.rs|main.rs]] (`crates/ferrotex-dap/src/main.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[crates/ferrotexd/src/build/tectonic.rs|tectonic.rs]] (`crates/ferrotexd/src/build/tectonic.rs`)
- [[docs/spec/dap.md|dap.md]] (`docs/spec/dap.md`)

