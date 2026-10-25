---
title: "Cargo.toml"
type: config
language: toml
crate: workspace
layer: workspace
tags:
  - type/config
  - workspace/root
summary: "Root Cargo workspace manifest for FerroTex, coordinating 10 core crates, workspace-wide metadata, compiler policies, Clippy lints, and optimized release compilation profiles."
---

# `Cargo.toml`

## Purpose & Architecture Role
The root `Cargo.toml` file defines the top-level Cargo workspace for the FerroTex project. It acts as the central orchestrator for the entire Rust codebase, establishing shared package metadata (such as versioning, license, and Rust edition), defining the compilation resolver strategy, enforcing consistent compiler warnings and Clippy lints, and configuring performance-oriented release profiles.

By unifying all sub-crates under a single workspace, it guarantees reproducible builds, shared dependency trees, and unified developer tooling (`cargo test`, `cargo clippy`, `cargo fmt`) across all system layers.

---

## Key Manifest Sections & Configurations

### 1. Workspace Configuration (`[workspace]`)
- **Resolver**: `resolver = "2"` — Employs Cargo's version 2 feature resolver to prevent unwanted feature unification across dev-dependencies and build-dependencies.
- **Workspace Members**:
  - `crates/ferrotex-cli`
  - `crates/ferrotex-core`
  - `crates/ferrotex-log`
  - `crates/ferrotex-syntax`
  - `crates/ferrotexd`
  - `crates/ferrotex-package`
  - `crates/ferrotex-math-semantics`
  - `crates/ferrotex-analysis`
  - `crates/ferrotex-build`
  - `crates/ferrotex-dap`
  - `fuzz` (Targeted fuzz testing crate)

### 2. Workspace Package Metadata (`[workspace.package]`)
Shared baseline attributes inherited by member crates using `.workspace = true`:
- **Version**: `0.25.0`
- **Edition**: `2024` (Leverages Rust 2024 idioms and language features)
- **Rust Version (MSRV)**: `1.85`
- **License**: `Apache-2.0 OR MIT`
- **Repository**: `https://github.com/jxoesneon/FerroTeX`
- **Homepage**: `https://jxoesneon.github.io/FerroTeX/`

### 3. Workspace Lints (`[workspace.lints]`)
Enforces strict workspace-wide code hygiene and safety standards:
- **Rust Compiler Lints (`[workspace.lints.rust]`)**:
  - `unsafe_code = "warn"`: Discourages the use of `unsafe` blocks, ensuring high memory safety unless explicitly justified.
- **Clippy Lints (`[workspace.lints.clippy]`)**:
  - `all = { level = "warn", priority = -1 }`: Warns on all default Clippy rules across all crates.
  - `manual_strip = "warn"`: Enforces use of `strip_prefix` and `strip_suffix` standard library methods.
  - `assertions_on_constants = "warn"`: Prevents redundant runtime assertions on compile-time constants.

### 4. Release Profile Optimization (`[profile.release]`)
Configured to maximize runtime execution speed and minimize binary size for LSP daemon and CLI distribution:
- **`lto = true`**: Enables Link-Time Optimization across crate boundaries to facilitate whole-program analysis and dead-code stripping.
- **`strip = true`**: Automatically strips symbol tables and debug symbols from final release binaries.
- **`codegen-units = 1`**: Restricts code generation to a single unit per crate, eliminating parallelism in code emission to allow deeper cross-module optimizations and better inlining.

---

## Dependencies & Workspace Crates

The workspace brings together 10 modular crates forming the FerroTex engine:

- [[crates/ferrotex-core/Cargo.toml|ferrotex-core]]: Foundation primitives, common document structures, diagnostics primitives, and cancellation tokens.
- [[crates/ferrotex-syntax/Cargo.toml|ferrotex-syntax]]: LaTeX and BibTeX lexer, parser, lossless Concrete Syntax Tree (CST), and syntax token definitions.
- [[crates/ferrotex-analysis/Cargo.toml|ferrotex-analysis]]: Incremental semantic analysis, symbol indexer, type checking, and diagnostic emission.
- [[crates/ferrotex-build/Cargo.toml|ferrotex-build]]: Universal build DAG, engine runner adapters (Tectonic, TeX Live, tectonic-stepping-io), and auxiliary artifact tracking.
- [[crates/ferrotex-cli/Cargo.toml|ferrotex-cli]]: Command-line interface frontend providing user commands for formatting, compiling, and linting.
- [[crates/ferrotex-dap/Cargo.toml|ferrotex-dap]]: Debug Adapter Protocol implementation enabling interactive LaTeX debugging and execution inspection.
- [[crates/ferrotex-log/Cargo.toml|ferrotex-log]]: Typed Event IR parser transforming raw LaTeX compiler logs and terminal stdout into structured events.
- [[crates/ferrotex-math-semantics/Cargo.toml|ferrotex-math-semantics]]: Mathematical formula parser, semantic AST, and verification engine for LaTeX math environments.
- [[crates/ferrotex-package/Cargo.toml|ferrotex-package]]: Package registry scanner, CTAN manifest resolver, and macro environment introspection.
- [[crates/ferrotexd/Cargo.toml|ferrotexd]]: Core Language Server Protocol (LSP) daemon integrating syntax, analysis, build, and formatting services.

---

## Consumers & Dependents

- [[FerroTex Overview.md|FerroTex Overview]]: Central Obsidian vault navigation hub.
- [[README.md|FerroTex README]]: Project entry point and developer instructions.
- [[ROADMAP.md|Roadmap]]: Architectural milestones, feature stages, and release planning.
- **Cargo Toolchain**: Invoked by commands including `cargo build`, `cargo check`, `cargo test`, and `cargo clippy`.
- **Continuous Integration (CI)**: GitHub Actions workflows verifying compilation, test suites, and formatting across platforms.

---

## Related Specs & ADRs

- [[docs/spec/project-model.md|Project Model Spec]]: Defines project model structures and module interaction boundaries.
- [[docs/spec/language-platform.md|Language Platform Spec]]: Comprehensive platform architecture for language servers and tooling.
- [[docs/spec/performance-slos.md|Performance SLOs]]: Service-level objectives governing parse and diagnostics latency targets justifying release optimization profiles.
- [[docs/adrs/0003-thin-extension-thick-server.md|ADR-0003: Thin Extension, Thick Server]]: Architectural decision to consolidate heavy computation in Rust workspace crates.
- [[docs/adrs/0005-universal-build-dag.md|ADR-0005: Universal Build DAG]]: Build orchestration architecture decoupled into modular crates.
