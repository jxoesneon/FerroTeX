# `ferrotex-package`

The package and metadata management layer of the FerroTeX ecosystem.

## 🏗️ Responsibilities

`ferrotex-package` provides the intelligence needed to understand LaTeX's vast dependency graph and integrate with external repositories like CTAN.

### Key Features

- **CTAN Integration**: Fetches and caches package metadata, providing descriptions and documentation links for hovers.
- **Dependency Resolution**: Analyzes `\usepackage` commands to build a complete dependency tree of the current project.
- **Environment Discovery**: Detects which macros and environments are provided by specific packages to power context-aware completion.

## 💎 Design

Built to be asynchronous and cached, ensuring that external network requests do not block the Language Server's main thread. It serves as the data provider for the completion and hover engines in `ferrotex-analysis`.

---

_Part of the FerroTeX Scientific Compiler Platform._
