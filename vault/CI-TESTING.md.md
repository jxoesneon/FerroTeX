---
title: "CI-TESTING.md"
type: doc
language: markdown
crate: workspace-root
layer: root
lines: 122
tags:
  - crate/workspace-root
  - lang/markdown
  - layer/root
  - type/doc
summary: "Local CI Testing Guide — Doc file (markdown) in workspace-root subsystem (122 lines)."
---

# `CI-TESTING.md`

> [!info] File Overview
> - **Crate/Domain**: `workspace-root` (root)
> - **Language/Type**: `markdown` (doc)
> - **Lines of Code**: `122`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **workspace-root** module within the `root` layer of the FerroTex architecture. Defines formal specifications, architectural decision records, or project documentation.

## Document Sections

- Local CI Testing Guide
- Why Use Docker for CI Testing?
- Quick Start
- Prerequisites
- Running the Test (Recommended)
- Run the full CI workflow (check, build, test, clippy)
- Quick check only (fastest feedback)
- See all available options
- Legacy Script
- Manual Docker Commands
- Build the Docker image
- Run a full build
- Run cargo check (faster)
- Run tests
- Run tests for a specific crate (faster iteration)

## Graph Links & Dependencies

- [[.cargo/config.toml|config.toml]] (`.cargo/config.toml`)
- [[.gitignore|.gitignore]] (`.gitignore`)
- [[Cargo.toml|Cargo.toml]] (`Cargo.toml`)
- [[Dockerfile.ci-test|Dockerfile.ci-test]] (`Dockerfile.ci-test`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/artifacts.rs|artifacts.rs]] (`crates/ferrotex-build/src/artifacts.rs`)
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[crates/ferrotex-math-semantics/src/analysis.rs|analysis.rs]] (`crates/ferrotex-math-semantics/src/analysis.rs`)
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/development/setup.md|setup.md]] (`docs/development/setup.md`)
- [[docs/spec/configuration.md|configuration.md]] (`docs/spec/configuration.md`)
- [[editors/vscode/package.json|package.json]] (`editors/vscode/package.json`)
- [[package.json|package.json]] (`package.json`)
- [[scripts/ci-run.sh|ci-run.sh]] (`scripts/ci-run.sh`)

