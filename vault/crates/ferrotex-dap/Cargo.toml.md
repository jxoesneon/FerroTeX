---
title: "crates/ferrotex-dap/Cargo.toml"
type: manifest
language: toml
crate: ferrotex-dap
layer: dap
lines: 57
tags:
  - crate/ferrotex-dap
  - lang/toml
  - layer/dap
  - type/manifest
summary: "crossbeam-channel = "0.5" # Likely needed for message passing — Manifest file (toml) in ferrotex-dap subsystem (57 lines)."
---

# `crates/ferrotex-dap/Cargo.toml`

> [!info] File Overview
> - **Crate/Domain**: `ferrotex-dap` (dap)
> - **Language/Type**: `toml` (manifest)
> - **Lines of Code**: `57`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotex-dap** module within the `dap` layer of the FerroTex architecture. Contains project configuration, workflows, or developer tooling.

## Document Sections

- crossbeam-channel = "0.5" # Likely needed for message passing
- dap-types? Writing raw for now to avoid bulky deps
- [features]
- default = []
- jxoesneon-tectonic-engine = ["jxoesneon-tectonic", "jxoesneon-tectonic-status_base", "jxoesneon-tectonic-io_base", "jxoesneon-tectonic-bridge_core"]

## Graph Links & Dependencies

- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-dap/README.md|README.md]] (`crates/ferrotex-dap/README.md`)
- [[crates/ferrotex-dap/src/lib.rs|lib.rs]] (`crates/ferrotex-dap/src/lib.rs`)
- [[crates/ferrotex-dap/src/main.rs|main.rs]] (`crates/ferrotex-dap/src/main.rs`)
- [[crates/ferrotexd/src/build/tectonic.rs|tectonic.rs]] (`crates/ferrotexd/src/build/tectonic.rs`)
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/architecture/tectonic-io.md|tectonic-io.md]] (`docs/architecture/tectonic-io.md`)
- [[docs/spec/dap.md|dap.md]] (`docs/spec/dap.md`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[editors/vscode/snippets/latex.json|latex.json]] (`editors/vscode/snippets/latex.json`)
- [[package.json|package.json]] (`package.json`)

