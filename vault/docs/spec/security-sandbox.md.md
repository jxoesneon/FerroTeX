---
title: "docs/spec/security-sandbox.md"
type: spec
language: markdown
crate: specification
layer: spec
lines: 79
tags:
  - crate/specification
  - lang/markdown
  - layer/spec
  - type/spec
summary: "Safe-TeX Sandbox Specification — Spec file (markdown) in specification subsystem (79 lines)."
---

# `docs/spec/security-sandbox.md`

> [!info] File Overview
> - **Crate/Domain**: `specification` (spec)
> - **Language/Type**: `markdown` (spec)
> - **Lines of Code**: `79`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **specification** module within the `spec` layer of the FerroTex architecture. Defines formal specifications, architectural decision records, or project documentation.

## Document Sections

- Safe-TeX Sandbox Specification
- Status
- Purpose
- Core Principles
- Capabilities-Based Security Model
- 1. File System Capabilities
- 2. Execution Capabilities (`shell-escape`)
- 3. Network Capabilities
- 4. Environment Capabilities
- Granular Permissions for `shell-escape`
- Command Allowlist (`exec:restricted`)
- Zero-Trust Implementation
- User Interaction
- Interactions

## Graph Links & Dependencies

- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-syntax/src/bibtex.rs|bibtex.rs]] (`crates/ferrotex-syntax/src/bibtex.rs`)
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/adrs/index.md|index.md]] (`docs/adrs/index.md`)
- [[docs/architecture/index.md|index.md]] (`docs/architecture/index.md`)
- [[docs/development/index.md|index.md]] (`docs/development/index.md`)
- [[docs/index.md|index.md]] (`docs/index.md`)
- [[docs/research/index.md|index.md]] (`docs/research/index.md`)
- [[docs/spec/index.md|index.md]] (`docs/spec/index.md`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[editors/vscode/src/test/suite/index.ts|index.ts]] (`editors/vscode/src/test/suite/index.ts`)
- [[package.json|package.json]] (`package.json`)

