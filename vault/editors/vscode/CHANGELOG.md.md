---
title: "editors/vscode/CHANGELOG.md"
type: doc
language: markdown
crate: vscode-extension
layer: editor
lines: 369
tags:
  - crate/vscode-extension
  - lang/markdown
  - layer/editor
  - type/doc
summary: "Changelog — Doc file (markdown) in vscode-extension subsystem (369 lines)."
---

# `editors/vscode/CHANGELOG.md`

> [!info] File Overview
> - **Crate/Domain**: `vscode-extension` (editor)
> - **Language/Type**: `markdown` (doc)
> - **Lines of Code**: `369`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **vscode-extension** module within the `editor` layer of the FerroTex architecture. Defines formal specifications, architectural decision records, or project documentation.

## Document Sections

- Changelog
- [Unreleased]
- [0.24.0] - 2026-04-09
- Added
- Changed
- [0.23.0] - 2026-04-04
- Added
- Changed
- Fixed
- [0.22.0] - 2026-01-16
- Added
- Fixed
- Changed
- [0.21.0] - 2026-01-03
- Added

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
- [[.github/workflows/coverage.yml|coverage.yml]] (`.github/workflows/coverage.yml`)
- [[.github/workflows/release.yml|release.yml]] (`.github/workflows/release.yml`)
- [[Cargo.lock|Cargo.lock]] (`Cargo.lock`)
- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[Dockerfile.ci-test|Dockerfile.ci-test]] (`Dockerfile.ci-test`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[LICENSE|LICENSE]] (`LICENSE`)
- [[crates/ferrotex-analysis/Cargo.toml|Cargo.toml]] (`crates/ferrotex-analysis/Cargo.toml`)
- [[crates/ferrotex-build/Cargo.toml|Cargo.toml]] (`crates/ferrotex-build/Cargo.toml`)
- [[crates/ferrotex-build/src/artifacts.rs|artifacts.rs]] (`crates/ferrotex-build/src/artifacts.rs`)
- [[crates/ferrotex-build/src/compiler.rs|compiler.rs]] (`crates/ferrotex-build/src/compiler.rs`)
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-cli/Cargo.toml|Cargo.toml]] (`crates/ferrotex-cli/Cargo.toml`)
- [[crates/ferrotex-core/Cargo.toml|Cargo.toml]] (`crates/ferrotex-core/Cargo.toml`)
- [[crates/ferrotex-dap/Cargo.toml|Cargo.toml]] (`crates/ferrotex-dap/Cargo.toml`)
- [[crates/ferrotex-log/Cargo.toml|Cargo.toml]] (`crates/ferrotex-log/Cargo.toml`)
- [[crates/ferrotex-log/src/parser.rs|parser.rs]] (`crates/ferrotex-log/src/parser.rs`)
- [[crates/ferrotex-math-semantics/Cargo.toml|Cargo.toml]] (`crates/ferrotex-math-semantics/Cargo.toml`)
- [[crates/ferrotex-math-semantics/src/analysis.rs|analysis.rs]] (`crates/ferrotex-math-semantics/src/analysis.rs`)
- [[crates/ferrotex-package/Cargo.toml|Cargo.toml]] (`crates/ferrotex-package/Cargo.toml`)
- [[crates/ferrotex-package/src/scanner.rs|scanner.rs]] (`crates/ferrotex-package/src/scanner.rs`)
- [[crates/ferrotex-syntax/Cargo.toml|Cargo.toml]] (`crates/ferrotex-syntax/Cargo.toml`)
- [[crates/ferrotex-syntax/src/parser.rs|parser.rs]] (`crates/ferrotex-syntax/src/parser.rs`)
- [[crates/ferrotexd/Cargo.toml|Cargo.toml]] (`crates/ferrotexd/Cargo.toml`)
- [[crates/ferrotexd/src/build/latexmk.rs|latexmk.rs]] (`crates/ferrotexd/src/build/latexmk.rs`)
- [[crates/ferrotexd/src/build/tectonic.rs|tectonic.rs]] (`crates/ferrotexd/src/build/tectonic.rs`)
- [[crates/ferrotexd/src/completer.rs|completer.rs]] (`crates/ferrotexd/src/completer.rs`)
- [[crates/ferrotexd/src/hover.rs|hover.rs]] (`crates/ferrotexd/src/hover.rs`)
- [[crates/ferrotexd/src/macros.rs|macros.rs]] (`crates/ferrotexd/src/macros.rs`)
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/_sass/custom/custom.scss|custom.scss]] (`docs/_sass/custom/custom.scss`)
- [[docs/adrs/0001-typed-event-ir.md|0001-typed-event-ir.md]] (`docs/adrs/0001-typed-event-ir.md`)
- [[docs/adrs/index.md|index.md]] (`docs/adrs/index.md`)
- [[docs/architecture/index.md|index.md]] (`docs/architecture/index.md`)
- [[docs/development/index.md|index.md]] (`docs/development/index.md`)
- [[docs/development/release.md|release.md]] (`docs/development/release.md`)
- [[docs/development/testing.md|testing.md]] (`docs/development/testing.md`)
- [[docs/index.md|index.md]] (`docs/index.md`)
- [[docs/research/index.md|index.md]] (`docs/research/index.md`)
- [[docs/research/reproducibility.md|reproducibility.md]] (`docs/research/reproducibility.md`)
- [[docs/spec/completion.md|completion.md]] (`docs/spec/completion.md`)
- [[docs/spec/configuration.md|configuration.md]] (`docs/spec/configuration.md`)
- [[docs/spec/formatting.md|formatting.md]] (`docs/spec/formatting.md`)
- [[docs/spec/index.md|index.md]] (`docs/spec/index.md`)
- [[docs/spec/log-event-ir.md|log-event-ir.md]] (`docs/spec/log-event-ir.md`)
- [[editors/vscode/.vscodeignore|.vscodeignore]] (`editors/vscode/.vscodeignore`)
- [[editors/vscode/LICENSE.md|LICENSE.md]] (`editors/vscode/LICENSE.md`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[editors/vscode/snippets/latex.json|latex.json]] (`editors/vscode/snippets/latex.json`)
- [[editors/vscode/src/extension.ts|extension.ts]] (`editors/vscode/src/extension.ts`)
- [[editors/vscode/src/test/suite/index.ts|index.ts]] (`editors/vscode/src/test/suite/index.ts`)
- [[fuzz/Cargo.toml|Cargo.toml]] (`fuzz/Cargo.toml`)
- [[package.json|package.json]] (`package.json`)
- [[scripts/ci-run.sh|ci-run.sh]] (`scripts/ci-run.sh`)

