# `ferrotex-cli`

The unified command-line interface and entry point for the FerroTeX ecosystem.

## 🏗️ Overview

`ferrotex-cli` provides a developer-friendly way to interact with the FerroTeX engine and its various subsystems (Build, DAP, LSP, Analysis).

## 🚀 Commands

### `build`

High-level build orchestration. It automatically resolves dependencies and executes the build DAG.

```bash
ferrotex build main.tex
```

### `debug`

Starts a DAP (Debug Adapter Protocol) session. Usually invoked by VS Code, but can be used for protocol testing.

```bash
ferrotex debug --engine tectonic
```

### `parse`

Emits structured JSON events from TeX log files or source files for external tool integration.

```bash
ferrotex parse main.log
```

### `verify`

Validates the integrity of a workspace against its `ferrotex.lock` file.

```bash
ferrotex verify
```

## 🛠️ Configuration

The CLI respects standard environment variables and local project configurations, ensuring a seamless experience in CI/CD environments.

---

_Part of the FerroTeX Scientific Compiler Platform._
