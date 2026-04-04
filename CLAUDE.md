# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🏗️ Architecture Overview

FerroTeX is a modular, research-grade LaTeX platform built with Rust. Its architecture is highly decoupled, revolving around several core, specialized crates:

- **`ferrotex-syntax`**: This is the foundation, responsible for parsing LaTeX source code. It uses the `rowan` parser and is designed for _lossless_ parsing, ensuring that source fidelity (including trivia) is preserved.
- **`ferrotex-analysis`**: The semantic engine. It operates on the parsed Document Object Model (DOM) to provide context-aware completion and structural checks, such as detecting mismatched delimiters in complex environments (e.g., `matrix`).
- **`ferrotex-build`**: Manages the build graph using a Directed Acyclic Graph (DAG). It enforces **content-addressable reproducibility** by hashing all inputs and storing them in `ferrotex.lock` (SHA256).
- **`ferrotex-dap`**: Implements the Debug Adapter Protocol (DAP) for advanced observability. This allows developers to step through TeX compilation passes and inspect internal registers like `\count` or `\dimen`.

The high-level flow is: **Editors/CLI** $\rightarrow$ **`ferrotex-analysis`** $\rightarrow$ **`ferrotex-syntax`** $\rightarrow$ **Tectonic/TeX Engine**.

## 🛠️ Common Development Commands

The project uses standard Cargo conventions.

### 1. Build and Compile

To perform a full, optimized build of the entire system:

```bash
cargo build --release
```

### 2. Running a Specific Analysis

To run the core parsing utility on a log file:

```bash
./target/release/ferrotex parse <path_to_log_file>
```

### 3. Running Tests

To run all unit and integration tests:

```bash
cargo test
```

To run tests for a specific package (e.g., syntax):

```bash
cargo test -p ferrotex-syntax
```

### 4. Docker-based Workflow (CI Replication)

To run build and tests in a clean Ubuntu 22.04 environment (consistent with CI):

```bash
./scripts/ci-run.sh
```

## 🧱 Key Concepts for Development

- **Reproducibility**: Always check and maintain the integrity of `ferrotex.lock`. Changes to _any_ input file or dependency require updating this lock file to maintain hermetic builds.
- **Observability**: Use the `ferrotex-dap` components when debugging complex macro expansion, as standard logging is insufficient.
- **Performance**: Focus on optimizing parsing and analysis using non-allocating techniques where possible, as indicated by the core crates.
