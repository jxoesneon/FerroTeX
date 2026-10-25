---
title: "crates/ferrotex-package/src/lib.rs"
type: source-code
language: rust
crate: ferrotex-package
layer: package
lines: 161
tags:
  - crate/ferrotex-package
  - lang/rust
  - layer/package
  - type/source-code
summary: "[derive(Debug, Clone, Serialize, Deserialize, Default)] — Source-code file (rust) in ferrotex-package subsystem (161 lines)."
---

# `crates/ferrotex-package/src/lib.rs`

> [!info] File Overview
> - **Crate/Domain**: `ferrotex-package` (package)
> - **Language/Type**: `rust` (source-code)
> - **Lines of Code**: `161`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **ferrotex-package** module within the `package` layer of the FerroTex architecture. Provides core Rust data structures, logic, and interfaces compiled into the workspace engine.

## Key Symbols & Interfaces

| Kind | Identifier |
|---|---|
| `struct` | `PackageIndex` |
| `struct` | `PackageMetadata` |
| `fn` | `new` |
| `fn` | `insert` |
| `fn` | `get` |
| `fn` | `cache_path` |
| `fn` | `save_to_cache` |
| `fn` | `save_to_path` |
| `fn` | `load_from_cache` |
| `fn` | `load_from_path` |

## Graph Links & Dependencies

- [[.github/workflows/coverage.yml|coverage.yml]] (`.github/workflows/coverage.yml`)
- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[crates/ferrotex-package/Cargo.toml|Cargo.toml]] (`crates/ferrotex-package/Cargo.toml`)
- [[crates/ferrotex-package/README.md|README.md]] (`crates/ferrotex-package/README.md`)
- [[crates/ferrotex-package/src/scanner.rs|scanner.rs]] (`crates/ferrotex-package/src/scanner.rs`)
- [[docs/adrs/index.md|index.md]] (`docs/adrs/index.md`)
- [[docs/architecture/index.md|index.md]] (`docs/architecture/index.md`)
- [[docs/development/index.md|index.md]] (`docs/development/index.md`)
- [[docs/index.md|index.md]] (`docs/index.md`)
- [[docs/research/index.md|index.md]] (`docs/research/index.md`)
- [[docs/spec/completion.md|completion.md]] (`docs/spec/completion.md`)
- [[docs/spec/index.md|index.md]] (`docs/spec/index.md`)
- [[docs/spec/tex-path-resolution.md|tex-path-resolution.md]] (`docs/spec/tex-path-resolution.md`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[editors/vscode/src/test/suite/index.ts|index.ts]] (`editors/vscode/src/test/suite/index.ts`)
- [[package.json|package.json]] (`package.json`)

