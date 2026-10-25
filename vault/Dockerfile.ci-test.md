---
title: "Dockerfile.ci-test"
type: file
language: text
crate: workspace-root
layer: root
lines: 38
tags:
  - crate/workspace-root
  - lang/text
  - layer/root
  - type/file
summary: "CI Testing Dockerfile for FerroTeX — File file (text) in workspace-root subsystem (38 lines)."
---

# `Dockerfile.ci-test`

> [!info] File Overview
> - **Crate/Domain**: `workspace-root` (root)
> - **Language/Type**: `text` (file)
> - **Lines of Code**: `38`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **workspace-root** module within the `root` layer of the FerroTex architecture. Contains project configuration, workflows, or developer tooling.

## Document Sections

- CI Testing Dockerfile for FerroTeX
- Based on Ubuntu 22.04 to match GitHub Actions CI environment
- Prevent interactive prompts during package installation
- Install system dependencies matching GitHub Actions CI
- Install Rust stable toolchain
- Install cargo tools for CI
- Install mold linker for faster builds
- Default command

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[package.json|package.json]] (`package.json`)

