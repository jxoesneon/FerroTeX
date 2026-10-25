---
title: "editors/vscode/src/engineValidator.ts"
type: source-code
language: typescript
crate: vscode-extension
layer: editor
lines: 118
tags:
  - crate/vscode-extension
  - lang/typescript
  - layer/editor
  - type/source-code
summary: "Source-code file (typescript) in vscode-extension subsystem (118 lines)."
---

# `editors/vscode/src/engineValidator.ts`

> [!info] File Overview
> - **Crate/Domain**: `vscode-extension` (editor)
> - **Language/Type**: `typescript` (source-code)
> - **Lines of Code**: `118`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **vscode-extension** module within the `editor` layer of the FerroTex architecture. Implements client-side editor capabilities, Language Client connection management, and UI commands.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `function` | `validateBuildEngine` |
| `function` | `validateSyncTeX` |

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-cli/tests/integration.rs|integration.rs]] (`crates/ferrotex-cli/tests/integration.rs`)
- [[crates/ferrotexd/src/build/latexmk.rs|latexmk.rs]] (`crates/ferrotexd/src/build/latexmk.rs`)
- [[crates/ferrotexd/src/build/tectonic.rs|tectonic.rs]] (`crates/ferrotexd/src/build/tectonic.rs`)
- [[crates/ferrotexd/src/synctex.rs|synctex.rs]] (`crates/ferrotexd/src/synctex.rs`)
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/spec/export.md|export.md]] (`docs/spec/export.md`)
- [[docs/spec/synctex.md|synctex.md]] (`docs/spec/synctex.md`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[editors/vscode/snippets/latex.json|latex.json]] (`editors/vscode/snippets/latex.json`)
- [[editors/vscode/src/extension.ts|extension.ts]] (`editors/vscode/src/extension.ts`)

