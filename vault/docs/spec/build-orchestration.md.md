---
title: "docs/spec/build-orchestration.md"
type: spec
language: markdown
crate: specification
layer: spec
lines: 75
tags:
  - crate/specification
  - lang/markdown
  - layer/spec
  - type/spec
summary: "Build Orchestration (latexmk reruns, biber/bibtex, makeindex) — Spec file (markdown) in specification subsystem (75 lines)."
---

# `docs/spec/build-orchestration.md`

> [!info] File Overview
> - **Crate/Domain**: `specification` (spec)
> - **Language/Type**: `markdown` (spec)
> - **Lines of Code**: `75`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **specification** module within the `spec` layer of the FerroTex architecture. Defines formal specifications, architectural decision records, or project documentation.

## Document Sections

- Build Orchestration (latexmk reruns, biber/bibtex, makeindex)
- Status
- Purpose
- Concepts
- Orchestration Modes
- Mode 1: Delegated (latexmk)
- Mode 2: Explicit Pipeline (direct)
- Rerun Policy
- Eventing
- Failure Modes

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/artifacts.rs|artifacts.rs]] (`crates/ferrotex-build/src/artifacts.rs`)
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-log/src/parser.rs|parser.rs]] (`crates/ferrotex-log/src/parser.rs`)
- [[crates/ferrotex-syntax/src/bibtex.rs|bibtex.rs]] (`crates/ferrotex-syntax/src/bibtex.rs`)
- [[crates/ferrotex-syntax/src/parser.rs|parser.rs]] (`crates/ferrotex-syntax/src/parser.rs`)
- [[crates/ferrotexd/src/build/latexmk.rs|latexmk.rs]] (`crates/ferrotexd/src/build/latexmk.rs`)
- [[docs/adrs/0005-universal-build-dag.md|0005-universal-build-dag.md]] (`docs/adrs/0005-universal-build-dag.md`)
- [[docs/adrs/index.md|index.md]] (`docs/adrs/index.md`)
- [[docs/architecture/index.md|index.md]] (`docs/architecture/index.md`)
- [[docs/development/index.md|index.md]] (`docs/development/index.md`)
- [[docs/index.md|index.md]] (`docs/index.md`)
- [[docs/research/index.md|index.md]] (`docs/research/index.md`)
- [[docs/spec/build-orchestration.md|build-orchestration.md]] (`docs/spec/build-orchestration.md`)
- [[docs/spec/configuration.md|configuration.md]] (`docs/spec/configuration.md`)
- [[docs/spec/index.md|index.md]] (`docs/spec/index.md`)
- [[editors/vscode/snippets/latex.json|latex.json]] (`editors/vscode/snippets/latex.json`)
- [[editors/vscode/src/test/suite/index.ts|index.ts]] (`editors/vscode/src/test/suite/index.ts`)

