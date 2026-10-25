---
title: "docs/development/setup.md"
type: doc
language: markdown
crate: documentation
layer: docs
lines: 160
tags:
  - crate/documentation
  - lang/markdown
  - layer/docs
  - type/doc
summary: "Development Setup — Doc file (markdown) in documentation subsystem (160 lines)."
---

# `docs/development/setup.md`

> [!info] File Overview
> - **Crate/Domain**: `documentation` (docs)
> - **Language/Type**: `markdown` (doc)
> - **Lines of Code**: `160`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **documentation** module within the `docs` layer of the FerroTex architecture. Defines formal specifications, architectural decision records, or project documentation.

## Document Sections

- Development Setup
- Prerequisites
- System
- Required System Libraries
- Installation by Platform
- Ubuntu/Debian
- macOS
- Edit .cargo/config.toml and update harfbuzz VERSION to match:
- Windows (vcpkg)
- Docker Alternative (Recommended for CI reproduction)
- Rust
- Node.js (VS Code extension)
- TeX Toolchain
- Repository Conventions (planned)
- Local Workflow (intended)

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
- [[.cargo/config.toml.example|config.toml.example]] (`.cargo/config.toml.example`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-log/src/parser.rs|parser.rs]] (`crates/ferrotex-log/src/parser.rs`)
- [[crates/ferrotex-syntax/src/parser.rs|parser.rs]] (`crates/ferrotex-syntax/src/parser.rs`)
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/spec/configuration.md|configuration.md]] (`docs/spec/configuration.md`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[editors/vscode/src/extension.ts|extension.ts]] (`editors/vscode/src/extension.ts`)
- [[package.json|package.json]] (`package.json`)
- [[scripts/ci-run.sh|ci-run.sh]] (`scripts/ci-run.sh`)

