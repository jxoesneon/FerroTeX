---
title: "editors/vscode/src/installTectonic.ts"
type: source-code
language: typescript
crate: vscode-extension
layer: editor
tags:
  - crate/vscode-extension
  - lang/typescript
  - layer/editor
  - type/source-code
summary: "Automated self-installer for our customized Tectonic engine from jxoesneon/tectonic prebuilt releases."
---

# `editors/vscode/src/installTectonic.ts`

> [!info] File Overview
> - **Crate/Domain**: `vscode-extension` (editor)
> - **Language/Type**: `typescript` (source-code)
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Implements frictionless, zero-configuration automated provisioning of our customized Tectonic TeX engine (`jxoesneon/tectonic`). Replaces generic package manager prompts with standalone prebuilt binary self-installation.

## Architectural Capabilities
1. **Target Detection**: Maps platform and CPU architecture (`aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-pc-windows-msvc`).
2. **Release Resolution**: Fetches release assets from `https://api.github.com/repos/jxoesneon/tectonic/releases/tags/continuous` with deterministic direct URL fallback.
3. **Extraction & Installation**: Unpacks `.tar.gz` / `.zip` into `globalStorage` and `editors/vscode/bin/tectonic`, setting `755` permissions.
4. **Environment Injection**: Dynamically registers the binary directory into `process.env.PATH` for VS Code sub-processes and daemon child processes.

## Key Symbols & Interfaces

| Kind | Identifier | Description |
|---|---|---|
| `function` | `ensureTectonicBinary` | Resolves bundled, cached, configured, or auto-downloaded Tectonic binary |
| `function` | `checkAndInstallTectonic` | Activation hook ensuring Tectonic engine readiness |

## Graph Links & Dependencies

- [[editors/vscode/src/extension.ts|extension.ts]]
- [[editors/vscode/src/engineValidator.ts|engineValidator.ts]]
- [[crates/ferrotexd/src/build/tectonic.rs|crates/ferrotexd/src/build/tectonic.rs]]
- [[docs/spec/engine-adapters.md|Engine Adapters Specification]]
