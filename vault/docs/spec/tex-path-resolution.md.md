---
title: "docs/spec/tex-path-resolution.md"
type: spec
language: markdown
crate: specification
layer: spec
lines: 71
tags:
  - crate/specification
  - lang/markdown
  - layer/spec
  - type/spec
summary: "TeX Path Resolution (kpathsea / TEXINPUTS) — Spec file (markdown) in specification subsystem (71 lines)."
---

# `docs/spec/tex-path-resolution.md`

> [!info] File Overview
> - **Crate/Domain**: `specification` (spec)
> - **Language/Type**: `markdown` (spec)
> - **Lines of Code**: `71`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **specification** module within the `spec` layer of the FerroTex architecture. Defines formal specifications, architectural decision records, or project documentation.

## Document Sections

- TeX Path Resolution (kpathsea / TEXINPUTS)
- Status
- Purpose
- Resolution Modes
- Mode A: Workspace-Relative (Baseline)
- Mode B: Environment-Aware (Target)
- Mode C: kpathsea-assisted (Opt-in)
- Security Constraints
- Precedence Rules (recommended)
- Diagnostics
- Interactions

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/spec/build-orchestration.md|build-orchestration.md]] (`docs/spec/build-orchestration.md`)
- [[docs/spec/project-model.md|project-model.md]] (`docs/spec/project-model.md`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[editors/vscode/src/extension.ts|extension.ts]] (`editors/vscode/src/extension.ts`)
- [[package.json|package.json]] (`package.json`)

