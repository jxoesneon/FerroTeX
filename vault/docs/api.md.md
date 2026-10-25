---
title: "docs/api.md"
type: doc
language: markdown
crate: documentation
layer: docs
lines: 989
tags:
  - crate/documentation
  - lang/markdown
  - layer/docs
  - type/doc
summary: "FerroTeX Public API Reference — Doc file (markdown) in documentation subsystem (989 lines)."
---

# `docs/api.md`

> [!info] File Overview
> - **Crate/Domain**: `documentation` (docs)
> - **Language/Type**: `markdown` (doc)
> - **Lines of Code**: `989`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **documentation** module within the `docs` layer of the FerroTex architecture. Defines formal specifications, architectural decision records, or project documentation.

## Document Sections

- FerroTeX Public API Reference
- Table of Contents
- Overview
- Crate Dependency Graph
- ferrotex-syntax
- Primary Entry Points
- `parse(source: &str) -> ParseResult`
- Key Types
- `SyntaxKind`
- `SyntaxNode`
- `SyntaxToken`
- `SyntaxElement`
- Re-exports from `rowan`
- Error Types
- Modules

## Graph Links & Dependencies

- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/artifacts.rs|artifacts.rs]] (`crates/ferrotex-build/src/artifacts.rs`)
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-cli/tests/integration.rs|integration.rs]] (`crates/ferrotex-cli/tests/integration.rs`)
- [[crates/ferrotex-log/src/parser.rs|parser.rs]] (`crates/ferrotex-log/src/parser.rs`)
- [[crates/ferrotex-math-semantics/src/analysis.rs|analysis.rs]] (`crates/ferrotex-math-semantics/src/analysis.rs`)
- [[crates/ferrotex-math-semantics/src/delimiters.rs|delimiters.rs]] (`crates/ferrotex-math-semantics/src/delimiters.rs`)
- [[crates/ferrotex-package/src/scanner.rs|scanner.rs]] (`crates/ferrotex-package/src/scanner.rs`)
- [[crates/ferrotex-syntax/src/bibtex.rs|bibtex.rs]] (`crates/ferrotex-syntax/src/bibtex.rs`)
- [[crates/ferrotex-syntax/src/lexer.rs|lexer.rs]] (`crates/ferrotex-syntax/src/lexer.rs`)
- [[crates/ferrotex-syntax/src/parser.rs|parser.rs]] (`crates/ferrotex-syntax/src/parser.rs`)
- [[crates/ferrotexd/src/build/tectonic.rs|tectonic.rs]] (`crates/ferrotexd/src/build/tectonic.rs`)
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/adrs/index.md|index.md]] (`docs/adrs/index.md`)
- [[docs/architecture/index.md|index.md]] (`docs/architecture/index.md`)
- [[docs/architecture/overview.md|overview.md]] (`docs/architecture/overview.md`)
- [[docs/development/index.md|index.md]] (`docs/development/index.md`)
- [[docs/development/testing.md|testing.md]] (`docs/development/testing.md`)
- [[docs/extension/overview.md|overview.md]] (`docs/extension/overview.md`)
- [[docs/index.md|index.md]] (`docs/index.md`)
- [[docs/research/index.md|index.md]] (`docs/research/index.md`)
- [[docs/spec/export.md|export.md]] (`docs/spec/export.md`)
- [[docs/spec/index.md|index.md]] (`docs/spec/index.md`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[editors/vscode/snippets/latex.json|latex.json]] (`editors/vscode/snippets/latex.json`)
- [[editors/vscode/src/test/suite/index.ts|index.ts]] (`editors/vscode/src/test/suite/index.ts`)
- [[package.json|package.json]] (`package.json`)

