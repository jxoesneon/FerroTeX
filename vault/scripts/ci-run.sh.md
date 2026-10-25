---
title: "scripts/ci-run.sh"
type: script
language: shell
crate: tooling
layer: scripts
lines: 412
tags:
  - crate/tooling
  - lang/shell
  - layer/scripts
  - type/script
summary: "!/usr/bin/env bash — Script file (shell) in tooling subsystem (412 lines)."
---

# `scripts/ci-run.sh`

> [!info] File Overview
> - **Crate/Domain**: `tooling` (scripts)
> - **Language/Type**: `shell` (script)
> - **Lines of Code**: `412`
> - **Hub**: [[FerroTex Overview]] | [[Crates Index]]

## Purpose & System Role
Part of the **tooling** module within the `scripts` layer of the FerroTex architecture. Contains project configuration, workflows, or developer tooling.

## Document Sections

- !/usr/bin/env bash
- ci-run.sh - Local CI Testing Wrapper Script for FerroTeX
- This script provides a unified way to run CI workflows locally using Docker,
- replicating the GitHub Actions CI environment.
- Usage: ./scripts/ci-run.sh [OPTIONS]
- Configuration
- Color codes for output
- Timing
- Command line flags
- Function to print colored output
- Function to print usage information
- Parse command line arguments
- Function to check if Docker is installed and running
- Function to setup signal handlers for graceful interruption
- Function to start timing

## Graph Links & Dependencies

- [[Dockerfile.ci-test|Dockerfile.ci-test]] (`Dockerfile.ci-test`)
- [[FerroTex Overview|Central Knowledge Hub]]
- [[crates/ferrotex-build/src/artifacts.rs|artifacts.rs]] (`crates/ferrotex-build/src/artifacts.rs`)
- [[crates/ferrotex-build/src/error.rs|error.rs]] (`crates/ferrotex-build/src/error.rs`)
- [[crates/ferrotex-core/src/math_validator/tests.rs|tests.rs]] (`crates/ferrotex-core/src/math_validator/tests.rs`)
- [[crates/ferrotex-core/src/package_manager/tests.rs|tests.rs]] (`crates/ferrotex-core/src/package_manager/tests.rs`)
- [[crates/ferrotex-log/src/tests.rs|tests.rs]] (`crates/ferrotex-log/src/tests.rs`)
- [[crates/ferrotexd/src/workspace.rs|workspace.rs]] (`crates/ferrotexd/src/workspace.rs`)
- [[docs/development/setup.md|setup.md]] (`docs/development/setup.md`)

