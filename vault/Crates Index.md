---
title: "Crates Index"
type: hub
layer: crates
tags:
  - type/hub
  - layer/crates
summary: "Index of all 10 Rust workspace crates comprising FerroTex."
---

# FerroTex Crates Ecosystem

The FerroTex workspace is partitioned into 10 specialized crates designed around separation of concerns, fast incremental compilation, and modularity:

| Crate | Role | Manifest | Entry Point |
|---|---|---|---|
| **ferrotexd** | Language Server Protocol (LSP) Daemon | [[crates/ferrotexd/Cargo.toml\|Cargo.toml]] | [[crates/ferrotexd/src/main.rs\|main.rs]] / [[crates/ferrotexd/src/lib.rs\|lib.rs]] |
| **ferrotex-syntax** | Resilient AST & Streaming Parser | [[crates/ferrotex-syntax/Cargo.toml\|Cargo.toml]] | [[crates/ferrotex-syntax/src/lib.rs\|lib.rs]] / [[crates/ferrotex-syntax/src/parser.rs\|parser.rs]] |
| **ferrotex-core** | Core validators, CTAN & Math engine | [[crates/ferrotex-core/Cargo.toml\|Cargo.toml]] | [[crates/ferrotex-core/src/lib.rs\|lib.rs]] |
| **ferrotex-build** | Compiler execution & artifacts (Tectonic/latexmk) | [[crates/ferrotex-build/Cargo.toml\|Cargo.toml]] | [[crates/ferrotex-build/src/lib.rs\|lib.rs]] |
| **ferrotex-log** | Typed Log Event IR & parser | [[crates/ferrotex-log/Cargo.toml\|Cargo.toml]] | [[crates/ferrotex-log/src/lib.rs\|lib.rs]] / [[crates/ferrotex-log/src/ir.rs\|ir.rs]] |
| **ferrotex-math-semantics** | Mathematical equation semantics & delimiters | [[crates/ferrotex-math-semantics/Cargo.toml\|Cargo.toml]] | [[crates/ferrotex-math-semantics/src/lib.rs\|lib.rs]] |
| **ferrotex-package** | Package discovery, CTAN scanning & introspection | [[crates/ferrotex-package/Cargo.toml\|Cargo.toml]] | [[crates/ferrotex-package/src/lib.rs\|lib.rs]] |
| **ferrotex-analysis** | Advanced semantic analysis & linter | [[crates/ferrotex-analysis/Cargo.toml\|Cargo.toml]] | [[crates/ferrotex-analysis/src/lib.rs\|lib.rs]] |
| **ferrotex-dap** | Debug Adapter Protocol implementation | [[crates/ferrotex-dap/Cargo.toml\|Cargo.toml]] | [[crates/ferrotex-dap/src/lib.rs\|lib.rs]] / [[crates/ferrotex-dap/src/main.rs\|main.rs]] |
| **ferrotex-cli** | FerroTex developer command-line interface | [[crates/ferrotex-cli/Cargo.toml\|Cargo.toml]] | [[crates/ferrotex-cli/src/main.rs\|main.rs]] |

Back to [[FerroTex Overview|System Overview]].
