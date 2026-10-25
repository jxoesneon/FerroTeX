---
title: "docs/spec/log-grammar.md"
type: spec
language: markdown
crate: specification
layer: spec
lines: 93
tags:
  - crate/specification
  - lang/markdown
  - layer/spec
  - type/spec
summary: "Log Grammar and Parsing Rules — Spec file (markdown) in specification subsystem (93 lines)."
---

# `docs/spec/log-grammar.md`

> [!info] File Overview
> - **Crate/Domain**: `specification` (spec)
> - **Language/Type**: `markdown` (spec)
> - **Lines of Code**: `93`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **specification** module within the `spec` layer of the FerroTex architecture. Defines formal specifications, architectural decision records, or project documentation.

## Document Sections

- Log Grammar and Parsing Rules
- Status
- Problem Statement
- Normalization
- Token Classes (informal)
- File Path Recognition
- Path Heuristics (MUST/SHOULD)
- Line Wrapping Handling
- Error Blocks
- Recovery
- Output

## Graph Links & Dependencies

- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-log/src/parser.rs|parser.rs]] (`crates/ferrotex-log/src/parser.rs`)
- [[crates/ferrotex-syntax/src/parser.rs|parser.rs]] (`crates/ferrotex-syntax/src/parser.rs`)
- [[docs/adrs/0001-typed-event-ir.md|0001-typed-event-ir.md]] (`docs/adrs/0001-typed-event-ir.md`)
- [[docs/spec/log-event-ir.md|log-event-ir.md]] (`docs/spec/log-event-ir.md`)
- [[editors/vscode/src/extension.ts|extension.ts]] (`editors/vscode/src/extension.ts`)

