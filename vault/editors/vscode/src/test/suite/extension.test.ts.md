---
title: "editors/vscode/src/test/suite/extension.test.ts"
type: source-code
language: typescript
crate: vscode-extension
layer: editor
lines: 620
tags:
  - crate/vscode-extension
  - lang/typescript
  - layer/editor
  - type/source-code
summary: "Source-code file (typescript) in vscode-extension subsystem (620 lines)."
---

# `editors/vscode/src/test/suite/extension.test.ts`

> [!info] File Overview
> - **Crate/Domain**: `vscode-extension` (editor)
> - **Language/Type**: `typescript` (source-code)
> - **Lines of Code**: `620`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **vscode-extension** module within the `editor` layer of the FerroTex architecture. Implements client-side editor capabilities, Language Client connection management, and UI commands.

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[crates/ferrotex-math-semantics/src/analysis.rs|analysis.rs]] (`crates/ferrotex-math-semantics/src/analysis.rs`)
- [[crates/ferrotexd/src/build/latexmk.rs|latexmk.rs]] (`crates/ferrotexd/src/build/latexmk.rs`)
- [[crates/ferrotexd/src/build/tectonic.rs|tectonic.rs]] (`crates/ferrotexd/src/build/tectonic.rs`)
- [[crates/ferrotexd/src/hover.rs|hover.rs]] (`crates/ferrotexd/src/hover.rs`)
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/adrs/index.md|index.md]] (`docs/adrs/index.md`)
- [[docs/architecture/index.md|index.md]] (`docs/architecture/index.md`)
- [[docs/development/index.md|index.md]] (`docs/development/index.md`)
- [[docs/index.md|index.md]] (`docs/index.md`)
- [[docs/research/index.md|index.md]] (`docs/research/index.md`)
- [[docs/spec/completion.md|completion.md]] (`docs/spec/completion.md`)
- [[docs/spec/index.md|index.md]] (`docs/spec/index.md`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[editors/vscode/snippets/latex.json|latex.json]] (`editors/vscode/snippets/latex.json`)
- [[editors/vscode/src/extension.ts|extension.ts]] (`editors/vscode/src/extension.ts`)
- [[editors/vscode/src/imagePaste.ts|imagePaste.ts]] (`editors/vscode/src/imagePaste.ts`)
- [[editors/vscode/src/test/suite/index.ts|index.ts]] (`editors/vscode/src/test/suite/index.ts`)
- [[package.json|package.json]] (`package.json`)

