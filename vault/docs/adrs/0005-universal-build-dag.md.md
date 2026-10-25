---
title: "docs/adrs/0005-universal-build-dag.md"
type: adr
language: markdown
crate: architecture
layer: adr
lines: 38
tags:
  - crate/architecture
  - lang/markdown
  - layer/adr
  - type/adr
summary: "ADR 0005: Universal Build DAG & Content-Addressing — Adr file (markdown) in architecture subsystem (38 lines)."
---

# `docs/adrs/0005-universal-build-dag.md`

> [!info] File Overview
> - **Crate/Domain**: `architecture` (adr)
> - **Language/Type**: `markdown` (adr)
> - **Lines of Code**: `38`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **architecture** module within the `adr` layer of the FerroTex architecture. Defines formal specifications, architectural decision records, or project documentation.

## Document Sections

- ADR 0005: Universal Build DAG & Content-Addressing
- Context
- Decision
- Alternatives Considered
- Consequences

## Graph Links & Dependencies

- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/artifacts.rs|artifacts.rs]] (`crates/ferrotex-build/src/artifacts.rs`)
- [[crates/ferrotexd/src/build/latexmk.rs|latexmk.rs]] (`crates/ferrotexd/src/build/latexmk.rs`)
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/adrs/0005-universal-build-dag.md|0005-universal-build-dag.md]] (`docs/adrs/0005-universal-build-dag.md`)
- [[docs/research/reproducibility.md|reproducibility.md]] (`docs/research/reproducibility.md`)
- [[docs/spec/build-orchestration.md|build-orchestration.md]] (`docs/spec/build-orchestration.md`)
- [[editors/vscode/snippets/latex.json|latex.json]] (`editors/vscode/snippets/latex.json`)

